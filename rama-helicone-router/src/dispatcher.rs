use std::{convert::Infallible, str::FromStr, sync::Arc};

use rama::{
    Layer, Service,
    http::{
        Body, HeaderName, HeaderValue, Request, Response,
        client::{EasyHttpWebClient, TlsConnectorConfig},
        layer::{
            auth::AsyncRequireAuthorizationLayer,
            compress_adapter::CompressAdaptLayer,
            map_response_body::MapResponseBodyLayer,
            remove_header::{
                RemoveRequestHeaderLayer, RemoveResponseHeaderLayer,
            },
            required_header::AddRequiredRequestHeadersLayer,
        },
    },
    layer::ConsumeErrLayer,
    tls::rustls::client::TlsConnectorData,
};

use crate::{
    app::{AppState, Context},
    error::{api::Error, internal::InternalError},
    middleware::auth::AuthService,
    types::{provider::Provider, request::RequestContext},
};

pub trait AiProviderDispatcher:
    Service<AppState, Request, Response = Response, Error = Error>
    + Clone
    + Send
    + Sync
    + 'static
{
    fn provider(&self) -> Provider;
}

#[derive(Debug, Clone)]
pub struct Dispatcher {
    client: EasyHttpWebClient,
    provider: Provider,
}

impl AiProviderDispatcher for Dispatcher {
    fn provider(&self) -> Provider {
        self.provider
    }
}

impl Dispatcher {
    pub fn new(_ctx: Context, provider: Provider) -> Self {
        let tls = TlsConnectorData::new_http_auto().unwrap();
        let client = EasyHttpWebClient::default()
            .with_tls_connector_config(TlsConnectorConfig::Rustls(Some(tls)));
        Self { client, provider }
    }

    pub fn new_with_middleware(
        ctx: Context,
        provider: Provider,
    ) -> impl rama::Service<AppState, Request, Response = Response, Error = Infallible>
    {
        let dispatcher = Dispatcher::new(ctx, provider);
        (
            MapResponseBodyLayer::new(Body::new),
            ConsumeErrLayer::default(),
            RemoveResponseHeaderLayer::hop_by_hop(),
            RemoveRequestHeaderLayer::hop_by_hop(),
            CompressAdaptLayer::default(),
            AddRequiredRequestHeadersLayer::new(),
            AsyncRequireAuthorizationLayer::new(AuthService),
            crate::middleware::request_context::Layer,
        )
            .into_layer(dispatcher)
    }
}

impl Service<AppState, Request> for Dispatcher {
    type Response = Response;
    type Error = Error;

    #[tracing::instrument(skip_all)]
    fn serve(
        &self,
        ctx: Context,
        req: Request,
    ) -> impl futures::Future<Output = Result<Self::Response, Self::Error>> + Send
    {
        tracing::info!("Dispatcher::serve");
        let this = self.clone();
        tracing::info!(uri = %req.uri(), headers = ?req.headers(), "Received request");
        async move { this.dispatch(ctx, req).await }
    }
}

impl Dispatcher {
    async fn dispatch(
        &self,
        ctx: Context,
        mut req: Request,
    ) -> Result<Response, Error> {
        let req_ctx = ctx
            .get::<Arc<RequestContext>>()
            .ok_or(InternalError::ExtensionNotFound("RequestContext"))?;
        let target_provider = req_ctx.proxy_context.target_provider.clone();
        let target_base_url = ctx
            .state()
            .0
            .config
            .dispatcher
            .get_provider_url(target_provider)?
            .clone();
        let provider_api_key = req_ctx
            .proxy_context
            .provider_api_keys
            .as_ref()
            .get(&target_provider)
            .unwrap()
            .clone();
        // Get the parts after the router ID
        let remaining_path = req
            .uri()
            .path()
            .split('/')
            .skip(3)
            .collect::<Vec<&str>>()
            .join("/");
        let target_url = target_base_url.join(remaining_path.as_str()).unwrap();
        {
            let r = req.headers_mut();
            r.remove(http::header::HOST);
            let host_header = match target_url.host() {
                Some(url::Host::Domain(host)) => {
                    HeaderValue::from_str(host).unwrap()
                }
                None | _ => HeaderValue::from_str("").unwrap(),
            };
            r.insert(http::header::HOST, host_header);
            r.remove(http::header::AUTHORIZATION);
            r.remove(http::header::CONTENT_LENGTH);
            r.remove(HeaderName::from_str("helicone-api-key").unwrap());
            match target_provider {
                Provider::OpenAI => {
                    let openai_auth_header =
                        format!("Bearer {}", provider_api_key.0);
                    r.insert(
                        http::header::AUTHORIZATION,
                        HeaderValue::from_str(&openai_auth_header).unwrap(),
                    );
                }
                Provider::Anthropic => {
                    r.insert(
                        HeaderName::from_str("x-api-key").unwrap(),
                        HeaderValue::from_str(&provider_api_key).unwrap(),
                    );
                    r.insert(
                        HeaderName::from_str("anthropic-version").unwrap(),
                        HeaderValue::from_str("2023-06-01").unwrap(),
                    );
                }
            }
        }

        let target_uri = http::Uri::from_str(target_url.as_str()).unwrap();
        *req.uri_mut() = target_uri;
        tracing::info!(uri = %req.uri(), "Request to target provider");
        let response = self
            .client
            .serve(ctx, req)
            .await
            .map_err(InternalError::RequestClientError)?;
        Ok(response)
    }
}
