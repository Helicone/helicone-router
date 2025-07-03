use http::Request;
use tower_governor::{GovernorError, key_extractor::KeyExtractor};

use crate::{
    error::internal::InternalError,
    types::{extensions::AuthContext, user::UserId},
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

pub fn get_redis_rl_key<T>(req: &Request<T>) -> Result<String, InternalError> {
    let user_id = get_user_id(req)?;
    Ok(format!("rl:per-api-key:{user_id}"))
}
