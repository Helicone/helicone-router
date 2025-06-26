use std::{
    marker::PhantomData,
    task::{Context, Poll},
};

use chrono::Utc;
use tokio::time::Instant;
use tower::{Layer, Service};

#[derive(Debug, Clone)]
pub struct TimerLayer<ReqBody> {
    _marker: PhantomData<ReqBody>,
}

impl<ReqBody> TimerLayer<ReqBody> {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<ReqBody> Default for TimerLayer<ReqBody> {
    fn default() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<S, ReqBody> Layer<S> for TimerLayer<ReqBody>
where
    S: tower::Service<http::Request<ReqBody>>,
{
    type Service = Timer<S, ReqBody>;

    fn layer(&self, inner: S) -> Self::Service {
        Timer::new(inner)
    }
}

#[derive(Debug)]
pub struct Timer<S, ReqBody> {
    inner: S,
    _marker: PhantomData<ReqBody>,
}

impl<S: Clone, ReqBody> Clone for Timer<S, ReqBody> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            _marker: PhantomData,
        }
    }
}

impl<S, ReqBody> Timer<S, ReqBody>
where
    S: tower::Service<http::Request<ReqBody>>,
{
    pub const fn new(inner: S) -> Self {
        Self {
            inner,
            _marker: PhantomData,
        }
    }
}

impl<S, ReqBody> Service<http::Request<ReqBody>> for Timer<S, ReqBody>
where
    S: Service<http::Request<ReqBody>> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: http::Request<ReqBody>) -> Self::Future {
        // why two? Instant doesn't serialize well, and there's no way to
        // convert the two
        let start_time = Instant::now();
        let req_start_dt = Utc::now();
        req.extensions_mut().insert(start_time);
        req.extensions_mut().insert(req_start_dt);
        tracing::info!(
            "adding start time and request start time to req extensions"
        );
        self.inner.call(req)
    }
}
