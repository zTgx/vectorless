// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Error types for the service layer.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// API error type.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Parsing error: {0}")]
    Parsing(String),

    #[error("Indexing error: {0}")]
    Indexing(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::DocumentNotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ApiError::InvalidRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Storage(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Parsing(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Indexing(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Query(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            ApiError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(json!({
            "error": message,
            "code": status.as_u16(),
        }));

        (status, body).into_response()
    }
}
