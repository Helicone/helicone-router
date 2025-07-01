use axum_core::response::IntoResponse;
use displaydoc::Display;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use utoipa::ToSchema;

use super::{
    ErrorMetric,
    auth::{AuthError, AuthErrorMetric},
    internal::{InternalError, InternalErrorMetric},
    invalid_req::{InvalidRequestError, InvalidRequestErrorMetric},
};
use crate::{
    middleware::mapper::openai::{
        INVALID_REQUEST_ERROR_TYPE, SERVER_ERROR_TYPE,
    },
    types::json::Json,
};

/// Common API errors
#[derive(Debug, Error, Display, strum::AsRefStr)]
pub enum ApiError {
    /// Invalid request: {0}
    InvalidRequest(#[from] InvalidRequestError),
    /// Authentication error: {0}
    Authentication(#[from] AuthError),
    /// Internal error: {0}
    Internal(#[from] InternalError),
    /// Stream error: {0}
    StreamError(#[from] Box<reqwest_eventsource::Error>),
    /// Service panicked: {0}
    Panic(String),
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

/// This type is intended to mirror the error type returned by the `OpenAI` API.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct ErrorDetails {
    pub message: String,
    pub r#type: Option<String>,
    pub param: Option<String>,
    pub code: Option<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum_core::response::Response {
        match self {
            ApiError::InvalidRequest(error) => error.into_response(),
            ApiError::Authentication(error) => error.into_response(),
            ApiError::Internal(error) => error.into_response(),
            ApiError::StreamError(error) => {
                if let reqwest_eventsource::Error::InvalidStatusCode(
                    status_code,
                    _response,
                ) = &*error
                {
                    if status_code.is_server_error() {
                        tracing::error!(error = %error, "upstream server error in stream");
                        (
                            *status_code,
                            Json(ErrorResponse {
                                error: ErrorDetails {
                                    message: error.to_string(),
                                    r#type: Some("server_error".to_string()),
                                    param: None,
                                    code: None,
                                },
                            }),
                        )
                            .into_response()
                    } else if status_code.is_client_error() {
                        tracing::debug!(error = %error, "invalid request error in stream");
                        (
                            *status_code,
                            Json(ErrorResponse {
                                error: ErrorDetails {
                                    message: error.to_string(),
                                    r#type: Some(
                                        INVALID_REQUEST_ERROR_TYPE.to_string(),
                                    ),
                                    param: None,
                                    code: None,
                                },
                            }),
                        )
                            .into_response()
                    } else {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(ErrorResponse {
                                error: ErrorDetails {
                                    message: error.to_string(),
                                    r#type: Some("server_error".to_string()),
                                    param: None,
                                    code: None,
                                },
                            }),
                        )
                            .into_response()
                    }
                } else {
                    tracing::error!(error = %error, "internal error in stream");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: ErrorDetails {
                                message: error.to_string(),
                                r#type: Some("server_error".to_string()),
                                param: None,
                                code: None,
                            },
                        }),
                    )
                        .into_response()
                }
            }
            ApiError::Panic(error) => {
                tracing::error!(error = %error, "Internal server error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: ErrorDetails {
                            message: "Internal server error".to_string(),
                            r#type: Some(SERVER_ERROR_TYPE.to_string()),
                            param: None,
                            code: None,
                        },
                    }),
                )
                    .into_response()
            }
        }
    }
}

/// Top level metric type that reduces cardinality such that
/// we can use the error type in metrics.
#[derive(Debug, Error, Display, strum::AsRefStr)]
pub enum ApiErrorMetric {
    /// Invalid request
    InvalidRequest(#[from] InvalidRequestErrorMetric),
    /// Authentication
    Authentication(#[from] AuthErrorMetric),
    /// Internal
    Internal(#[from] InternalErrorMetric),
    /// Panic
    Panic,
}

impl From<&ApiError> for ApiErrorMetric {
    fn from(error: &ApiError) -> Self {
        match error {
            ApiError::InvalidRequest(invalid_request_error) => {
                Self::InvalidRequest(InvalidRequestErrorMetric::from(
                    invalid_request_error,
                ))
            }
            ApiError::Authentication(auth_error) => {
                Self::Authentication(AuthErrorMetric::from(auth_error))
            }
            ApiError::Internal(internal_error) => {
                Self::Internal(InternalErrorMetric::from(internal_error))
            }
            ApiError::StreamError(error) => match &**error {
                reqwest_eventsource::Error::InvalidStatusCode(
                    status_code,
                    _response,
                ) if status_code.is_client_error() => Self::InvalidRequest(
                    InvalidRequestErrorMetric::InvalidRequest,
                ),
                _ => Self::Internal(InternalErrorMetric::Internal),
            },
            ApiError::Panic(_error) => Self::Panic,
        }
    }
}

impl ErrorMetric for ApiErrorMetric {
    fn error_metric(&self) -> String {
        match self {
            Self::InvalidRequest(error) => {
                format!("InvalidRequest:{}", error.as_ref())
            }
            Self::Authentication(error) => {
                format!("Authentication:{}", error.as_ref())
            }
            Self::Internal(error) => {
                if let InternalErrorMetric::MapperError(e) = error {
                    format!("InternalError:MapperError:{}", e.as_ref())
                } else {
                    format!("InternalError:{}", error.as_ref())
                }
            }
            Self::Panic => String::from("Panic"),
        }
    }
}

impl ErrorMetric for ApiError {
    fn error_metric(&self) -> String {
        ApiErrorMetric::from(self).error_metric()
    }
}
