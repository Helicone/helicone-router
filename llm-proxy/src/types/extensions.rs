use derive_more::{AsRef, From, Into};

#[derive(Debug, Clone, AsRef, From, Into)]
pub struct ProviderRequestId(pub(crate) http::HeaderValue);
