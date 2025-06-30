use axum::{Json, extract::State};

use crate::AppState;

const RESPONSE: &str = include_str!(
    "../../../../ai-gateway/stubs/anthropic/messages_success.json"
);

pub(crate) async fn handler(
    State(state): State<AppState>,
) -> Json<serde_json::Value> {
    if state.anthropic_latency > 0 {
        crate::routes::sleep(state.anthropic_latency).await;
    }
    let stub = serde_json::from_str::<serde_json::Value>(RESPONSE).unwrap()
        ["response"]["jsonBody"]
        .clone();
    Json(stub)
}
