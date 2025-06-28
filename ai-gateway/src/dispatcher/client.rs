use bytes::Bytes;
use futures::StreamExt;
use http_body_util::BodyExt;
use reqwest::RequestBuilder;
use reqwest_eventsource::{Event, EventSource, RequestBuilderExt};
use tracing::{Instrument, info_span};

use crate::{
    app_state::AppState,
    dispatcher::{
        SSEStream, anthropic_client::Client as AnthropicClient,
        bedrock_client::Client as BedrockClient,
        google_gemini_client::Client as GoogleGeminiClient,
        ollama_client::Client as OllamaClient,
        openai_client::Client as OpenAIClient,
    },
    error::{api::ApiError, init::InitError, internal::InternalError},
    types::{
        provider::{InferenceProvider, ProviderKey},
        router::RouterId,
    },
};

pub trait ProviderClient {
    fn extract_and_sign_aws_headers(
        &self,
        request_builder: reqwest::RequestBuilder,
        req_body_bytes: &bytes::Bytes,
    ) -> Result<reqwest::RequestBuilder, ApiError>;
}

impl ProviderClient for Client {
    fn extract_and_sign_aws_headers(
        &self,
        request_builder: reqwest::RequestBuilder,
        req_body_bytes: &bytes::Bytes,
    ) -> Result<reqwest::RequestBuilder, ApiError> {
        match self {
            Client::Bedrock(inner) => inner
                .extract_and_sign_aws_headers(request_builder, req_body_bytes),
            _ => Ok(request_builder),
        }
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

    fn new_inner(
        app_state: &AppState,
        inference_provider: InferenceProvider,
        api_key: Option<&ProviderKey>,
    ) -> Result<Self, InitError> {
        // connection timeout, timeout, etc.
        let base_client = reqwest::Client::builder()
            .connect_timeout(app_state.0.config.dispatcher.connection_timeout)
            .timeout(app_state.0.config.dispatcher.timeout)
            .tcp_nodelay(true);

        match inference_provider {
            InferenceProvider::OpenAI => Ok(Self::OpenAI(OpenAIClient::new(
                app_state,
                base_client,
                api_key,
            )?)),
            InferenceProvider::Anthropic => Ok(Self::Anthropic(
                AnthropicClient::new(app_state, base_client, api_key)?,
            )),
            InferenceProvider::GoogleGemini => Ok(Self::GoogleGemini(
                GoogleGeminiClient::new(app_state, base_client, api_key)?,
            )),
            InferenceProvider::Bedrock => Ok(Self::Bedrock(
                BedrockClient::new(app_state, base_client, api_key)?,
            )),
            InferenceProvider::Ollama => {
                Ok(Self::Ollama(OllamaClient::new(app_state, base_client)?))
            }
        }
    }

    pub(crate) async fn new_for_router(
        app_state: &AppState,
        inference_provider: InferenceProvider,
        router_id: &RouterId,
    ) -> Result<Self, InitError> {
        if inference_provider == InferenceProvider::Ollama {
            return Self::new_inner(app_state, inference_provider, None);
        }
        let api_key = &app_state
            .get_provider_api_key_for_router(router_id, inference_provider)
            .await?;

        Self::new_inner(app_state, inference_provider, api_key.as_ref())
    }

    pub(crate) fn new_for_direct_proxy(
        app_state: &AppState,
        inference_provider: InferenceProvider,
    ) -> Result<Self, InitError> {
        if inference_provider == InferenceProvider::Ollama {
            return Self::new_inner(app_state, inference_provider, None);
        }
        let api_key = &app_state
            .get_provider_api_key_for_direct_proxy(inference_provider)?;

        Self::new_inner(app_state, inference_provider, api_key.as_ref())
    }

    pub(crate) fn new_for_unified_api(
        app_state: &AppState,
        inference_provider: InferenceProvider,
    ) -> Result<Self, InitError> {
        if inference_provider == InferenceProvider::Ollama {
            return Self::new_inner(app_state, inference_provider, None);
        }
        // we're cheating here but this will be changed soon for cloud hosted
        // version
        let api_key = &app_state
            .get_provider_api_key_for_direct_proxy(inference_provider)?;

        Self::new_inner(app_state, inference_provider, api_key.as_ref())
    }
}

impl AsRef<reqwest::Client> for Client {
    fn as_ref(&self) -> &reqwest::Client {
        match self {
            Client::OpenAI(client) => &client.0,
            Client::Anthropic(client) => &client.0,
            Client::GoogleGemini(client) => &client.0,
            Client::Ollama(client) => &client.0,
            Client::Bedrock(client) => &client.inner,
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
                        if matches!(e, reqwest_eventsource::Error::StreamEnded) {
                            // `StreamEnded` is returned for valid stream end cases
                            // so we don't send the error in the channel
                            tracing::trace!("stream ended");
                            break;
                        }

                        cfg_if::cfg_if! {
                            if #[cfg(debug_assertions)] {
                                if let Err(e) = debug_stream_error(tx.clone(), e).await {
                                    tracing::error!(error = %e, "rx dropped before stream ended");
                                    break;
                                }
                            } else {
                                if let Err(e) = tx.send(Err(InternalError::StreamError(Box::new(e)))) {
                                    tracing::error!(error = %e, "rx dropped before stream ended");
                                    break;
                                }
                            }
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
                                tracing::trace!(
                                    "rx dropped before stream ended"
                                );
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

async fn debug_stream_error(
    tx: tokio::sync::mpsc::UnboundedSender<Result<Bytes, InternalError>>,
    error: reqwest_eventsource::Error,
) -> Result<(), tokio::sync::mpsc::error::SendError<Result<Bytes, InternalError>>>
{
    match error {
        reqwest_eventsource::Error::InvalidStatusCode(
            status_code,
            response,
        ) => {
            let http_resp = http::Response::from(response);
            let (parts, body) = http_resp.into_parts();
            let Ok(body) = body.collect().await else {
                tracing::error!("failed to collect body in stream");
                // silence the error
                return Ok(());
            };
            let body = body.to_bytes();
            let text = String::from_utf8_lossy(&body);
            tracing::debug!(status_code = %status_code, body = %text, "received error response in stream");
            let stream = futures::stream::once(futures::future::ok::<
                _,
                InternalError,
            >(body));
            let new_body = reqwest::Body::wrap_stream(stream);
            let new_response = http::Response::from_parts(parts, new_body);
            let new_response = reqwest::Response::from(new_response);

            let e = reqwest_eventsource::Error::InvalidStatusCode(
                status_code,
                new_response,
            );
            tx.send(Err(InternalError::StreamError(Box::new(e))))?;
            Ok(())
        }
        e => {
            // propagate other errors
            tx.send(Err(InternalError::StreamError(Box::new(e))))?;
            Ok(())
        }
    }
}
