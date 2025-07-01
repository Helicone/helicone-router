pub mod routes;

use axum::{
    Router,
    routing::{get, post},
};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AppState {
    #[serde(default = "default_provider_latency")]
    pub openai_latency: u32,
    #[serde(default = "default_provider_latency")]
    pub anthropic_latency: u32,
    #[serde(default = "default_provider_latency")]
    pub gemini_latency: u32,
    #[serde(default = "default_provider_latency")]
    pub bedrock_latency: u32,
    #[serde(default = "default_jawn_latency")]
    pub jawn_latency: u32,
    #[serde(default = "default_minio_latency")]
    pub minio_latency: u32,
    #[serde(default = "default_address")]
    pub address: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            openai_latency: default_provider_latency(),
            anthropic_latency: default_provider_latency(),
            gemini_latency: default_provider_latency(),
            bedrock_latency: default_provider_latency(),
            jawn_latency: 20,
            minio_latency: 5,
            address: "0.0.0.0".to_string(),
            port: 5150,
        }
    }
}

fn default_provider_latency() -> u32 {
    60
}

fn default_jawn_latency() -> u32 {
    20
}

fn default_minio_latency() -> u32 {
    5
}

fn default_address() -> String {
    "[::]".to_string()
}

fn default_port() -> u16 {
    5150
}

pub fn router(app_state: AppState) -> Router {
    Router::new()
        .route("/v1/chat/completions", post(routes::openai::handler))
        .route("/v1/messages", post(routes::anthropic::handler))
        .route(
            "/v1beta/openai/chat/completions",
            post(routes::gemini::handler),
        )
        .route("/model/{modelId}/converse", post(routes::bedrock::handler))
        .route("/v1/log/request", post(routes::jawn::log_request))
        .route(
            "/v1/router/control-plane/sign-s3-url",
            post(routes::jawn::sign_s3_url),
        )
        .route(
            "/request-response-storage/organizations/{org_id}/requests/\
             {request_id}/raw_request_response_body",
            post(routes::minio::upload_request),
        )
        .route(
            "/ws/v1/router/control-plane",
            get(routes::jawn::websocket_handler),
        )
        .with_state(app_state)
}
