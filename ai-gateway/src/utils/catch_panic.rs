use std::any::Any;

use axum_core::response::IntoResponse;
use tower_http::catch_panic::ResponseForPanic;

use crate::error::api::ApiError;

#[derive(Debug, Clone, Copy)]
pub struct PanicResponder;

impl ResponseForPanic for PanicResponder {
    type ResponseBody = axum_core::body::Body;
    fn response_for_panic(
        &mut self,
        err: Box<dyn Any + Send + 'static>,
    ) -> http::Response<axum_core::body::Body> {
        let details = if let Some(s) = err.downcast_ref::<String>() {
            s.clone()
        } else if let Some(s) = err.downcast_ref::<&str>() {
            s.to_string()
        } else {
            "Service panicked but `CatchPanic` was unable to downcast the \
             panic info"
                .to_string()
        };
        ApiError::Panic(details).into_response()
    }
}
