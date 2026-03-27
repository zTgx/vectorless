// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ZAI API client for chat completions.

use super::chat::{ChatCompletion, ChatModel, ChatOptions, Error as ChatError, Message, Role};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Default base URL for ZAI API (general purpose).
pub const ZAI_API_BASE: &str = "https://api.z.ai/api/paas/v4";

/// Base URL for ZAI Coding API (coding scenarios only).
pub const ZAI_CODING_BASE: &str = "https://api.z.ai/api/coding/paas/v4";

/// ZAI API client.
#[derive(Debug, Clone)]
pub struct ZaiClient {
    api_key: String,
    client: Client,
    model: String,
    base_url: String,
}

impl ZaiClient {
    /// Create a new ZAI client with the given API key (uses general endpoint).
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: Client::new(),
            model: "glm-5".to_string(),
            base_url: ZAI_API_BASE.to_string(),
        }
    }

    /// Create a new ZAI client with custom endpoint.
    pub fn with_endpoint(api_key: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: Client::new(),
            model: "glm-5".to_string(),
            base_url: endpoint.into(),
        }
    }

    /// Create a new ZAI client with custom model.
    pub fn with_model(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: Client::new(),
            model: model.into(),
            base_url: ZAI_API_BASE.to_string(),
        }
    }

    /// Create a new ZAI client for coding scenarios.
    pub fn for_coding(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: Client::new(),
            model: "glm-5".to_string(),
            base_url: ZAI_CODING_BASE.to_string(),
        }
    }

    /// Create a new ZAI client with all options.
    pub fn with_options(
        api_key: impl Into<String>,
        model: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            client: Client::new(),
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
impl ChatModel for ZaiClient {
    async fn chat(&self, messages: &[Message], options: &ChatOptions) -> Result<ChatCompletion, ChatError> {
        // Convert our Message format to ZAI format
        let zai_messages: Vec<ZaiMessage> = messages
            .iter()
            .map(|m| ZaiMessage {
                role: match m.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::System => "system",
                }
                .to_string(),
                content: m.content.clone(),
            })
            .collect();

        let request = ZaiRequest {
            model: self.model.clone(),
            messages: zai_messages,
            temperature: options.temperature.unwrap_or(1.0),
            stream: false,
        };

        let response = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .header("Content-Type", "application/json")
            .header("Accept-Language", "en-US,en")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| ChatError::RequestFailed(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unable to read error".to_string());
            return Err(ChatError::RequestFailed(format!(
                "ZAI API error ({}): {}",
                status, error_text
            )));
        }

        let zai_response: ZaiResponse = response
            .json()
            .await
            .map_err(|e| ChatError::InvalidResponse(e.to_string()))?;

        Ok(ChatCompletion {
            content: zai_response.choices.first().map(|c| c.message.content.clone())
                .unwrap_or_default(),
            finish_reason: zai_response.choices.first().and_then(|c| c.finish_reason.clone()),
        })
    }
}

/// ZAI API request format.
#[derive(Debug, Serialize)]
struct ZaiRequest {
    model: String,
    messages: Vec<ZaiMessage>,
    temperature: f32,
    stream: bool,
}

/// ZAI API message format.
#[derive(Debug, Serialize, Deserialize, Clone)]
struct ZaiMessage {
    role: String,
    content: String,
}

/// ZAI API response format.
#[derive(Debug, Deserialize)]
struct ZaiResponse {
    choices: Vec<ZaiChoice>,
}

/// ZAI API choice format.
#[derive(Debug, Deserialize)]
struct ZaiChoice {
    message: ZaiMessage,
    finish_reason: Option<String>,
}
