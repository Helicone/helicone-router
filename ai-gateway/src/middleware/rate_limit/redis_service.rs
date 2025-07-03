use std::{
    sync::Arc,
    task::{Context, Poll},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use axum_core::response::Response;
use futures::future::BoxFuture;
use r2d2::Pool;
use redis::{Client, Commands};

use crate::{
    config::rate_limit::{LimitsConfig, default_refill_frequency},
    error::{
        api::ApiError,
        init::InitError,
        internal::InternalError,
        invalid_req::{InvalidRequestError, TooManyRequestsError},
    },
    middleware::rate_limit::extractor::get_redis_rl_key,
    types::request::Request,
};

#[derive(Debug, Clone)]
pub struct RedisRateLimitLayer {
    pub config: Arc<LimitsConfig>,
    pub pool: Pool<Client>,
}

impl RedisRateLimitLayer {
    pub fn new(
        config: Arc<LimitsConfig>,
        url: url::Url,
    ) -> Result<Self, InitError> {
        let client = Client::open(url)?;
        let pool = Pool::builder().build(client)?;
        Ok(Self { config, pool })
    }
}

impl<S> tower::layer::Layer<S> for RedisRateLimitLayer {
    type Service = RedisRateLimitService<S>;

    fn layer(&self, service: S) -> Self::Service {
        RedisRateLimitService::new(
            service,
            self.config.clone(),
            self.pool.clone(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct RedisRateLimitService<S> {
    pub inner: S,
    pub config: Arc<LimitsConfig>,
    pub pool: Pool<Client>,
}

impl<S> RedisRateLimitService<S> {
    pub fn new(
        inner: S,
        config: Arc<LimitsConfig>,
        pool: Pool<Client>,
    ) -> Self {
        Self {
            inner,
            config,
            pool,
        }
    }
}

impl<S> tower::Service<Request> for RedisRateLimitService<S>
where
    S: tower::Service<Request, Response = Response, Error = ApiError>
        + Send
        + Clone
        + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = ApiError;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(cx)
            .map_err(|_| ApiError::Internal(InternalError::Internal))
    }

    #[tracing::instrument(name = "rate_limit", skip_all)]
    fn call(&mut self, req: Request) -> Self::Future {
        tracing::trace!("rate_limit middleware");
        // see: https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let mut this = self.clone();
        std::mem::swap(self, &mut this);
        Box::pin(async move {
            make_request(&mut this.inner, &this.config, &this.pool, req).await
        })
    }
}

async fn make_request<S>(
    inner: &mut S,
    config: &LimitsConfig,
    pool: &Pool<Client>,
    req: Request,
) -> Result<Response, ApiError>
where
    S: tower::Service<Request, Response = Response, Error = ApiError>
        + Send
        + Clone
        + 'static,
    S::Future: Send + 'static,
{
    tracing::info!("making request with redis on config: {:?}", config);
    let mut conn = pool.get().map_err(InternalError::PoolError)?;

    let key = get_redis_rl_key(&req)?;

    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let gcra = &config.per_api_key;
    let interval_per_token_ms = gcra
        .refill_frequency
        .checked_div(gcra.capacity.into())
        .unwrap_or_else(|| {
            tracing::warn!(
                "fill_frequency is too small for capacity, using default fill \
                 frequency"
            );
            default_refill_frequency()
        })
        .as_millis();

    // get previous theoretical arrival time (TAT)
    let existing_tat: Option<u128> =
        conn.get(&key).map_err(InternalError::RedisError)?;
    let tat = existing_tat.unwrap_or(now_ms);

    let new_tat = if tat < now_ms {
        now_ms + interval_per_token_ms
    } else {
        tat + interval_per_token_ms
    };

    let earliest_allowed_time =
        new_tat - (interval_per_token_ms * u128::from(gcra.capacity.get()));

    if earliest_allowed_time <= now_ms {
        let _: () =
            conn.set(&key, new_tat).map_err(InternalError::RedisError)?;

        let ratelimit_remaining = u128::from(gcra.capacity.get())
            - (new_tat - now_ms) / interval_per_token_ms;

        let ratelimit_limit = u64::from(gcra.capacity.get());

        if let Ok(mut res) = inner.call(req).await {
            res.headers_mut().insert(
                "X-RateLimit-Limit",
                ratelimit_limit.to_string().parse().unwrap(),
            );
            res.headers_mut().insert(
                "X-RateLimit-Remaining",
                ratelimit_remaining.to_string().parse().unwrap(),
            );
            Ok(res)
        } else {
            Err(ApiError::Internal(InternalError::Internal))
        }
    } else {
        let ratelimit_limit = u64::from(gcra.capacity.get());
        let ratelimit_remaining = 0;
        let difference = earliest_allowed_time - now_ms;
        let ratelimit_after = Duration::from_millis(
            difference.try_into().expect("value too large"),
        )
        .as_secs()
            + 1; // adding a second to retry-after header to prevent rounding errors
        Err(ApiError::InvalidRequest(
            InvalidRequestError::TooManyRequests(TooManyRequestsError {
                ratelimit_limit,
                ratelimit_remaining,
                ratelimit_after,
            }),
        ))
    }
}
