// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Embedding model interface.

use async_trait::async_trait;

/// Embedding model options.
#[derive(Debug, Clone, Default)]
pub struct EmbeddingOptions {
    pub model: Option<String>,
    pub dimensions: Option<u32>,
}

/// A single embedding result.
#[derive(Debug, Clone)]
pub struct Embedding {
    pub embedding: Vec<f32>,
    pub index: usize,
}

/// Result of an embedding request.
#[derive(Debug, Clone)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<Embedding>,
    pub model: String,
}

/// Async embedding model interface.
#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    /// Generate embeddings for the given texts.
    async fn embed(&self, texts: &[&str], options: &EmbeddingOptions) -> Result<EmbeddingResponse, Error>;

    /// Generate embedding for a single text.
    async fn embed_one(&self, text: &str, options: &EmbeddingOptions) -> Result<Vec<f32>, Error> {
        let response = self.embed(&[text], options).await?;
        Ok(response.embeddings.into_iter().next().unwrap().embedding)
    }
}

/// Embedding model error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("API request failed: {0}")]
    RequestFailed(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Authentication failed")]
    AuthenticationFailed,
}
