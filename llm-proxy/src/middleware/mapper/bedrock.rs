use std::{collections::HashMap, str::FromStr};

use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessageContentPart,
    ChatCompletionToolChoiceOption, CreateChatCompletionResponse,
    CreateChatCompletionStreamResponse,
};
use aws_sdk_bedrockruntime::types::ImageBlock;

use super::{
    TryConvert, TryConvertStreamData, bedrock, error::MapperError,
    model::ModelMapper,
};
use crate::types::{model_id::ModelId, provider::InferenceProvider};

pub struct BedrockConverter {
    model_mapper: ModelMapper,
}

impl BedrockConverter {
    #[must_use]
    pub fn new(model_mapper: ModelMapper) -> Self {
        Self { model_mapper }
    }
}

impl
    TryConvert<
        async_openai::types::CreateChatCompletionRequest,
        aws_sdk_bedrockruntime::operation::converse::ConverseInput,
    > for BedrockConverter
{
    type Error = MapperError;
    fn try_convert(
        &self,
        mut value: async_openai::types::CreateChatCompletionRequest,
    ) -> Result<
        aws_sdk_bedrockruntime::operation::converse::ConverseInput,
        Self::Error,
    > {
        let target_provider = InferenceProvider::Bedrock;
        let source_model = ModelId::from_str(&value.model)?;
        use async_openai::types as openai;
        use aws_sdk_bedrockruntime as bedrock;

        let target_model = self
            .model_mapper
            .map_model(&source_model, &target_provider)?;

        tracing::trace!(source_model = ?source_model, target_model = ?target_model, "mapped model");

        let max_tokens = value
            .max_completion_tokens
            .unwrap_or_else(|| value.max_tokens.unwrap_or(100));
        let stop_sequences = match value.stop {
            Some(openai::Stop::String(stop)) => Some(vec![stop]),
            Some(openai::Stop::StringArray(stops)) => Some(stops),
            None => None,
        };
        let temperature = value.temperature;
        let top_p = value.top_p;

        let metadata = value
            .user
            .map(|user| HashMap::from([("user_id".to_string(), user)]));

        let tool_choice = match value.tool_choice {
            Some(openai::ChatCompletionToolChoiceOption::Named(tool)) => {
                Some(bedrock::types::ToolChoice::Tool(
                    bedrock::types::SpecificToolChoice::builder()
                        .name(tool.function.name)
                        .build()
                        .unwrap(),
                ))
            }
            Some(openai::ChatCompletionToolChoiceOption::Auto) => {
                Some(bedrock::types::ToolChoice::Auto(
                    bedrock::types::AutoToolChoice::builder().build(),
                ))
            }
            Some(openai::ChatCompletionToolChoiceOption::Required) => {
                Some(bedrock::types::ToolChoice::Any(
                    bedrock::types::AnyToolChoice::builder().build(),
                ))
            }
            Some(openai::ChatCompletionToolChoiceOption::None) => None,
            None => None,
        };

        let tools = if let Some(tools) = value.tools {
            let mapped_tools = tools.iter().map(|tool| {
                let json_value: aws_smithy_types::Document = value_to_document(
                    tool.function.parameters.clone().unwrap_or_default(),
                );

                let tool_spec = bedrock::types::ToolSpecification::builder()
                    .name(tool.function.name.clone())
                    .set_description(tool.function.description.clone())
                    .input_schema(bedrock::types::ToolInputSchema::Json(
                        json_value,
                    ))
                    .build()
                    .unwrap();

                bedrock::types::Tool::ToolSpec(tool_spec)
            });
            Some(mapped_tools.collect::<Vec<_>>())
        } else {
            None
        };

        let mut mapped_messages = Vec::with_capacity(value.messages.len());
        let mut found_mapping_error = false;
        for message in value.messages {
            match message {
                openai::ChatCompletionRequestMessage::Developer(_)
                | openai::ChatCompletionRequestMessage::System(_) => {}
                openai::ChatCompletionRequestMessage::User(message) => {
                    let mapped_content: Vec<bedrock::types::ContentBlock> = match message.content {
                        openai::ChatCompletionRequestUserMessageContent::Text(content) => {
                            vec![bedrock::types::ContentBlock::Text(content)]
                        },
                        openai::ChatCompletionRequestUserMessageContent::Array(content) => {
                            content.into_iter().filter_map(|part| {
                                match part {
                                    openai::ChatCompletionRequestUserMessageContentPart::Text(text) => {
                                        Some(bedrock::types::ContentBlock::Text(text.text))
                                    },
                                    openai::ChatCompletionRequestUserMessageContentPart::ImageUrl(image) => {
                                         found_mapping_error =  image.image_url.url.starts_with("http");

                                            let mapped_image = bedrock::types::ImageBlock::builder().format(
                                                bedrock::types::ImageFormat::Png,
                                            ).source(
                                                bedrock::types::ImageSource::Bytes(aws_smithy_types::Blob::new(image.image_url.url))
                                            ).build();

                                        Some(bedrock::types::ContentBlock::Image(mapped_image.unwrap()))
                                    }
                                    openai::ChatCompletionRequestUserMessageContentPart::InputAudio(_audio) => {
                                        // Anthropic does not support audio
                                        None
                                    },
                                }
                            }).collect()
                        }
                    };
                    let mapped_message = bedrock::types::Message::builder()
                        .role(bedrock::types::ConversationRole::User)
                        .set_content(Some(mapped_content))
                        .build();

                    mapped_messages.push(mapped_message.unwrap());
                }
                openai::ChatCompletionRequestMessage::Assistant(message) => {
                    let mapped_content = match message.content {
                        Some(openai::ChatCompletionRequestAssistantMessageContent::Text(content)) => {
                            vec![bedrock::types::ContentBlock::Text(content)]
                        },
                        Some(openai::ChatCompletionRequestAssistantMessageContent::Array(content)) => {
                            content.into_iter().map(|part| {
                                match part {
                                    openai::ChatCompletionRequestAssistantMessageContentPart::Text(text) => {
                                        bedrock::types::ContentBlock::Text(text.text)
                                    },
                                    openai::ChatCompletionRequestAssistantMessageContentPart::Refusal(text) => {
                                        bedrock::types::ContentBlock::Text(text.refusal.clone())
                                    },
                                }
                            }).collect()
                        },
                        None => continue,
                    };
                    let mapped_message = bedrock::types::Message::builder()
                        .role(bedrock::types::ConversationRole::Assistant)
                        .set_content(Some(mapped_content))
                        .build();
                    mapped_messages.push(mapped_message.unwrap());
                }
                openai::ChatCompletionRequestMessage::Tool(message) => {
                    let mapped_content = match message.content {
                        // TODO: Copying from Anthropic but should support more than just text
                        openai::ChatCompletionRequestToolMessageContent::Text(text) => {
                            vec![
                                bedrock::types::ContentBlock::ToolResult(
                                    bedrock::types::ToolResultBlock::builder().tool_use_id(message.tool_call_id).content(
                                        bedrock::types::ToolResultContentBlock::Text(text)
                                    ).build().unwrap()
                                )
                            ]
                        },
                        openai::ChatCompletionRequestToolMessageContent::Array(content) => {
                            content.into_iter().map(|part| {
                                match part {
                                    openai::ChatCompletionRequestToolMessageContentPart::Text(text) => {
                                        bedrock::types::ContentBlock::ToolResult(
                                            bedrock::types::ToolResultBlock::builder()
                                                .tool_use_id(message.tool_call_id.clone())
                                                .content(
                                                    bedrock::types::ToolResultContentBlock::Text(text.text)
                                                )
                                                .build().unwrap()
                                        )
                                    }
                                }
                            }).collect()
                        },
                    };

                    let mapped_message = bedrock::types::Message::builder()
                        .role(bedrock::types::ConversationRole::Assistant)
                        .set_content(Some(mapped_content))
                        .build();
                    mapped_messages.push(mapped_message.unwrap());
                }
                openai::ChatCompletionRequestMessage::Function(message) => {
                    let tools_ref = tools.as_ref();
                    let Some(tool) = tools_ref.and_then(|tools| {
                        tools.iter().find_map(|tool| {
                            if let bedrock::types::Tool::ToolSpec(spec) = tool {
                                if spec.name == message.name {
                                    Some(tool.clone())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                    }) else {
                        continue;
                    };

                    let tool_spec = tool.as_tool_spec().map_err(|_| {
                        MapperError::ProviderNotSupported(
                            "Tool spec not found".to_string(),
                        )
                    })?;

                    let input = tool_spec
                        .input_schema
                        .as_ref()
                        .and_then(|schema| schema.as_json().ok())
                        .cloned();

                    let mapped_content =
                        vec![bedrock::types::ContentBlock::ToolUse(
                            bedrock::types::ToolUseBlock::builder()
                                .name(message.name.clone())
                                .tool_use_id(message.name.clone())
                                .set_input(input)
                                .build()
                                .unwrap(),
                        )];

                    let mapped_message = bedrock::types::Message::builder()
                        .role(bedrock::types::ConversationRole::Assistant)
                        .set_content(Some(mapped_content))
                        .build()
                        .unwrap();
                    mapped_messages.push(mapped_message);
                }
            }
        }

        if found_mapping_error {
            return Err(MapperError::ProviderNotSupported(String::from(
                "Not support Image url",
            )));
        }
        Ok(aws_sdk_bedrockruntime::operation::converse::ConverseInput::builder()
            .model_id(target_model.to_string())
            .set_messages(Some(mapped_messages))
            .set_request_metadata(metadata)
            .tool_config(
                aws_sdk_bedrockruntime::types::ToolConfiguration::builder()
                    .set_tool_choice(tool_choice)
                    .set_tools(tools)
                    .build().unwrap()
            )
            .set_inference_config(
                Some(
                    aws_sdk_bedrockruntime::types::InferenceConfiguration::builder()
                        .top_p(top_p.unwrap())
                        .temperature(temperature.unwrap())
                        .max_tokens(max_tokens as i32)
                        .set_stop_sequences(stop_sequences)
                        .build()
                )
            ).build().unwrap())
    }
}

// TODO(tom): AM i trolling or Document doesnt have a way to deserialize lol
fn value_to_document(value: serde_json::Value) -> aws_smithy_types::Document {
    use aws_smithy_types::{Document, Number};
    match value {
        serde_json::Value::Null => Document::Null,
        serde_json::Value::Bool(b) => Document::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Document::Number(Number::NegInt(i))
            } else if let Some(u) = n.as_u64() {
                Document::Number(Number::PosInt(u))
            } else if let Some(f) = n.as_f64() {
                Document::Number(Number::Float(f))
            } else {
                Document::Null
            }
        }
        serde_json::Value::String(s) => Document::String(s),
        serde_json::Value::Array(arr) => {
            Document::Array(arr.into_iter().map(value_to_document).collect())
        }
        serde_json::Value::Object(map) => Document::Object(
            map.into_iter()
                .map(|(k, v)| (k, value_to_document(v)))
                .collect(),
        ),
    }
}
