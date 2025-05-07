use std::collections::HashMap;

use http::{Method, Request, StatusCode};
use llm_proxy::{
    config::{
        Config,
        router::{BalanceConfig, RouterConfig, RouterConfigs},
    },
    tests::{TestDefault, harness::Harness, mock::MockArgs},
    types::{provider::Provider, router::RouterId},
};
use nonempty_collections::nev;
use serde_json::json;
use tower::Service;

/// Sending a request to https://localhost/router should
/// result in the proxied request targeting https://api.openai.com/v1/chat/completions
#[tokio::test]
#[serial_test::serial(default_mock)]
async fn openai() {
    let config = Config::test_default();
    let mock_args = MockArgs::builder()
        .stubs(HashMap::from([(
            "success:openai:chat_completion",
            1.into(),
        )]))
        .build();
    let mut harness = Harness::builder()
        .with_config(config)
        .with_mock_args(mock_args)
        .build()
        .await;
    let request_body = axum_core::body::Body::from(
        serde_json::to_vec(&json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello, world!"
                }
            ]
        }))
        .unwrap(),
    );
    let request = Request::builder()
        .method(Method::POST)
        // default router
        .uri("http://router.helicone.com/router/v1/chat/completions")
        .body(request_body)
        .unwrap();
    let response = harness.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // technically verification happens on drop but we do it here to be explicit
    harness.mock.openai_mock.verify().await;
}

/// Sending a request to https://localhost/router should
/// result in the proxied request targeting https://api.openai.com/v1/chat/completions
#[tokio::test]
#[serial_test::serial(default_mock)]
async fn anthropic_with_openai_request_style() {
    let mut config = Config::test_default();
    let router_config = RouterConfigs::new(HashMap::from([(
        RouterId::Default,
        RouterConfig {
            request_style: Provider::OpenAI,
            providers: nev![Provider::Anthropic],
            cache: None,
            fallback: None,
            balance: BalanceConfig::P2C {
                targets: nev![Provider::Anthropic],
            },
            retries: None,
            rate_limit: None,
            spend_control: None,
        },
    )]));
    config.routers = router_config;
    let mock_args = MockArgs::builder()
        .stubs(HashMap::from([("success:anthropic:messages", 1.into())]))
        .build();
    let mut harness = Harness::builder()
        .with_config(config)
        .with_mock_args(mock_args)
        .build()
        .await;
    let request_body = axum_core::body::Body::from(
        serde_json::to_vec(&json!({
            "model": "claude-3-5-sonnet-20240620",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello, world!"
                }
            ]
        }))
        .unwrap(),
    );
    let request = Request::builder()
        .method(Method::POST)
        // default router
        .uri("http://router.helicone.com/router/v1/chat/completions")
        .body(request_body)
        .unwrap();
    let response = harness.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // assert that the request was proxied to the mock server correctly
    harness.mock.anthropic_mock.verify().await;
    harness
        .mock
        .anthropic_mock
        .set_expectation("success:anthropic:messages", 2.into())
        .await;

    // test that using an openai model name works as well
    let request_body = axum_core::body::Body::from(
        serde_json::to_vec(&json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello, world!"
                }
            ]
        }))
        .unwrap(),
    );
    let request = Request::builder()
        .method(Method::POST)
        // default router
        .uri("http://router.helicone.com/router/v1/chat/completions")
        .body(request_body)
        .unwrap();
    let response = harness.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // assert that the request was proxied to the mock server correctly
    harness.mock.anthropic_mock.verify().await;
}

#[tokio::test]
#[serial_test::serial(default_mock)]
async fn anthropic_with_anthropic_request_style() {
    let mut config = Config::test_default();
    let router_config = RouterConfigs::new(HashMap::from([(
        RouterId::Default,
        RouterConfig {
            request_style: Provider::OpenAI,
            providers: nev![Provider::Anthropic],
            cache: None,
            fallback: None,
            balance: BalanceConfig::P2C {
                targets: nev![Provider::Anthropic],
            },
            retries: None,
            rate_limit: None,
            spend_control: None,
        },
    )]));
    config.routers = router_config;
    let mock_args = MockArgs::builder()
        .stubs(HashMap::from([("success:anthropic:messages", 1.into())]))
        .build();
    let mut harness = Harness::builder()
        .with_config(config)
        .with_mock_args(mock_args)
        .build()
        .await;
    let request_body = axum_core::body::Body::from(
        serde_json::to_vec(&json!({
            "model": "claude-3-5-sonnet-20240620",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello, world!"
                }
            ]
        }))
        .unwrap(),
    );
    let request = Request::builder()
        .method(Method::POST)
        // default router
        .uri("http://router.helicone.com/router/v1/chat/completions")
        .body(request_body)
        .unwrap();
    let response = harness.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // assert that the request was proxied to the mock server correctly
    harness.mock.anthropic_mock.verify().await;
    // update the expectation to 2 requests
    harness
        .mock
        .anthropic_mock
        .set_expectation("success:anthropic:messages", 2.into())
        .await;

    // test that using an openai model name works as well
    let request_body = axum_core::body::Body::from(
        serde_json::to_vec(&json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello, world!"
                }
            ]
        }))
        .unwrap(),
    );
    let request = Request::builder()
        .method(Method::POST)
        // default router
        .uri("http://router.helicone.com/router/v1/chat/completions")
        .body(request_body)
        .unwrap();
    let response = harness.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    // assert that the request was proxied to the mock server correctly
    harness.mock.anthropic_mock.verify().await;
}

/// Sending a request to https://localhost/router should
/// result in the proxied request targeting https://api.openai.com/v1/chat/completions
#[tokio::test]
#[serial_test::serial(default_mock)]
async fn anthropic_request_style() {
    let mut config = Config::test_default();
    let router_config = RouterConfigs::new(HashMap::from([(
        RouterId::Default,
        RouterConfig {
            request_style: Provider::Anthropic,
            providers: nev![Provider::OpenAI],
            cache: None,
            fallback: None,
            balance: BalanceConfig::P2C {
                targets: nev![Provider::OpenAI],
            },
            retries: None,
            rate_limit: None,
            spend_control: None,
        },
    )]));
    config.routers = router_config;
    let mock_args = MockArgs::builder()
        .stubs(HashMap::from([(
            "success:openai:chat_completion",
            1.into(),
        )]))
        .build();
    let mut harness = Harness::builder()
        .with_config(config)
        .with_mock_args(mock_args)
        .build()
        .await;
    let request_body = axum_core::body::Body::from(
        serde_json::to_vec(&json!({
            "model": "gpt-4o-mini",
            "messages": [
                {
                    "role": "user",
                    "content": "Hello, world!"
                }
            ],
            "max_tokens": 100
        }))
        .unwrap(),
    );
    let request = Request::builder()
        .method(Method::POST)
        // default router
        .uri("http://router.helicone.com/router/v1/messages")
        .body(request_body)
        .unwrap();
    let response = harness.call(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    harness.mock.openai_mock.verify().await;
}
