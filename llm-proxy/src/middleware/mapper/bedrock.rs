use std::{collections::HashMap, str::FromStr};

use async_openai::types::{
    CreateChatCompletionResponse, CreateChatCompletionStreamResponse,
};
use aws_sdk_bedrockruntime::types::{
    ContentBlock, ContentBlockDelta, ContentBlockStart, ConversationRole,
};
use uuid::Uuid;

use super::{
    TryConvert, TryConvertStreamData, error::MapperError, model::ModelMapper,
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
    #[allow(clippy::too_many_lines)]
    fn try_convert(
        &self,
        value: async_openai::types::CreateChatCompletionRequest,
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

        let max_tokens = value.max_completion_tokens.unwrap_or(100);
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
            Some(openai::ChatCompletionToolChoiceOption::None) | None => None,
        };

        let tools = if let Some(tools) = value.tools {
            let mapped_tools = tools.iter().map(|tool| {
                let parameters = match tool.function.parameters.clone() {
                    Some(params) => params,
                    None => {
                        return Err(MapperError::ToolMappingInvalid(
                            "Tool parameters are missing".to_string(),
                        ));
                    }
                };
                let json_value = match serde_json::from_value(parameters) {
                    Ok(val) => val,
                    Err(e) => {
                        return Err(MapperError::ToolMappingInvalid(format!(
                            "Failed to parse tool parameters: {}",
                            e
                        )));
                    }
                };

                let tool_spec =
                    match bedrock::types::ToolSpecification::builder()
                        .name(tool.function.name.clone())
                        .set_description(tool.function.description.clone())
                        .input_schema(bedrock::types::ToolInputSchema::Json(
                            json_value,
                        ))
                        .build()
                    {
                        Ok(spec) => spec,
                        Err(e) => {
                            return Err(MapperError::ToolMappingInvalid(
                                format!(
                                    "Failed to build tool specification: {}",
                                    e
                                ),
                            ));
                        }
                    };

                Ok(bedrock::types::Tool::ToolSpec(tool_spec))
            });
            let mapped_tools: Result<Vec<_>, _> = mapped_tools.collect();
            let mapped_tools = mapped_tools?;
            Some(mapped_tools)
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
                        }
                        openai::ChatCompletionRequestUserMessageContent::Array(content) => {
                            content.into_iter().filter_map(|part| {
                                match part {
                                    openai::ChatCompletionRequestUserMessageContentPart::Text(text) => {
                                        Some(bedrock::types::ContentBlock::Text(text.text))
                                    }
                                    openai::ChatCompletionRequestUserMessageContentPart::ImageUrl(image) => {
                                        found_mapping_error = image.image_url.url.starts_with("http");

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
                                    }
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
                        }
                        Some(openai::ChatCompletionRequestAssistantMessageContent::Array(content)) => {
                            content.into_iter().map(|part| {
                                match part {
                                    openai::ChatCompletionRequestAssistantMessageContentPart::Text(text) => {
                                        bedrock::types::ContentBlock::Text(text.text)
                                    }
                                    openai::ChatCompletionRequestAssistantMessageContentPart::Refusal(text) => {
                                        bedrock::types::ContentBlock::Text(text.refusal.clone())
                                    }
                                }
                            }).collect()
                        }
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
                        openai::ChatCompletionRequestToolMessageContent::Text(text) => {
                            vec![
                                bedrock::types::ContentBlock::ToolResult(
                                    bedrock::types::ToolResultBlock::builder().tool_use_id(message.tool_call_id).content(
                                        bedrock::types::ToolResultContentBlock::Text(text)
                                    ).build().unwrap()
                                )
                            ]
                        }
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
                        }
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
                        MapperError::ToolMappingInvalid(message.name.clone())
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
            return Err(MapperError::ImageUrlNotSupported(String::from(
                "Not support Image url",
            )));
        }

        let mut builder = aws_sdk_bedrockruntime::operation::converse::ConverseInput::builder()
            .model_id(target_model.to_string())
            .set_messages(Some(mapped_messages))
            .set_request_metadata(metadata);

        if let Some(tools) = tools {
            builder = builder.tool_config(
                aws_sdk_bedrockruntime::types::ToolConfiguration::builder()
                    .set_tool_choice(tool_choice)
                    .set_tools(Some(tools))
                    .build()
                    .unwrap(),
            );
        }

        Ok(builder
            .set_inference_config(Some(
                aws_sdk_bedrockruntime::types::InferenceConfiguration::builder(
                )
                .top_p(top_p.unwrap_or_default())
                .temperature(temperature.unwrap_or_default())
                .max_tokens(max_tokens as i32)
                .set_stop_sequences(stop_sequences)
                .build(),
            ))
            .build()
            .unwrap())
    }
}

