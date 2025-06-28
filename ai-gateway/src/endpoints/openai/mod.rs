pub mod chat_completions;

use super::EndpointType;
pub use crate::endpoints::openai::chat_completions::ChatCompletions;
use crate::{
    endpoints::{Endpoint, EndpointRoute},
    error::invalid_req::InvalidRequestError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, strum::EnumIter)]
pub enum OpenAI {
    ChatCompletions(ChatCompletions),
}

impl OpenAI {
    #[must_use]
    pub fn path(&self) -> &str {
        match self {
            Self::ChatCompletions(_) => ChatCompletions::PATH,
        }
    }

    #[must_use]
    pub fn chat_completions() -> Self {
        Self::ChatCompletions(ChatCompletions)
    }

    #[must_use]
    pub fn endpoint_type(&self) -> EndpointType {
        match self {
            Self::ChatCompletions(_) => EndpointType::Chat,
        }
    }
}

impl TryFrom<&EndpointRoute> for OpenAI {
    type Error = InvalidRequestError;

    fn try_from(endpoint: &EndpointRoute) -> Result<Self, Self::Error> {
        match endpoint {
            EndpointRoute::ChatCompletions => {
                Ok(Self::ChatCompletions(ChatCompletions))
            }
        }
    }
}
