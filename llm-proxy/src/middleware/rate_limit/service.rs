use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use brakes::{
    RateLimiter, backend::redis::RedisBackend, types::token_bucket::TokenBucket,
};

use super::{brakes::TowerRateLimiterLayer, extractor::RateLimitKeyExtractor};
use crate::{app::AppState, config::rate_limit::RateLimitConfig};

pub type RateLimitService<S> =
    crate::middleware::rate_limit::brakes::TowerRateLimiter<
        S,
        TokenBucket,
        RedisBackend,
        RateLimitKeyExtractor,
    >;

#[derive(Clone)]
pub struct Layer {
    inner: Option<
        crate::middleware::rate_limit::brakes::TowerRateLimiterLayer<
            TokenBucket,
            RedisBackend,
            RateLimitKeyExtractor,
        >,
    >,
}

impl Layer {
    /// Create a new rate limit layer.
    ///
    /// If the config is `None`, the layer will be a no-op.
    #[must_use]
    pub fn new(app_state: &AppState) -> Self {
        match &app_state.0.config.rate_limit {
            RateLimitConfig::Enabled { limits, redis: _ } => {
                let strategy = TokenBucket::new(
                    limits.per_user.capacity,
                    limits.per_user.fill_frequency,
                );
                let backend = RedisBackend::new(
                    app_state
                        .0
                        .redis
                        .as_ref()
                        .expect("redis is required")
                        .clone(),
                );
                let limiter = RateLimiter::builder()
                    .with_backend(backend)
                    .with_limiter(strategy)
                    .with_failure_strategy(brakes::RetryStrategy::RetryAndDeny(
                        1,
                    ))
                    .with_conflict_strategy(
                        brakes::RetryStrategy::RetryAndDeny(1),
                    )
                    .build();

                let inner = Some(TowerRateLimiterLayer::new(
                    limiter,
                    RateLimitKeyExtractor,
                ));
                Self { inner }
            }
            RateLimitConfig::Disabled => Self { inner: None },
        }
    }
}

impl<S> tower::layer::Layer<S> for Layer {
    type Service = Service<S>;

    fn layer(&self, service: S) -> Self::Service {
        match &self.inner {
            Some(inner) => Service::Enabled {
                service: inner.layer(service),
            },
            None => Service::Disabled { service },
        }
    }
}

#[derive(Debug, Clone)]
pub enum Service<S> {
    Enabled { service: RateLimitService<S> },
    Disabled { service: S },
}

pin_project_lite::pin_project! {
    #[derive(Debug)]
    #[project = EnumProj]
    pub enum ResponseFuture<EnabledFuture, DisabledFuture> {
        Enabled { #[pin] future: EnabledFuture },
        Disabled { #[pin] future: DisabledFuture },
    }
}

impl<EnabledFuture, DisabledFuture, Response, Error> Future
    for ResponseFuture<EnabledFuture, DisabledFuture>
where
    EnabledFuture: Future<Output = Result<Response, Error>>,
    DisabledFuture: Future<Output = Result<Response, Error>>,
{
    type Output = Result<Response, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            EnumProj::Enabled { future } => future.poll(cx),
            EnumProj::Disabled { future } => future.poll(cx),
        }
    }
}

impl<S, Request> tower::Service<Request> for Service<S>
where
    S: tower::Service<Request>,
    RateLimitService<S>:
        tower::Service<Request, Response = S::Response, Error = S::Error>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = ResponseFuture<
        <RateLimitService<S> as tower::Service<Request>>::Future,
        S::Future,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        match self {
            Service::Enabled { service } => service.poll_ready(cx),
            Service::Disabled { service } => service.poll_ready(cx),
        }
    }

    fn call(&mut self, request: Request) -> Self::Future {
        match self {
            Service::Enabled { service } => ResponseFuture::Enabled {
                future: service.call(request),
            },
            Service::Disabled { service } => ResponseFuture::Disabled {
                future: service.call(request),
            },
        }
    }
}
