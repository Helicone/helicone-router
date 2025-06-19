use std::{
    future::{Future, Ready, ready},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use http::uri::PathAndQuery;
use rustc_hash::FxHashMap as HashMap;
use tower::ServiceBuilder;

use crate::{
    app_state::AppState,
    balancer::provider::ProviderBalancer,
    config::{DeploymentTarget, SDK},
    dispatcher::Dispatcher,
    endpoints::{ApiEndpoint, EndpointType},
    error::{
        api::ApiError, init::InitError, internal::InternalError,
        invalid_req::InvalidRequestError,
    },
    middleware::{rate_limit, request_context},
    router::direct::DirectProxyService,
    types::router::RouterId,
};

pub type RouterService =
    rate_limit::Service<request_context::Service<ProviderBalancer>>;

#[derive(Debug)]
pub struct Router {
    inner: HashMap<EndpointType, RouterService>,
    direct_proxy: DirectProxyService,
}

impl Router {
    pub async fn new(
        id: RouterId,
        app_state: AppState,
    ) -> Result<Self, InitError> {
        let router_config = match &app_state.0.config.deployment_target {
            DeploymentTarget::Cloud => {
                return Err(InitError::DeploymentTargetNotSupported(
                    app_state.0.config.deployment_target.clone(),
                ));
            }
            DeploymentTarget::SelfHosted | DeploymentTarget::Sidecar => {
                let router_config = app_state
                    .0
                    .config
                    .routers
                    .as_ref()
                    .get(&id)
                    .ok_or(InitError::DefaultRouterNotFound)?
                    .clone();
                Arc::new(router_config)
            }
        };
        router_config.validate()?;

        let provider_keys = app_state
            .add_provider_keys_for_router(id.clone(), &router_config)
            .await?;

        let mut inner = HashMap::default();
        let rl_layer = rate_limit::Layer::per_router(
            &app_state,
            id.clone(),
            &router_config,
        )
        .await?;
        for (endpoint_type, balance_config) in
            router_config.load_balance.as_ref()
        {
            let balancer = ProviderBalancer::new(
                app_state.clone(),
                id.clone(),
                router_config.clone(),
                balance_config,
            )
            .await?;
            let service_stack: RouterService = ServiceBuilder::new()
                .layer(rl_layer.clone())
                .layer(request_context::Layer::for_router(
                    router_config.clone(),
                    provider_keys.clone(),
                ))
                // other middleware: caching, etc, etc
                // will be added here as well from the router config
                // .map_err(|e| crate::error::api::Error::Box(e))
                .service(balancer);

            inner.insert(*endpoint_type, service_stack);
        }
        let direct_proxy_dispatcher =
            Dispatcher::new(app_state.clone(), &id, &router_config, SDK)
                .await?;

        let direct_proxy = ServiceBuilder::new()
            .layer(rl_layer)
            .layer(request_context::Layer::for_router(
                router_config.clone(),
                provider_keys.clone(),
            ))
            // other middleware: caching, etc, etc
            // will be added here as well from the router config
            // .map_err(|e| crate::error::api::Error::Box(e))
            .service(direct_proxy_dispatcher);

        tracing::info!(id = %id, "router created");

        Ok(Self {
            inner,
            direct_proxy,
        })
    }
}

impl tower::Service<crate::types::request::Request> for Router {
    type Response = crate::types::response::Response;
    type Error = ApiError;
    type Future = RouterFuture;

    #[inline]
    fn poll_ready(
        &mut self,
        ctx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let mut any_pending = false;
        for balancer in self.inner.values_mut() {
            if balancer.poll_ready(ctx).is_pending() {
                any_pending = true;
            }
        }
        if self.direct_proxy.poll_ready(ctx).is_pending() {
            any_pending = true;
        }
        if any_pending {
            Poll::Pending
        } else {
            Poll::Ready(Ok(()))
        }
    }

    #[inline]
    #[tracing::instrument(level = "debug", name = "router", skip_all)]
    fn call(
        &mut self,
        mut req: crate::types::request::Request,
    ) -> Self::Future {
        let Some(extracted_path_and_query) =
            req.extensions().get::<PathAndQuery>()
        else {
            return RouterFuture::Ready(ready(Err(
                InternalError::ExtensionNotFound("PathAndQuery").into(),
            )));
        };

        let api_endpoint =
            ApiEndpoint::new(extracted_path_and_query.path(), SDK);
        match api_endpoint {
            Some(api_endpoint) => {
                let endpoint_type = api_endpoint.endpoint_type();
                if let Some(balancer) = self.inner.get_mut(&endpoint_type) {
                    req.extensions_mut().insert(api_endpoint);
                    RouterFuture::Balancer(balancer.call(req))
                } else {
                    RouterFuture::Ready(ready(Err(
                        InvalidRequestError::NotFound(
                            extracted_path_and_query.path().to_string(),
                        )
                        .into(),
                    )))
                }
            }
            None => RouterFuture::DirectProxy(self.direct_proxy.call(req)),
        }
    }
}

pub enum RouterFuture {
    /// Ready with an immediate response
    Ready(Ready<Result<crate::types::response::Response, ApiError>>),
    /// Calling the `ProviderBalancer`
    Balancer(<RouterService as tower::Service<crate::types::request::Request>>::Future),
    /// Calling the direct proxy
    DirectProxy(<DirectProxyService as tower::Service<crate::types::request::Request>>::Future),
}

impl Future for RouterFuture {
    type Output = Result<crate::types::response::Response, ApiError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.get_mut() {
            RouterFuture::Ready(ready) => Pin::new(ready).poll(cx),
            RouterFuture::Balancer(fut) => Pin::new(fut).poll(cx),
            RouterFuture::DirectProxy(fut) => {
                match Pin::new(fut).poll(cx) {
                    Poll::Ready(Ok(response)) => Poll::Ready(Ok(response)),
                    Poll::Ready(Err(infallible)) => {
                        // This match confirms the error is truly infallible
                        match infallible {}
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}
