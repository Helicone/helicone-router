use http::Request;
use tower_governor::{GovernorError, key_extractor::KeyExtractor};

use crate::{
    error::internal::InternalError,
    types::{extensions::AuthContext, router::RouterId, user::UserId},
};

#[derive(Debug, Clone)]
pub struct RateLimitKeyExtractor;

impl KeyExtractor for RateLimitKeyExtractor {
    type Key = UserId;
    fn extract<T>(&self, req: &Request<T>) -> Result<Self::Key, GovernorError> {
        get_user_id(req).map_err(|_| GovernorError::UnableToExtractKey)
    }
}

fn get_user_id<T>(req: &Request<T>) -> Result<UserId, InternalError> {
    let Some(ctx) = req.extensions().get::<AuthContext>() else {
        return Err(InternalError::ExtensionNotFound("AuthContext"));
    };

    Ok(ctx.user_id)
}

pub fn get_redis_rl_key<T>(
    req: &Request<T>,
    router_id: Option<&RouterId>,
) -> Result<String, InternalError> {
    let user_id = get_user_id(req)?;
    if let Some(router_id) = router_id {
        Ok(format!("rl:per-api-key:{router_id}:{user_id}"))
    } else {
        Ok(format!("rl:per-api-key:GLOBAL:{user_id}"))
    }
}
