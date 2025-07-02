use std::{
    sync::Arc,
    task::{Context, Poll},
    time::{SystemTime, UNIX_EPOCH},
};

use axum_core::response::{IntoResponse, Response};
use futures::future::BoxFuture;
use http::StatusCode;
use r2d2::Pool;
use redis::{Client, Commands};

use crate::{
    config::rate_limit::{LimitsConfig, default_refill_frequency},
    error::{api::ApiError, internal::InternalError},
    types::{extensions::AuthContext, request::Request},
};

#[derive(Debug, Clone)]
pub struct RedisRateLimitLayer {
    pub config: Arc<LimitsConfig>,
    pub url: url::Url,
}

impl RedisRateLimitLayer {
    pub fn new(config: Arc<LimitsConfig>, url: url::Url) -> Self {
        Self { config, url }
    }
}

impl<S> tower::layer::Layer<S> for RedisRateLimitLayer {
    type Service = RedisRateLimitService<S>;

    fn layer(&self, service: S) -> Self::Service {
        RedisRateLimitService::new(
            service,
            self.config.clone(),
            self.url.clone(),
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
    // TODO: handle errors - ask @tom
    pub fn new(inner: S, config: Arc<LimitsConfig>, url: url::Url) -> Self {
        tracing::info!("connecting to redis at {}", url);
        let client = Client::open(url).unwrap();
        let pool = Pool::builder().build(client).unwrap();
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

    let Some(ctx) = req.extensions().get::<AuthContext>() else {
        return Err(ApiError::Internal(InternalError::ExtensionNotFound(
            "AuthContext",
        )));
    };

    let key = format!("rl:{}", ctx.user_id);

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
    tracing::info!("existing_tat: {:?}", existing_tat);
    let tat = existing_tat.unwrap_or(now_ms);
    tracing::info!("tat: {:?}", tat);

    let new_tat = if tat < now_ms {
        now_ms + interval_per_token_ms
    } else {
        tat + interval_per_token_ms
    };

    let earliest_allowed_time =
        new_tat - (interval_per_token_ms * gcra.capacity.get() as u128);

    if earliest_allowed_time <= now_ms {
        let _: () =
            conn.set(&key, new_tat).map_err(InternalError::RedisError)?;
        return inner.call(req).await.map_err(|e| {
            tracing::error!("error calling inner service: {:?}", e);
            ApiError::Internal(InternalError::Internal)
        });
    } else {
        let response = (StatusCode::TOO_MANY_REQUESTS, "Too Many Requests")
            .into_response();
        Ok(response)
    }
}
