// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Chat model interface.

use async_trait::async_trait;

/// A message in a chat conversation.
#[derive(Debug, Clone)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

/// The role of a message sender.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    User,
    Assistant,
    System,
}

/// Chat completion options.
#[derive(Debug, Clone, Default)]
pub struct ChatOptions {
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

/// Result of a chat completion request.
#[derive(Debug, Clone)]
pub struct ChatCompletion {
    pub content: String,
    pub finish_reason: Option<String>,
}

/// Async chat model interface.
#[async_trait]
pub trait ChatModel: Send + Sync {
    /// Generate a chat completion.
    async fn chat(&self, messages: &[Message], options: &ChatOptions) -> Result<ChatCompletion, Error>;
}

/// Chat model error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Authentication failed")]
    AuthenticationFailed,
}