impl
    TryConvert<
        aws_sdk_bedrockruntime::operation::converse::ConverseOutput,
        async_openai::types::CreateChatCompletionResponse,
    > for BedrockConverter
{
    type Error = MapperError;

    #[allow(clippy::too_many_lines)]
    fn try_convert(
        &self,
        value: aws_sdk_bedrockruntime::operation::converse::ConverseOutput,
    ) -> std::result::Result<CreateChatCompletionResponse, Self::Error> {
        use async_openai::types as openai;
        let model = value
            .trace
            .and_then(|t| t.prompt_router)
            .and_then(|r| r.invoked_model_id)
            .unwrap_or_default();

        let created = 0;
        let usage = value.usage.unwrap();

        let usage = openai::CompletionUsage {
            prompt_tokens: usage.input_tokens.try_into().unwrap_or(0),
            completion_tokens: usage.output_tokens.try_into().unwrap_or(0),
            total_tokens: usage.total_tokens.try_into().unwrap_or(0),
            prompt_tokens_details: Some(openai::PromptTokensDetails {
                audio_tokens: None,
                cached_tokens: usage
                    .cache_read_input_tokens
                    .and_then(|i| i.try_into().ok()),
            }),
            completion_tokens_details: None,
        };

        let mut tool_calls: Vec<openai::ChatCompletionMessageToolCall> =
            Vec::new();
        let mut content = None;
        for bedrock_content in
            value.output.unwrap().as_message().unwrap().content.clone()
        {
            match bedrock_content {
                ContentBlock::ToolUse(tool_use_block) => {
                    tool_calls.push(openai::ChatCompletionMessageToolCall {
                        id: tool_use_block.tool_use_id.clone(),
                        r#type: openai::ChatCompletionToolType::Function,
                        function: openai::FunctionCall {
                            name: tool_use_block.name.clone(),
                            arguments: tool_use_block
                                .input
                                .as_string()
                                .unwrap()
                                .to_string(),
                        },
                    });
                }
                ContentBlock::ToolResult(tool_result_block) => {
                    tool_calls.push(openai::ChatCompletionMessageToolCall {
                        id: tool_result_block.tool_use_id.clone(),
                        r#type: openai::ChatCompletionToolType::Function,
                        function: openai::FunctionCall {
                            name: tool_result_block.tool_use_id.clone(),
                            arguments: serde_json::to_string(&content)?,
                        },
                    });
                }
                ContentBlock::Text(text) => {
                    content = Some(text.clone());
                }
                ContentBlock::Image(_)
                | ContentBlock::Document(_)
                | ContentBlock::CachePoint(_)
                | ContentBlock::ReasoningContent(_)
                | ContentBlock::GuardContent(_)
                | ContentBlock::Video(_)
                | _ => {}
            }
        }
        let tool_calls = if tool_calls.is_empty() {
            None
        } else {
            Some(tool_calls)
        };

        #[allow(deprecated)]
        let message = openai::ChatCompletionResponseMessage {
            content,
            refusal: None,
            tool_calls,
            role: openai::Role::Assistant,
            function_call: None,
            audio: None,
        };

        let choice = openai::ChatChoice {
            index: 0,
            message,
            finish_reason: None,
            logprobs: None,
        };

        let response = openai::CreateChatCompletionResponse {
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
        aws_sdk_bedrockruntime::types::ConverseStreamOutput,
        async_openai::types::CreateChatCompletionStreamResponse,
    > for BedrockConverter
{
    type Error = MapperError;

    #[allow(clippy::too_many_lines)]
    fn try_convert_chunk(
        &self,
        value: aws_sdk_bedrockruntime::types::ConverseStreamOutput,
    ) -> Result<
        std::option::Option<CreateChatCompletionStreamResponse>,
        Self::Error,
    > {
        use async_openai::types as openai;
        use aws_sdk_bedrockruntime::types::ConverseStreamOutput as bedrock;

        const CHAT_COMPLETION_CHUNK_OBJECT: &str = "chat.completion.chunk";
        // TODO: These placeholder values for id, model, and created should be
        // replaced by actual values from the MessageStart event,
        // propagated by the stream handling logic.
        const PLACEHOLDER_STREAM_ID: &str = "bedrock-stream-id";
        const PLACEHOLDER_MODEL_NAME: &str = "bedrock-model";
        const DEFAULT_CREATED_TIMESTAMP: u32 = 0;

        #[allow(deprecated)]
        let mut choices = Vec::new();
        let mut completion_usage: openai::CompletionUsage =
            openai::CompletionUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
                prompt_tokens_details: None,
                completion_tokens_details: None,
            };
        match value {
            bedrock::MessageStart(message) => {
                let choice = openai::ChatChoiceStream {
                    index: 0,
                    delta: openai::ChatCompletionStreamResponseDelta {
                        role: Some(match message.role {
                            ConversationRole::Assistant => {
                                openai::Role::Assistant
                            }
                            ConversationRole::User => openai::Role::User,
                            _ => openai::Role::System,
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
            bedrock::ContentBlockStart(content_block_start) => {
                if let ContentBlockStart::ToolUse(tool_use) =
                    content_block_start.start.unwrap()
                {
                    let tool_call_chunk =
                        openai::ChatCompletionMessageToolCallChunk {
                            index: content_block_start
                                .content_block_index
                                .try_into()
                                .unwrap_or(0),
                            id: Some(tool_use.tool_use_id),
                            r#type: Some(
                                openai::ChatCompletionToolType::Function,
                            ),
                            function: Some(openai::FunctionCallStream {
                                name: Some(tool_use.name),
                                arguments: Some("".to_string()),
                            }),
                        };
                    let choice = openai::ChatChoiceStream {
                        index: 0,
                        delta: openai::ChatCompletionStreamResponseDelta {
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
            bedrock::ContentBlockDelta(content_block_delta_event) => {
                match content_block_delta_event.delta.unwrap() {
                    ContentBlockDelta::Text(text) => {
                        let choice = openai::ChatChoiceStream {
                            index: u32::try_from(
                                content_block_delta_event.content_block_index,
                            )
                            .unwrap_or(0),
                            delta: openai::ChatCompletionStreamResponseDelta {
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
                    ContentBlockDelta::ToolUse(tool_use) => {
                        let tool_call_chunk =
                            openai::ChatCompletionMessageToolCallChunk {
                                index: u32::try_from(
                                    content_block_delta_event
                                        .content_block_index,
                                )
                                .unwrap_or(0),
                                id: None, /* ID would have been sent with ContentBlockStart for this tool */
                                r#type: Some(
                                    openai::ChatCompletionToolType::Function,
                                ), // Assuming function
                                function: Some(openai::FunctionCallStream {
                                    name: None, /* Name would have been sent
                                                 * with ContentBlockStart */
                                    arguments: Some(tool_use.input),
                                }),
                            };
                        let choice = openai::ChatChoiceStream {
                            index: 0,
                            delta: openai::ChatCompletionStreamResponseDelta {
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
                    ContentBlockDelta::ReasoningContent(_) | _ => {}
                }
            }

            bedrock::Metadata(metadata) => {
                if let Some(usage) = metadata.usage {
                    completion_usage.prompt_tokens =
                        u32::try_from(usage.input_tokens).unwrap_or(0);
                    completion_usage.completion_tokens =
                        u32::try_from(usage.output_tokens).unwrap_or(0);
                    completion_usage.total_tokens =
                        u32::try_from(usage.total_tokens).unwrap_or(0);
                }
            }
            bedrock::ContentBlockStop(_) | bedrock::MessageStop(_) | _ => {}
        }

        Ok(Some(openai::CreateChatCompletionStreamResponse {
            id: PLACEHOLDER_STREAM_ID.to_string(), /* TODO: Use actual stream
                                                    * ID */
            choices,
            created: DEFAULT_CREATED_TIMESTAMP, /* TODO: Use actual created
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
