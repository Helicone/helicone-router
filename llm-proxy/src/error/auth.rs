use axum_core::response::{IntoResponse, Response};
use displaydoc::Display;
use http::StatusCode;
use tracing::error;

use super::api::ErrorResponse;
use crate::types::json::Json;

#[derive(Debug, strum::AsRefStr, thiserror::Error, Display)]
pub enum AuthError {
    /// Reqwest error: {0}
    Reqwest(#[from] reqwest::Error),
    /// Task join error: {0}
    TaskJoin(#[from] tokio::task::JoinError),
    /// Invalid credentials
    InvalidCredentials,
    /// Internal server error
    InternalServerError,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        if let Self::InvalidCredentials = self {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "Invalid credentials".to_string(),
                }),
            )
                .into_response()
        } else {
            error!(error = %self, "authentication error");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Internal server error".to_string(),
                }),
            )
                .into_response()
        }
    }
}
