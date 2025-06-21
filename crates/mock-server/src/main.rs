use ai_gateway::tests::mock::{Mock, MockArgs};
use stubr::wiremock_rs::ResponseTemplate;
use tokio::{
    main,
    signal::unix::{SignalKind, signal},
};
use tracing::{info, warn};
use tracing_subscriber::{
    EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt,
};

#[main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_writer(std::io::stdout),
        )
        .with(EnvFilter::new("info"))
        .init();

    let args = MockArgs::builder()
        .openai_port(8100)
        .anthropic_port(8101)
        .google_port(8102)
        .minio_port(8103)
        .jawn_port(8104)
        // .global_openai_latency(10)
        // .global_anthropic_latency(10)
        // .global_google_latency(10)
        .build();

    info!("Starting mock server");
    let mock = Mock::from_args(args).await;
    let minio_mock = &mock.minio_mock;
    let jawn_mock = &mock.jawn_mock;

    // the reason we do this hack is because the presigned url needs the
    // port of the minio mock, which is dynamic, so we can't use regular
    // stubs here.
    let minio_base_url = minio_mock.http_server.uri();
    let presigned_url = format!(
        "{minio_base_url}/request-response-storage/organizations/\
         c3bc2b69-c55c-4dfc-8a29-47db1245ee7c/requests/\
         a41cbcd7-5e9e-4104-b29b-2ef4473d71a7/raw_request_response_body"
    );
    info!(
        url = %presigned_url,
        "setting up presigned url mock"
    );
    let presigned_url_mock =
        ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "url": presigned_url
            }
        }));
    stubr::wiremock_rs::Mock::given(stubr::wiremock_rs::matchers::method(
        "POST",
    ))
    .and(stubr::wiremock_rs::matchers::path(
        "/v1/router/control-plane/sign-s3-url",
    ))
    .respond_with(presigned_url_mock)
    .named("success:jawn:sign_s3_url")
    .mount(&jawn_mock.http_server)
    .await;

    info!("Mock server started successfully");
    let mut sigint = signal(SignalKind::interrupt())
        .expect("failed to register SIGINT signal");
    let mut sigterm = signal(SignalKind::terminate())
        .expect("failed to register SIGTERM signal");

    tokio::select! {
        _ = sigint.recv() => {
            warn!("SIGINT received, starting shutdown");
        },
        _ = sigterm.recv() => {
            warn!("SIGTERM received, starting shutdown");
        },
    }
}
