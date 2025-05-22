use std::{str::FromStr, sync::Arc};

use futures::future::BoxFuture;
use http::{Request, StatusCode};
use serde::Deserialize;
use tower_http::auth::AsyncAuthorizeRequest;
use tracing::warn;
use uuid::Uuid;

use crate::{
    app::AppState,
    config::DeploymentTarget,
    error::auth::AuthError,
    types::{org::OrgId, request::AuthContext, user::UserId},
};

#[derive(Clone)]
pub struct AuthService {
    app_state: Arc<AppState>,
}

impl AuthService {
    #[must_use]
    pub fn new(app_state: AppState) -> Self {
        Self {
            app_state: Arc::new(app_state),
        }
    }

    // Private helper function for authentication
    async fn authenticate_request_inner(
        app_state: &AppState,
        api_key: String,
    ) -> Result<AuthContext, AuthError> {
        let whoami_url = app_state
            .0
            .config
            .helicone
            .base_url
            .join("/v1/router/control-plane/whoami")
            .map_err(|_| AuthError::InvalidCredentials)?;

        let whoami_result = app_state
            .0
            .jawn_client
            .get(whoami_url)
            .header("authorization", api_key.clone())
            .send()
            .await?;

        let body = whoami_result.json::<WhoamiResponse>().await?;

        let org_id = Uuid::from_str(&body.organization_id)
            .map_err(|_| AuthError::InvalidCredentials)?;
        let user_id = Uuid::from_str(&body.user_id)
            .map_err(|_| AuthError::InvalidCredentials)?;

        Ok(AuthContext {
            api_key: api_key.replace("Bearer ", ""),
            user_id: UserId::new(user_id),
            org_id: OrgId::new(org_id),
        })
    }
}

#[derive(Debug, Deserialize)]
struct WhoamiResponse {
    #[serde(rename = "userId")]
    user_id: String,
    #[serde(rename = "organizationId")]
    organization_id: String,
}

// Specific implementation for axum_core::body::Body
impl AsyncAuthorizeRequest<axum_core::body::Body> for AuthService {
    type RequestBody = axum_core::body::Body;
    type ResponseBody = axum_core::body::Body;
    type Future = BoxFuture<
        'static,
        Result<
            Request<axum_core::body::Body>,
            http::Response<Self::ResponseBody>,
        >,
    >;

    #[tracing::instrument(skip_all)]
    fn authorize(
        &mut self,
        mut request: Request<axum_core::body::Body>,
    ) -> Self::Future {
        // NOTE:
        // this is a temporary solution, when we get the control plane up and
        // running, we will actively be pushing the config to the router
        // rather than fetching it from the control plane each time

        tracing::trace!("Auth middleware for axum body");

        let api_key: Option<String> =
            match self.app_state.0.config.deployment_target {
                DeploymentTarget::Cloud { .. } => {
                    panic!("Cloud deployment target not implemented")
                }
                DeploymentTarget::Sidecar {
                    use_global_helicone_key,
                }
                | DeploymentTarget::SelfHosted {
                    use_global_helicone_key,
                } => {
                    if use_global_helicone_key {
                        std::env::var("HELICONE_API_KEY").ok()
                    } else {
                        request
                            .headers()
                            .get("authorization")
                            .and_then(|h| h.to_str().ok())
                            .map(String::from)
                    }
                }
            };
        println!("api_key: {:?}", api_key);

        // Just clone the Arc, which is much cheaper
        let app_state = self.app_state.clone();

        Box::pin(async move {
            if let Some(api_key) = api_key {
                match Self::authenticate_request_inner(&app_state, api_key)
                    .await
                {
                    Ok(auth_ctx) => {
                        request.extensions_mut().insert(Some(auth_ctx));
                        Ok(request)
                    }
                    Err(e) => {
                        warn!("Authentication error: {:?}", e);
                        Err(http::Response::builder()
                            .status(StatusCode::UNAUTHORIZED)
                            .body(axum_core::body::Body::empty())
                            .unwrap_or_else(|_| {
                                panic!("Failed to build response")
                            }))
                    }
                }
            } else {
                // @Tom - do i need to do this? This extensions type hashmap is
                // like magic to me, i have no idea what is happening
                request.extensions_mut().insert::<Option<AuthContext>>(None);
                Ok(request)
            }
        })
    }
}
