use std::str::FromStr;

use futures::future::BoxFuture;
use http::{HeaderValue, Request, StatusCode};
use serde::Deserialize;
use tower_http::auth::AsyncAuthorizeRequest;
use tracing::warn;
use uuid::Uuid;

use crate::{
    app::AppState,
    types::{org::OrgId, request::AuthContext, user::UserId},
};

#[derive(Clone)]
pub struct AuthService {
    app_state: AppState,
}

impl AuthService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }
}

#[derive(Debug, Deserialize)]
struct WhoamiResponse {
    userId: String,
    organizationId: String,
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

    fn authorize(
        &mut self,
        mut request: Request<axum_core::body::Body>,
    ) -> Self::Future {
        tracing::trace!("Auth middleware for axum body");
        let api_key = request
            .headers()
            .get("authorization")
            .unwrap_or(&HeaderValue::from_static(""))
            .to_str()
            .unwrap_or_default()
            .to_string();
        let app_state = self.app_state.clone();
        let whoami_url = self
            .app_state
            .0
            .config
            .helicone
            .base_url
            .join("/v1/router/control-plane/whoami")
            .unwrap();

        // For development, always authenticate without requiring AppState
        Box::pin(async move {
            // Try to make the request, but don't rely on the response

            let whoami_result = app_state
                .0
                .jawn_client
                .get(whoami_url)
                .header("authorization", api_key.clone())
                .send()
                .await;

            if let Ok(response) = whoami_result {
                if let Ok(body) = response.json::<WhoamiResponse>().await {
                    println!("body: {:?}", body);
                    let org_id = Uuid::from_str(&body.organizationId).unwrap();
                    let user_id = Uuid::from_str(&body.userId).unwrap();
                    let auth_ctx = AuthContext {
                        api_key: api_key.replace("Bearer ", ""),
                        user_id: UserId::new(user_id),
                        org_id: OrgId::new(org_id),
                    };
                    request.extensions_mut().insert(auth_ctx);
                    return Ok(request);
                }
            } else if let Err(e) = whoami_result {
                warn!("Error making whoami request: {:?}", e);
            }

            warn!(
                "Using hardcoded auth values - this should only happen in \
                 development"
            );
            let org_id =
                Uuid::from_str("545d62a5-5efc-4260-ac21-ded2d5b95f71").unwrap();
            let user_id =
                Uuid::from_str("35cc8ca0-c655-4bf7-a089-0c8a68e81dc5").unwrap();
            let api_key = std::env::var("HELICONE_API_KEY")
                .unwrap_or_else(|_| "mock-api-key".to_string());

            let auth_ctx = AuthContext {
                api_key,
                user_id: UserId::new(user_id),
                org_id: OrgId::new(org_id),
            };

            // Set `auth_ctx` as a request extension
            request.extensions_mut().insert(auth_ctx);
            Ok(request)
        })
    }
}
