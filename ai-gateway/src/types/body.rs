use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
};

pub use axum_core::body::Body;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use hyper::body::{Body as _, Frame, SizeHint};
use tokio::sync::{
    mpsc::{self, UnboundedReceiver},
    oneshot,
};

use crate::error::internal::InternalError;

/// Reads a stream of HTTP data frames as `Bytes` from a channel.
#[derive(Debug)]
pub struct BodyReader {
    rx: UnboundedReceiver<Bytes>,
    tfft_tx: Option<oneshot::Sender<()>>,
    is_end_stream: bool,
    size_hint: SizeHint,
}

impl BodyReader {
    #[must_use]
    pub fn new(
        rx: UnboundedReceiver<Bytes>,
        tfft_tx: oneshot::Sender<()>,
        size_hint: SizeHint,
    ) -> Self {
        Self {
            rx,
            tfft_tx: Some(tfft_tx),
            is_end_stream: false,
            size_hint,
        }
    }

    /// `append_newlines` is used to support LLM response logging with Helicone
    /// for streaming responses.
    pub fn wrap_stream(
        stream: impl Stream<Item = Result<Bytes, InternalError>> + Send + 'static,
    ) -> (axum_core::body::Body, BodyReader, oneshot::Receiver<()>) {
        // unbounded channel is okay since we limit memory usage higher in the
        // stack by limiting concurrency and request/response body size.
        let (tx, rx) = mpsc::unbounded_channel();
        let (tfft_tx, tfft_rx) = oneshot::channel();
        let s = stream.map(move |b| {
            match &b {
                Ok(b) => {
                    if let Err(e) = tx.send(b.clone()) {
                        tracing::error!(error = %e, "BodyReader dropped before stream ended");
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "encountered internal error in stream");
                }
            }
            b
        });
        let inner = axum_core::body::Body::from_stream(s);
        let size_hint = inner.size_hint();
        (inner, BodyReader::new(rx, tfft_tx, size_hint), tfft_rx)
    }
}

impl hyper::body::Body for BodyReader {
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        match Pin::new(&mut self.rx).poll_recv(cx) {
            Poll::Ready(Some(bytes)) => {
                if let Some(tfft_tx) = self.tfft_tx.take() {
                    if let Err(()) = tfft_tx.send(()) {
                        tracing::error!("Failed to send TFFT signal");
                    }
                }

                Poll::Ready(Some(Ok(Frame::data(bytes))))
            }
            Poll::Ready(None) => {
                self.is_end_stream = true;
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }

    fn is_end_stream(&self) -> bool {
        self.is_end_stream
    }

    fn size_hint(&self) -> SizeHint {
        self.size_hint.clone()
    }
}
