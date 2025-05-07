use std::collections::{HashMap, HashSet};

use http::{Method, Request, StatusCode};
use http_body_util::BodyExt;
use llm_proxy::{
    config::{
        Config,
        router::{BalanceConfig, BalanceTarget, RouterConfig, RouterConfigs},
    },
    tests::{TestDefault, harness::Harness, mock::MockArgs},
    types::{provider::Provider, router::RouterId},
};
use nonempty_collections::nev;
use rust_decimal::Decimal;
use serde_json::json;
use tower::Service;

#[tokio::test]
#[serial_test::serial]
async fn weighted_balancer() {
    let mut config = Config::test_default();
    let balance_config = BalanceConfig::Weighted {
        targets: nev![
            BalanceTarget {
                provider: Provider::OpenAI,
                weight: Decimal::try_from(0.25).unwrap(),
            },
            BalanceTarget {
                provider: Provider::Anthropic,
                weight: Decimal::try_from(0.75).unwrap(),
            },
        ],
    };
    config.routers = RouterConfigs::new(HashMap::from([(
        RouterId::Default,
        RouterConfig {
            balance: balance_config,
            ..Default::default()
        },
    )]));
    let mock_args = MockArgs::builder()
        .stubs(HashMap::from([
            ("success:openai:chat_completion", (23..28).into()),
            ("success:anthropic:messages", (73..78).into()),
            ("success:minio:upload_request", 100.into()),
            ("success:jawn:log_request", 100.into()),
        ]))
        .build();
    let mut harness = Harness::builder()
        .with_config(config)
        .with_mock_args(mock_args)
        .build()
        .await;
    let num_requests = 100;
    let body_bytes = serde_json::to_vec(&json!({
        "model": "gpt-4o-mini",
        "messages": [
            {
                "role": "user",
                "content": "Hello, world!"
            }
        ]
    }))
    .unwrap();

    for _ in 0..num_requests {
        let request_body = axum_core::body::Body::from(body_bytes.clone());
        let request = Request::builder()
            .method(Method::POST)
            // default router
            .uri("http://router.helicone.com/router/v1/chat/completions")
            .body(request_body)
            .unwrap();
        let response = harness.call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        // we need to collect the body here in order to poll the underlying body
        // so that the async logging task can complete
        let _response_body = response.into_body().collect().await.unwrap();
    }

    // sleep so that the background task for logging can complete
    // the proper way to write this test without a sleep is to
    // test it at the dispatcher level by returning a handle
    // to the async task and awaiting it in the test.
    //
    // but this is totes good for now
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    harness.mock.jawn_mock.verify().await;
    harness.mock.minio_mock.verify().await;
    harness.mock.openai_mock.verify().await;
    harness.mock.anthropic_mock.verify().await;
}
