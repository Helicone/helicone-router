
use aws_sdk_bedrockruntime::operation::converse::{ConverseInput, ConverseOutput};
use aws_sdk_bedrockruntime::operation::converse_stream::ConverseStreamInput;

use crate::{
    endpoints::{AiRequest, Endpoint},
    middleware::mapper::error::MapperError,
    types::{model_id::ModelId, provider::InferenceProvider},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Converse;

impl Endpoint for Converse {
    const PATH: &'static str = "/v1/messages";
    type RequestBody = ConverseInput;
    type ResponseBody = ConverseOutput;
    type StreamResponseBody = ConverseStreamInput;
}

// impl AiRequest for ConverseInput {
//     fn is_stream(&self) -> bool {
//         self.stream.unwrap_or(false)
//     }
//
//     fn model(&self) -> Result<ModelId, MapperError> {
//         ModelId::from_str_and_provider(
//             InferenceProvider::Anthropic,
//             &self.model,
//         )
//     }
// }
