use std::path::PathBuf;

use axum_mock_server::{AppState, router};
use clap::Parser;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Parser)]
#[command(version)]
pub struct Args {
    /// Path to the default config file.
    /// Configs in this file can be overridden by environment variables.
    #[arg(short, long)]
    config: Option<PathBuf>,

    #[arg(short, long, default_value = "[::]")]
    address: Option<String>,

    #[arg(short, long, default_value = "5150")]
    port: Option<u16>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| {
                    format!("{}=trace", env!("CARGO_CRATE_NAME")).into()
                }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Args::parse();
    let mut app_state = if let Some(config) = args.config {
        serde_yml::from_str(&std::fs::read_to_string(config).unwrap()).unwrap()
    } else {
        AppState::default()
    };
    if let Some(address) = args.address {
        app_state.address = address;
    }
    if let Some(port) = args.port {
        app_state.port = port;
    }
    let app = router(app_state.clone());

    // run it
    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        app_state.address, app_state.port
    ))
    .await
    .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
