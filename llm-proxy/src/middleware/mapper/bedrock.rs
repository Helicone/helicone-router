use std::{collections::HashMap, str::FromStr};

use async_openai::types::{
    CreateChatCompletionResponse, CreateChatCompletionStreamResponse,
};
use uuid::Uuid;

use super::{
    MapperError, TryConvert, TryConvertStreamData, model::ModelMapper,
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
        bedrock_type::operation::converse::ConverseInput,
    > for BedrockConverter
{
    type Error = MapperError;
    #[allow(clippy::too_many_lines)]
    fn try_convert(
        &self,
        value: async_openai::types::CreateChatCompletionRequest,
    ) -> Result<bedrock_type::operation::converse::ConverseInput, Self::Error>
    {
        let target_provider = InferenceProvider::Bedrock;
        let source_model = ModelId::from_str(&value.model)?;

        let target_model = self
            .model_mapper
            .map_model(&source_model, &target_provider)?;

        tracing::trace!(source_model = ?source_model, target_model = ?target_model, "mapped model");

        let max_tokens = value.max_completion_tokens.unwrap_or(100);
        let stop_sequences = match value.stop {
            Some(async_openai::types::Stop::String(stop)) => Some(vec![stop]),
            Some(async_openai::types::Stop::StringArray(stops)) => Some(stops),
            None => None,
        };
        let temperature = value.temperature;
        let top_p = value.top_p;

        let metadata = value
            .user
            .map(|user| HashMap::from([("user_id".to_string(), user)]));

        let tool_choice = match value.tool_choice {
            Some(
                async_openai::types::ChatCompletionToolChoiceOption::Named(
                    tool,
                ),
            ) => Some(bedrock_type::types::ToolChoice::Tool(
                bedrock_type::types::SpecificToolChoice::builder()
                    .name(tool.function.name)
                    .build()
                    .unwrap(),
            )),
            Some(async_openai::types::ChatCompletionToolChoiceOption::Auto) => {
                Some(bedrock_type::types::ToolChoice::Auto(
                    bedrock_type::types::AutoToolChoice::builder().build(),
                ))
            }
            Some(
                async_openai::types::ChatCompletionToolChoiceOption::Required,
            ) => Some(bedrock_type::types::ToolChoice::Any(
                bedrock_type::types::AnyToolChoice::builder().build(),
            )),
            Some(async_openai::types::ChatCompletionToolChoiceOption::None)
            | None => None,
        };

        let tools = if let Some(tools) = value.tools {
            let mapped_tools = tools.iter().map(|tool| {
                let Some(parameters) = tool.function.parameters.clone() else {
                    return Err(MapperError::ToolMappingInvalid(
                        "Tool parameters are missing".to_string(),
                    ));
                };
                let json_value = match serde_json::from_value(parameters) {
                    Ok(val) => val,
                    Err(e) => {
                        return Err(MapperError::ToolMappingInvalid(format!(
                            "Failed to parse tool parameters, error: {e}",
                        )));
                    }
                };

                let tool_spec =
                    match bedrock_type::types::ToolSpecification::builder()
                        .name(tool.function.name.clone())
                        .set_description(tool.function.description.clone())
                        .input_schema(
                            bedrock_type::types::ToolInputSchema::Json(
                                json_value,
                            ),
                        )
                        .build()
                    {
                        Ok(spec) => spec,
                        Err(e) => {
                            return Err(MapperError::ToolMappingInvalid(
                                format!(
                                    "Failed to build tool specification: {e}",
                                ),
                            ));
                        }
                    };

                Ok(bedrock_type::types::Tool::ToolSpec(tool_spec))
            });
            let mapped_tools: Result<Vec<_>, _> = mapped_tools.collect();
            let mapped_tools = mapped_tools?;
            Some(mapped_tools)
        } else {
            None
        };

        let mut mapped_messages = Vec::with_capacity(value.messages.len());
        for message in value.messages {
            match message {
                async_openai::types::ChatCompletionRequestMessage::Developer(_)
                | async_openai::types::ChatCompletionRequestMessage::System(_) => {}
                async_openai::types::ChatCompletionRequestMessage::User(message) => {
                    let mapped_content: Vec<bedrock_type::types::ContentBlock> = match message.content {
                        async_openai::types::ChatCompletionRequestUserMessageContent::Text(content) => {
                            vec![bedrock_type::types::ContentBlock::Text(content)]
                        }
                        async_openai::types::ChatCompletionRequestUserMessageContent::Array(content) => {
                            content.into_iter().filter_map(|part| {
                                match part {
                                    async_openai::types::ChatCompletionRequestUserMessageContentPart::Text(text) => {
                                        Some(bedrock_type::types::ContentBlock::Text(text.text))
                                    }
                                    async_openai::types::ChatCompletionRequestUserMessageContentPart::ImageUrl(image) => {
                                        let mapped_image = bedrock_type::types::ImageBlock::builder().format(
                                            bedrock_type::types::ImageFormat::Png,
                                        ).source(
                                            bedrock_type::types::ImageSource::Bytes(aws_smithy_types::Blob::new(image.image_url.url))
                                        ).build();

                                        Some(bedrock_type::types::ContentBlock::Image(mapped_image.unwrap()))
                                    }
                                    async_openai::types::ChatCompletionRequestUserMessageContentPart::InputAudio(_audio) => {
                                        // Anthropic does not support audio
                                        None
                                    }
                                }
                            }).collect()
                        }
                    };
                    let mapped_message = bedrock_type::types::Message::builder()
                        .role(bedrock_type::types::ConversationRole::User)
                        .set_content(Some(mapped_content))
                        .build();

                    mapped_messages.push(mapped_message.unwrap());
                }
                async_openai::types::ChatCompletionRequestMessage::Assistant(message) => {
                    let mapped_content = match message.content {
                        Some(async_openai::types::ChatCompletionRequestAssistantMessageContent::Text(content)) => {
                            vec![bedrock_type::types::ContentBlock::Text(content)]
                        }
                        Some(async_openai::types::ChatCompletionRequestAssistantMessageContent::Array(content)) => {
                            content.into_iter().map(|part| {
                                match part {
                                    async_openai::types::ChatCompletionRequestAssistantMessageContentPart::Text(text) => {
                                        bedrock_type::types::ContentBlock::Text(text.text)
                                    }
                                    async_openai::types::ChatCompletionRequestAssistantMessageContentPart::Refusal(text) => {
                                        bedrock_type::types::ContentBlock::Text(text.refusal.clone())
                                    }
                                }
                            }).collect()
                        }
                        None => continue,
                    };
                    let mapped_message = bedrock_type::types::Message::builder()
                        .role(bedrock_type::types::ConversationRole::Assistant)
                        .set_content(Some(mapped_content))
                        .build();
                    mapped_messages.push(mapped_message.unwrap());
                }
                async_openai::types::ChatCompletionRequestMessage::Tool(message) => {
                    let mapped_content = match message.content {
                        async_openai::types::ChatCompletionRequestToolMessageContent::Text(text) => {
                            vec![
                                bedrock_type::types::ContentBlock::ToolResult(
                                    bedrock_type::types::ToolResultBlock::builder().tool_use_id(message.tool_call_id).content(
                                        bedrock_type::types::ToolResultContentBlock::Text(text)
                                    ).build().unwrap()
                                )
                            ]
                        }
                        async_openai::types::ChatCompletionRequestToolMessageContent::Array(content) => {
                            content.into_iter().map(|part| {
                                match part {
                                    async_openai::types::ChatCompletionRequestToolMessageContentPart::Text(text) => {
                                        bedrock_type::types::ContentBlock::ToolResult(
                                            bedrock_type::types::ToolResultBlock::builder()
                                                .tool_use_id(message.tool_call_id.clone())
                                                .content(
                                                    bedrock_type::types::ToolResultContentBlock::Text(text.text)
                                                )
                                                .build().unwrap()
                                        )
                                    }
                                }
                            }).collect()
                        }
                    };

                    let mapped_message = bedrock_type::types::Message::builder()
                        .role(bedrock_type::types::ConversationRole::Assistant)
                        .set_content(Some(mapped_content))
                        .build();
                    mapped_messages.push(mapped_message.unwrap());
                }
                async_openai::types::ChatCompletionRequestMessage::Function(message) => {
                    let tools_ref = tools.as_ref();
                    let Some(tool) = tools_ref.and_then(|tools| {
                        tools.iter().find_map(|tool| {
                            if let bedrock_type::types::Tool::ToolSpec(spec) = tool {
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
                        MapperError::ToolMappingInvalid(message.name.clone())
                    })?;

                    let input = tool_spec
                        .input_schema
                        .as_ref()
                        .and_then(|schema| schema.as_json().ok())
                        .cloned();

                    let mapped_content =
                        vec![bedrock_type::types::ContentBlock::ToolUse(
                            bedrock_type::types::ToolUseBlock::builder()
                                .name(message.name.clone())
                                .tool_use_id(message.name.clone())
                                .set_input(input)
                                .build()
                                .unwrap(),
                        )];

                    let mapped_message = bedrock_type::types::Message::builder()
                        .role(bedrock_type::types::ConversationRole::Assistant)
                        .set_content(Some(mapped_content))
                        .build()
                        .unwrap();
                    mapped_messages.push(mapped_message);
                }
            }
        }

        let mut builder =
            bedrock_type::operation::converse::ConverseInput::builder()
                .model_id(target_model.to_string())
                .set_messages(Some(mapped_messages))
                .set_request_metadata(metadata);

        if let Some(tools) = tools {
            builder = builder.tool_config(
                bedrock_type::types::ToolConfiguration::builder()
                    .set_tool_choice(tool_choice)
                    .set_tools(Some(tools))
                    .build()
                    .unwrap(),
            );
        }

        Ok(builder
            .set_inference_config(Some(
                bedrock_type::types::InferenceConfiguration::builder()
                    .top_p(top_p.unwrap_or_default())
                    .temperature(temperature.unwrap_or_default())
                    .max_tokens(i32::try_from(max_tokens).unwrap_or_default())
                    .set_stop_sequences(stop_sequences)
                    .build(),
            ))
            .build()
            .unwrap())
    }
}

impl
    TryConvert<
        bedrock_type::operation::converse::ConverseOutput,
        CreateChatCompletionResponse,
    > for BedrockConverter
{
    type Error = MapperError;

    #[allow(clippy::too_many_lines)]
    fn try_convert(
        &self,
        value: bedrock_type::operation::converse::ConverseOutput,
    ) -> std::result::Result<CreateChatCompletionResponse, Self::Error> {
        let model = value
            .trace
            .and_then(|t| t.prompt_router)
            .and_then(|r| r.invoked_model_id)
            .unwrap_or_default();

        let created = 0;
        let usage = value.usage.unwrap();

        let usage = async_openai::types::CompletionUsage {
            prompt_tokens: usage.input_tokens.try_into().unwrap_or(0),
            completion_tokens: usage.output_tokens.try_into().unwrap_or(0),
            total_tokens: usage.total_tokens.try_into().unwrap_or(0),
            prompt_tokens_details: Some(
                async_openai::types::PromptTokensDetails {
                    audio_tokens: None,
                    cached_tokens: usage
                        .cache_read_input_tokens
                        .and_then(|i| i.try_into().ok()),
                },
            ),
            completion_tokens_details: None,
        };

        let mut tool_calls: Vec<
            async_openai::types::ChatCompletionMessageToolCall,
        > = Vec::new();
        let mut content = None;
        for bedrock_content in
            value.output.unwrap().as_message().unwrap().content.clone()
        {
            match bedrock_content {
                bedrock_type::types::ContentBlock::ToolUse(tool_use_block) => {
                    tool_calls.push(async_openai::types::ChatCompletionMessageToolCall {
                        id: tool_use_block.tool_use_id.clone(),
                        r#type: async_openai::types::ChatCompletionToolType::Function,
                        function: async_openai::types::FunctionCall {
                            name: tool_use_block.name.clone(),
                            arguments: tool_use_block
                                .input
                                .as_string()
                                .unwrap()
                                .to_string(),
                        },
                    });
                }
                bedrock_type::types::ContentBlock::ToolResult(
                    tool_result_block,
                ) => {
                    tool_calls.push(async_openai::types::ChatCompletionMessageToolCall {
                        id: tool_result_block.tool_use_id.clone(),
                        r#type: async_openai::types::ChatCompletionToolType::Function,
                        function: async_openai::types::FunctionCall {
                            name: tool_result_block.tool_use_id.clone(),
                            arguments: serde_json::to_string(&content)?,
                        },
                    });
                }
                bedrock_type::types::ContentBlock::Text(text) => {
                    content = Some(text.clone());
                }
                bedrock_type::types::ContentBlock::Image(_)
                | bedrock_type::types::ContentBlock::Document(_)
                | bedrock_type::types::ContentBlock::CachePoint(_)
                | bedrock_type::types::ContentBlock::ReasoningContent(_)
                | bedrock_type::types::ContentBlock::GuardContent(_)
                | bedrock_type::types::ContentBlock::Video(_)
                | _ => {}
            }
        }
        let tool_calls = if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        };

        #[allow(deprecated)]
        let message = async_openai::types::ChatCompletionResponseMessage {
            content,
            refusal: None,
            tool_calls,
            role: async_openai::types::Role::Assistant,
            function_call: None,
            audio: None,
        };

        let choice = async_openai::types::ChatChoice {
            index: 0,
            message,
            finish_reason: None,
            logprobs: None,
        };

        let response = async_openai::types::CreateChatCompletionResponse {
            choices: vec![choice],
            id: String::from(Uuid::new_v4()),
            created,
            model,
            object: crate::middleware::mapper::anthropic::OPENAI_CHAT_COMPLETION_OBJECT.to_string(),
            usage: Some(usage),
            service_tier: None,
            system_fingerprint: None,
        };
        Ok(response)
    }
}

impl
    TryConvertStreamData<
        bedrock_type::types::ConverseStreamOutput,
        CreateChatCompletionStreamResponse,
    > for BedrockConverter
{
    type Error = MapperError;

    #[allow(clippy::too_many_lines)]
    fn try_convert_chunk(
        &self,
        value: bedrock_type::types::ConverseStreamOutput,
    ) -> Result<
        std::option::Option<CreateChatCompletionStreamResponse>,
        Self::Error,
    > {
        const CHAT_COMPLETION_CHUNK_OBJECT: &str = "chat.completion.chunk";
        // TODO: These placeholder values for id, model, and created should be
        // replaced by actual values from the MessageStart event,
        // propagated by the stream handling logic.
        const PLACEHOLDER_STREAM_ID: &str = "bedrock-stream-id";
        const PLACEHOLDER_MODEL_NAME: &str = "bedrock-model";
        const DEFAULT_CREATED_TIMESTAMP: u32 = 0;

        #[allow(deprecated)]
        let mut choices = Vec::new();
        let mut completion_usage: async_openai::types::CompletionUsage =
            async_openai::types::CompletionUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            };
        match value {
            bedrock_type::types::ConverseStreamOutput::MessageStart(
                message,
            ) => {
                let choice = async_openai::types::ChatChoiceStream {
                    index: 0,
                    delta: async_openai::types::ChatCompletionStreamResponseDelta {
                        role: Some(match message.role {
                            bedrock_type::types::ConversationRole::Assistant => {
                                async_openai::types::Role::Assistant
                            }
                            bedrock_type::types::ConversationRole::User => async_openai::types::Role::User,
                            _ => async_openai::types::Role::System,
                        }),
                        content: None,
                        tool_calls: None,
                        refusal: None,
                        #[allow(deprecated)]
                        function_call: None,
                    },
                    finish_reason: None,
                    logprobs: None,
                };

                choices.push(choice);
            }
            bedrock_type::types::ConverseStreamOutput::ContentBlockStart(
                content_block_start,
            ) => {
                if let bedrock_type::types::ContentBlockStart::ToolUse(
                    tool_use,
                ) = content_block_start.start.unwrap()
                {
                    let tool_call_chunk =
                        async_openai::types::ChatCompletionMessageToolCallChunk {
                            index: content_block_start
                                .content_block_index
                                .try_into()
                                .unwrap_or(0),
                            id: Some(tool_use.tool_use_id),
                            r#type: Some(
                                async_openai::types::ChatCompletionToolType::Function,
                            ),
                            function: Some(async_openai::types::FunctionCallStream {
                                name: Some(tool_use.name),
                                arguments: Some(String::new()),
                            }),
                        };
                    let choice = async_openai::types::ChatChoiceStream {
                        index: 0,
                        delta: async_openai::types::ChatCompletionStreamResponseDelta {
                            role: None,
                            content: None,
                            tool_calls: Some(vec![tool_call_chunk]),
                            refusal: None,
                            #[allow(deprecated)]
                            function_call: None,
                        },
                        finish_reason: None,
                        logprobs: None,
                    };

                    choices.push(choice);
                }
            }
            bedrock_type::types::ConverseStreamOutput::ContentBlockDelta(
                content_block_delta_event,
            ) => {
                match content_block_delta_event.delta.unwrap() {
                    bedrock_type::types::ContentBlockDelta::Text(text) => {
                        let choice = async_openai::types::ChatChoiceStream {
                            index: u32::try_from(
                                content_block_delta_event.content_block_index,
                            )
                            .unwrap_or(0),
                            delta: async_openai::types::ChatCompletionStreamResponseDelta {
                                role: None,
                                content: Some(text),
                                tool_calls: None,
                                refusal: None,
                                #[allow(deprecated)]
                                function_call: None,
                            },
                            finish_reason: None,
                            logprobs: None,
                        };
                        choices.push(choice);
                    }
                    bedrock_type::types::ContentBlockDelta::ToolUse(tool_use) => {
                        let tool_call_chunk =
                            async_openai::types::ChatCompletionMessageToolCallChunk {
                                index: u32::try_from(
                                    content_block_delta_event
                                        .content_block_index,
                                )
                                .unwrap_or(0),
                                id: None, /* ID would have been sent with ContentBlockStart for this tool */
                                r#type: Some(
                                    async_openai::types::ChatCompletionToolType::Function,
                                ), // Assuming function
                                function: Some(async_openai::types::FunctionCallStream {
                                    name: None, /* Name would have been sent
                                                 * with ContentBlockStart */
                                    arguments: Some(tool_use.input),
                                }),
                            };
                        let choice = async_openai::types::ChatChoiceStream {
                            index: 0,
                            delta: async_openai::types::ChatCompletionStreamResponseDelta {
                                role: None,
                                content: None,
                                tool_calls: Some(vec![tool_call_chunk]),
                                refusal: None,
                                #[allow(deprecated)]
                                function_call: None,
                            },
                            finish_reason: None,
                            logprobs: None,
                        };

                        choices.push(choice);
                    }
                    bedrock_type::types::ContentBlockDelta::ReasoningContent(_) | _ => {}
                }
            }

            bedrock_type::types::ConverseStreamOutput::Metadata(metadata) => {
                if let Some(usage) = metadata.usage {
                    completion_usage.prompt_tokens =
                        u32::try_from(usage.input_tokens).unwrap_or(0);
                    completion_usage.completion_tokens =
                        u32::try_from(usage.output_tokens).unwrap_or(0);
                    completion_usage.total_tokens =
                        u32::try_from(usage.total_tokens).unwrap_or(0);
                }
            }
            bedrock_type::types::ConverseStreamOutput::ContentBlockStop(_)
            | bedrock_type::types::ConverseStreamOutput::MessageStop(_)
            | _ => {}
        }

        Ok(Some(CreateChatCompletionStreamResponse {
            id: PLACEHOLDER_STREAM_ID.to_string(), /* TODO: Use actual
                                                    * stream
                                                    * ID */
            choices,
            created: DEFAULT_CREATED_TIMESTAMP, /* TODO: Use actual
                                                 * created
                                                 * timestamp */
            model: PLACEHOLDER_MODEL_NAME.to_string(), /* TODO: Use actual
                                                        * model name */
            object: CHAT_COMPLETION_CHUNK_OBJECT.to_string(),
            system_fingerprint: None,
            service_tier: None,
            usage: Some(completion_usage),
        }))
    }
}
