use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::AppState;

pub(crate) async fn upload_request(
    State(state): State<AppState>,
) -> impl IntoResponse {
    if state.minio_latency > 0 {
        crate::routes::sleep(state.minio_latency).await;
    }
    StatusCode::OK
}
