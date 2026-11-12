// Modified for the Prometheus Skill System vendored optimizer. See PROVENANCE.md.
use super::{LanguageModel, Message, ModelRequest, ModelResponse, Usage};
use crate::error::{DspyError, Result};
use crate::primitives::{Chunk, FinishReason};
use async_trait::async_trait;
use eventsource_stream::{EventStreamError, Eventsource};
use futures::stream::{BoxStream, StreamExt};
use serde::{Deserialize, Serialize};

use std::env;

const OPENAI_CHAT_COMPLETIONS_URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Clone)]
pub struct OpenAIModel {
    pub model_name: String,
    pub api_key: String,
    pub client: reqwest::Client,
}

impl OpenAIModel {
    pub fn new() -> Self {
        dotenvy::dotenv().ok();
        let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set");
        Self {
            model_name: "gpt-4o".to_string(),
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Create a model for an OpenAI-compatible API base URL, such as http://localhost:8181/v1.
    ///
    /// The request includes a bearer token only when an API key is supplied.
    pub fn with_endpoint(
        model_name: String,
        base_url: String,
        api_key: Option<String>,
    ) -> impl LanguageModel + Clone {
        EndpointModel {
            model_name,
            url: format!("{}/chat/completions", base_url.trim_end_matches('/')),
            api_key,
            client: reqwest::Client::new(),
        }
    }

    fn chat(&self) -> OpenAIChat<'_> {
        OpenAIChat {
            model_name: &self.model_name,
            url: OPENAI_CHAT_COMPLETIONS_URL,
            api_key: Some(&self.api_key),
            client: &self.client,
        }
    }
}

#[derive(Clone)]
struct EndpointModel {
    model_name: String,
    url: String,
    api_key: Option<String>,
    client: reqwest::Client,
}

impl EndpointModel {
    fn chat(&self) -> OpenAIChat<'_> {
        OpenAIChat {
            model_name: &self.model_name,
            url: &self.url,
            api_key: self.api_key.as_deref(),
            client: &self.client,
        }
    }
}

struct OpenAIChat<'a> {
    model_name: &'a str,
    url: &'a str,
    api_key: Option<&'a str>,
    client: &'a reqwest::Client,
}

#[derive(Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: DeltaMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct DeltaMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct StreamResponse {
    choices: Vec<StreamChoice>,
}

#[async_trait]
impl LanguageModel for OpenAIModel {
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        self.chat().complete(request).await
    }

    async fn stream(&self, request: ModelRequest) -> Result<BoxStream<'static, Result<Chunk>>> {
        self.chat().stream(request).await
    }
}

#[async_trait]
impl LanguageModel for EndpointModel {
    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        self.chat().complete(request).await
    }

    async fn stream(&self, request: ModelRequest) -> Result<BoxStream<'static, Result<Chunk>>> {
        self.chat().stream(request).await
    }
}

impl OpenAIChat<'_> {
    fn request(&self, body: &OpenAIChatRequest) -> reqwest::RequestBuilder {
        let request = self.client.post(self.url).json(body);
        match self.api_key {
            Some(key) => request.bearer_auth(key),
            None => request,
        }
    }

    async fn complete(&self, request: ModelRequest) -> Result<ModelResponse> {
        let req_body = OpenAIChatRequest {
            model: self.model_name.to_owned(),
            messages: request.messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: None,
        };

        let res = self
            .request(&req_body)
            .send()
            .await
            .map_err(|e| DspyError::ModelCall(e.to_string()))?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(DspyError::ModelCall(format!(
                "OpenAI API error: {}",
                error_text
            )));
        }

        let body: OpenAIChatResponse = res
            .json()
            .await
            .map_err(|e| DspyError::ModelCall(e.to_string()))?;

        let choice = body
            .choices
            .first()
            .ok_or_else(|| DspyError::ModelCall("No choices returned from OpenAI".to_string()))?;

        Ok(ModelResponse {
            content: choice.message.content.clone(),
            usage: body.usage,
        })
    }

    async fn stream(&self, request: ModelRequest) -> Result<BoxStream<'static, Result<Chunk>>> {
        let req_body = OpenAIChatRequest {
            model: self.model_name.to_owned(),
            messages: request.messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            stream: Some(true),
        };

        let res = self
            .request(&req_body)
            .send()
            .await
            .map_err(|e| DspyError::ModelCall(e.to_string()))?;

        if !res.status().is_success() {
            let error_text = res.text().await.unwrap_or_default();
            return Err(DspyError::ModelCall(format!(
                "OpenAI API stream error: {}",
                error_text
            )));
        }

        let stream = res.bytes_stream().eventsource().map(
            |event: std::result::Result<
                eventsource_stream::Event,
                EventStreamError<reqwest::Error>,
            >|
             -> Result<Chunk> {
                match event {
                    Ok(event) => {
                        if event.data == "[DONE]" {
                            return Ok(Chunk::Finish(FinishReason::Stop));
                        }
                        let resp: StreamResponse = serde_json::from_str(&event.data)
                            .map_err(|e| DspyError::ModelCall(format!("Parse error: {}", e)))?;

                        if let Some(choice) = resp.choices.first() {
                            if let Some(content) = &choice.delta.content {
                                return Ok(Chunk::Content(content.clone()));
                            }
                            if let Some(reason) = &choice.finish_reason {
                                let reason = match reason.as_str() {
                                    "stop" => FinishReason::Stop,
                                    "length" => FinishReason::Length,
                                    "content_filter" => FinishReason::ContentFilter,
                                    "tool_calls" => FinishReason::ToolCalls,
                                    _ => FinishReason::Other(reason.clone()),
                                };
                                return Ok(Chunk::Finish(reason));
                            }
                        }
                        // Default empty chunk if nothing relevant
                        Ok(Chunk::Content("".to_string()))
                    }
                    Err(e) => Err(DspyError::ModelCall(e.to_string())),
                }
            },
        );

        Ok(Box::pin(stream))
    }
}
