// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Error types for the vectorless SDK.

/// SDK result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the vectorless SDK.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// HTTP request failed.
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Failed to parse response body.
    #[error("Failed to parse response: {0}")]
    ParseError(#[from] serde_json::Error),

    /// API returned an error response.
    #[error("API error: {0}")]
    ApiError(String),

    /// Document not found.
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    /// Invalid input parameter.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Authentication failed.
    #[error("Authentication failed")]
    AuthenticationFailed,

    /// Service unavailable.
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}
