use std::time::SystemTime;

use aws_credential_types::Credentials;
use aws_sigv4::{
    http_request::{SignableBody, SignableRequest, SigningSettings},
    sign::v4,
};
use bytes::Bytes;
use futures::StreamExt;
use reqwest::RequestBuilder;
use reqwest_eventsource::{Event, EventSource, RequestBuilderExt};
use tracing::{Instrument, info_span};

use crate::{
    dispatcher::{
        SSEStream, anthropic_client::Client as AnthropicClient,
        bedrock_client::Client as BedrockClient,
        google_gemini_client::Client as GoogleGeminiClient,
        ollama_client::Client as OllamaClient,
        openai_client::Client as OpenAIClient,
    },
    error::{
        init::InitError, internal::InternalError, provider::ProviderError,
    },
    types::provider::InferenceProvider,
};

const AWS_CREDENTIALS_ENV_VAR: &str = "AWS_ACCESS_KEY";
const AWS_CREDENTIALS_SECRET_KEY_ENV_VAR: &str = "AWS_SECRET_KEY";

pub trait ProviderClient {
    #[allow(clippy::needless_return, unused_mut, unused_variables)]
    fn extract_and_sign_aws_headers(
        &self,
        mut request_builder: reqwest::RequestBuilder,
        req_body_bytes: bytes::Bytes,
    ) -> reqwest::RequestBuilder {
        // Default: do nothing, just return the builder
        request_builder
    }

    fn get_aws_credentials(&self) -> Result<(String, String), InitError> {
        let key = std::env::var(AWS_CREDENTIALS_ENV_VAR).map_err(|_| {
            InitError::ProviderError(ProviderError::AwsCredentialsNotFound(
                InferenceProvider::Bedrock,
            ))
        })?;

        let secret = std::env::var(AWS_CREDENTIALS_SECRET_KEY_ENV_VAR)
            .map_err(|_| {
                InitError::ProviderError(ProviderError::AwsCredentialsNotFound(
                    InferenceProvider::Bedrock,
                ))
            })?;

        Ok((key, secret))
    }
}

impl ProviderClient for Client {
    fn extract_and_sign_aws_headers(
        &self,
        request_builder: reqwest::RequestBuilder,
        req_body_bytes: bytes::Bytes,
    ) -> reqwest::RequestBuilder {
        match self {
            Client::Bedrock(inner) => inner
                .extract_and_sign_aws_headers(request_builder, req_body_bytes),
            // ... delegate to other variants as needed ...
            _ => request_builder,
        }
    }
}

impl ProviderClient for BedrockClient {
    fn extract_and_sign_aws_headers(
        &self,
        mut request_builder: reqwest::RequestBuilder,
        req_body_bytes: bytes::Bytes,
    ) -> reqwest::RequestBuilder {
        let (access_key_id, secret) = self
            .get_aws_credentials()
            .expect("cannot get aws credentials");
        let identity =
            Credentials::new(access_key_id, secret, None, None, "Environment")
                .into();

        let request = request_builder.try_clone().unwrap().build().unwrap();
        let host = request.url().host().unwrap().to_string();
        let host_region: Vec<&str> = host.split('.').collect();
        let host_region = host_region.get(1).unwrap();

        let signing_settings = SigningSettings::default();
        let signing_params = v4::SigningParams::builder()
            .identity(&identity)
            .region(host_region)
            .name("bedrock")
            .time(SystemTime::now())
            .settings(signing_settings)
            .build()
            .unwrap()
            .into();

        let mut temp_request = http::Request::builder()
            .uri(request.url().as_str())
            .method(request.method().clone())
            .body(req_body_bytes.clone())
            .expect("cannot build temp request");
        temp_request.headers_mut().extend(request.headers().clone());

        let method_str = temp_request.method().to_string();
        let url_str = temp_request.uri().to_string();

        let signable_request = SignableRequest::new(
            method_str.as_str(),
            url_str.as_str(),
            temp_request
                .headers()
                .iter()
                .map(|(k, v)| (k.as_str(), v.to_str().unwrap())),
            SignableBody::Bytes(req_body_bytes.as_ref()),
        )
        .expect("signable request");

        let (signing_output, _signature) =
            aws_sigv4::http_request::sign(signable_request, &signing_params)
                .expect("cannot sign request")
                .into_parts();
        signing_output.apply_to_request_http1x(&mut temp_request);

        // Copy all the aws signed credentials from temp_request since the
        // apply_to_request_http1x is only for http::Request types
        for (key, value) in temp_request.headers() {
            if !request_builder
                .try_clone()
                .unwrap()
                .build()
                .unwrap()
                .headers()
                .contains_key(key)
            {
                tracing::info!(
                    "set aws signature headers key: {:?}, value: {:?}",
                    key,
                    value
                );
                request_builder = request_builder.header(key, value);
            }
        }

        request_builder
    }
}

#[derive(Debug, Clone)]
pub enum Client {
    OpenAI(OpenAIClient),
    Anthropic(AnthropicClient),
    GoogleGemini(GoogleGeminiClient),
    Ollama(OllamaClient),
    Bedrock(BedrockClient),
}

impl Client {
    pub(crate) fn sse_stream<B>(
        request_builder: RequestBuilder,
        body: B,
    ) -> Result<SSEStream, InternalError>
    where
        B: Into<reqwest::Body>,
    {
        let event_source = request_builder
            .body(body)
            .eventsource()
            .map_err(|e| InternalError::RequestBodyError(Box::new(e)))?;
        Ok(sse_stream(event_source))
    }
}

impl AsRef<reqwest::Client> for Client {
    fn as_ref(&self) -> &reqwest::Client {
        match self {
            Client::OpenAI(client) => &client.0,
            Client::Anthropic(client) => &client.0,
            Client::GoogleGemini(client) => &client.0,
            Client::Ollama(client) => &client.0,
            Client::Bedrock(client) => &client.0,
        }
    }
}

/// Request which responds with SSE.
/// [server-sent events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events/Using_server-sent_events#event_stream_format)
pub(super) fn sse_stream(mut event_source: EventSource) -> SSEStream {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(
        async move {
            while let Some(ev) = event_source.next().await {
                match ev {
                    Err(e) => {
                        if let Err(_e) = tx
                            .send(Err(InternalError::StreamError(Box::new(e))))
                        {
                            // rx dropped
                            break;
                        }
                    }
                    Ok(event) => match event {
                        Event::Message(message) => {
                            if message.data == "[DONE]" {
                                break;
                            }

                            let data = Bytes::from(message.data);

                            if let Err(_e) = tx.send(Ok(data)) {
                                // rx dropped
                                break;
                            }
                        }
                        Event::Open => {}
                    },
                }
            }

            event_source.close();
        }
        .instrument(info_span!("sse_stream")),
    );

    Box::pin(tokio_stream::wrappers::UnboundedReceiverStream::new(rx))
}
