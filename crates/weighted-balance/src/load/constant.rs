//! A constant [`Load`] implementation.
//! Copyright (c) 2019 Tower Contributors
//!
//! Permission is hereby granted, free of charge, to any
//! person obtaining a copy of this software and associated
//! documentation files (the "Software"), to deal in the
//! Software without restriction, including without
//! limitation the rights to use, copy, modify, merge,
//! publish, distribute, sublicense, and/or sell copies of
//! the Software, and to permit persons to whom the Software
//! is furnished to do so, subject to the following
//! conditions:
//!
//! The above copyright notice and this permission notice
//! shall be included in all copies or substantial portions
//! of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF
//! ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED
//! TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A
//! PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT
//! SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
//! CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION
//! OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR
//! IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
//! DEALINGS IN THE SOFTWARE.
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{Stream, ready};
use pin_project_lite::pin_project;
use tower::{
    Service,
    discover::{Change, Discover},
    load::Load,
};

use super::weight::Weight;

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct ConstantLoad(usize);

impl From<usize> for ConstantLoad {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl std::ops::Div<Weight> for ConstantLoad {
    type Output = f64;

    fn div(self, rhs: Weight) -> Self::Output {
        (self.0 as f64) / rhs
    }
}

pin_project! {
    #[derive(Debug)]
    /// Wraps a type so that it implements [`Load`] and returns a constant load metric.
    ///
    /// This load estimator is primarily useful for testing.
    pub struct Constant<T> {
        inner: T,
        load: ConstantLoad,
    }
}

// ===== impl Constant =====

impl<T> Constant<T> {
    /// Wraps a `T`-typed service with a constant `M`-typed load metric.
    pub const fn new(inner: T, load: ConstantLoad) -> Self {
        Self { inner, load }
    }
}

impl<T> Load for Constant<T> {
    type Metric = ConstantLoad;

    fn load(&self) -> Self::Metric {
        self.load
    }
}

impl<S, Request> Service<Request> for Constant<S>
where
    S: Service<Request>,
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

    fn call(&mut self, req: Request) -> Self::Future {
        self.inner.call(req)
    }
}

/// Proxies [`Discover`] such that all changes are wrapped with a constant load.
impl<D: Discover + Unpin> Stream for Constant<D> {
    type Item = Result<Change<D::Key, Constant<D::Service>>, D::Error>;

    /// Yields the next discovery change set.
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        use self::Change::*;

        let this = self.project();
        let change = match ready!(Pin::new(this.inner).poll_discover(cx))
            .transpose()?
        {
            None => return Poll::Ready(None),
            Some(Insert(k, svc)) => Insert(k, Constant::new(svc, *this.load)),
            Some(Remove(k)) => Remove(k),
        };

        Poll::Ready(Some(Ok(change)))
    }
}
