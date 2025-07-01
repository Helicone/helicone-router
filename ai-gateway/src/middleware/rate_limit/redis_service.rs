use std::{
    sync::Arc,
    task::{Context, Poll},
};

use futures::future::BoxFuture;

use crate::{
    config::rate_limit::LimitsConfig,
    error::api::ApiError,
    types::{request::Request, response::Response},
};

#[derive(Debug, Clone)]
pub struct RedisRateLimitLayer {
    pub config: Arc<LimitsConfig>,
}

impl RedisRateLimitLayer {
    pub fn new(config: Arc<LimitsConfig>) -> Self {
        Self { config }
    }
}

impl<S> tower::layer::Layer<S> for RedisRateLimitLayer {
    type Service = RedisRateLimitService<S>;

    fn layer(&self, service: S) -> Self::Service {
        // RedisRateLimitService::new(service, self.config.clone())
        RedisRateLimitService {
            inner: service,
            config: self.config.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RedisRateLimitService<S> {
    pub inner: S,
    pub config: Arc<LimitsConfig>,
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
        self.inner.poll_ready(cx)
    }

    #[tracing::instrument(name = "rate_limit", skip_all)]
    fn call(&mut self, req: Request) -> Self::Future {
        tracing::trace!("rate_limit middleware");
        // see: https://docs.rs/tower/latest/tower/trait.Service.html#be-careful-when-cloning-inner-services
        let mut this = self.clone();
        std::mem::swap(self, &mut this);
        Box::pin(async move {
            make_request(&mut this.inner, &this.config, req).await
        })
    }
}

async fn make_request<S>(
    inner: &mut S,
    _config: &LimitsConfig,
    req: Request,
) -> Result<Response, ApiError>
where
    S: tower::Service<Request, Response = Response, Error = ApiError>
        + Send
        + Clone
        + 'static,
    S::Future: Send + 'static,
{
    tracing::info!("doing some rate limiting logic");
    inner.call(req).await
}
