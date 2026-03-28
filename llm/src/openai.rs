// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! OpenAI API client for chat completions.
//!
//! Supports OpenAI and OpenAI-compatible APIs including:
//! - OpenAI (GPT-4o, GPT-4-turbo, GPT-3.5-turbo, o1)
//! - Azure OpenAI
//! - DeepSeek, Groq, and other compatible providers

use super::chat::{ChatCompletion, ChatModel, ChatOptions, Error as ChatError, Message, Role};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Default base URL for OpenAI API.
pub const OPENAI_API_BASE: &str = "https://api.openai.com/v1";

/// OpenAI API client.
#[derive(Debug, Clone)]
pub struct OpenAIClient {
    api_key: String,
    client: Client,
    model: String,
    base_url: String,
}

impl OpenAIClient {
    /// Create a new OpenAI client with the given API key.
    ///
    /// Uses GPT-4o by default.
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap();

        Self {
            api_key: api_key.into(),
            client,
            model: "gpt-4o".to_string(),
            base_url: OPENAI_API_BASE.to_string(),
        }
    }

    /// Create a new OpenAI client with custom model.
    pub fn with_model(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap();

        Self {
            api_key: api_key.into(),
            client,
            model: model.into(),
            base_url: OPENAI_API_BASE.to_string(),
        }
    }

    /// Create a new OpenAI client with custom endpoint.
    ///
    /// Useful for OpenAI-compatible APIs like Azure OpenAI, DeepSeek, Groq, etc.
    pub fn with_endpoint(api_key: impl Into<String>, endpoint: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap();

        Self {
            api_key: api_key.into(),
            client,
            model: "gpt-4o".to_string(),
            base_url: endpoint.into(),
        }
    }

    /// Create a new OpenAI client with all options.
    pub fn with_options(
        api_key: impl Into<String>,
        model: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .unwrap();

        Self {
            api_key: api_key.into(),
            client,
            model: model.into(),
            base_url: endpoint.into(),
        }
    }

    /// Set the model to use.
    pub fn set_model(&mut self, model: impl Into<String>) {
        self.model = model.into();
    }

    /// Set the base URL to use.
    pub fn set_endpoint(&mut self, endpoint: impl Into<String>) {
        self.base_url = endpoint.into();
    }
}

#[async_trait]
impl ChatModel for OpenAIClient {
    async fn chat(&self, messages: &[Message], options: &ChatOptions) -> Result<ChatCompletion, ChatError> {
        // Convert our Message format to OpenAI format
        let openai_messages: Vec<OpenAIMessage> = messages
            .iter()
            .map(|m| OpenAIMessage {
                role: match m.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::System => "system",
                }
                .to_string(),
                content: m.content.clone(),
            })
            .collect();

        let mut request = OpenAIRequest {
            model: self.model.clone(),
            messages: openai_messages,
            temperature: options.temperature.unwrap_or(1.0),
            ..Default::default()
        };

        // Set max_tokens if provided
        if let Some(max_tokens) = options.max_tokens {
            request.max_tokens = Some(max_tokens);
        }

        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                ChatError::RequestFailed(format!(
                    "Failed to send request to {}: {}",
                    url, e
                ))
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error".to_string());
            return Err(ChatError::RequestFailed(format!(
                "OpenAI API error ({}): {}",
                status, error_text
            )));
        }

        let openai_response: OpenAIResponse = response
            .json()
            .await
            .map_err(|e| ChatError::InvalidResponse(format!(
                "Failed to parse response: {}",
                e
            )))?;

        Ok(ChatCompletion {
            content: openai_response
                .choices
                .first()
                .and_then(|c| Some(c.message.content.clone()))
                .unwrap_or_default(),
            finish_reason: openai_response
                .choices
                .first()
                .and_then(|c| c.finish_reason.clone()),
        })
    }
}

/// OpenAI API request format.
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

impl Default for OpenAIRequest {
    fn default() -> Self {
        Self {
            model: String::new(),
            messages: Vec::new(),
            temperature: 1.0,
            max_tokens: None,
            top_p: None,
            stream: None,
        }
    }
}

/// OpenAI API message format.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI API response format.
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<OpenAIChoice>,
    #[serde(default)]
    usage: OpenAIUsage,
}

/// OpenAI API choice format.
#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    index: u32,
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

/// OpenAI API usage information.
#[derive(Debug, Deserialize, Default)]
struct OpenAIUsage {
    #[serde(default)]
    prompt_tokens: u32,
    #[serde(default)]
    completion_tokens: u32,
    #[serde(default)]
    total_tokens: u32,
}
