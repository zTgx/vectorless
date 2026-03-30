// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Data types for the vectorless service API.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Health check response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HealthResponse {
    /// Service status (e.g., "ok")
    pub status: String,
}

/// Create document request.
#[derive(Debug, Clone, Serialize)]
pub struct CreateDocumentRequest {
    /// Document title
    pub title: String,
}

/// Create document response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateDocumentResponse {
    /// Document ID
    pub id: Uuid,

    /// Document status
    pub status: String,
}

/// Document metadata.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Document {
    /// Document ID
    pub id: Uuid,

    /// Document type
    #[serde(rename = "type")]
    pub doc_type: String,

    /// Document title
    pub title: String,

    /// Document description
    pub doc_description: String,

    /// Document status
    pub status: String,

    /// Page count (for PDF documents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_count: Option<usize>,

    /// Line count (for Markdown documents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_count: Option<usize>,

    /// When the document was created
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,

    /// When the document was last modified
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}

/// Upload document content request.
#[derive(Debug, Clone, Serialize)]
pub struct UploadContentRequest {
    /// Document content (text or base64-encoded)
    pub content: String,
}

/// Upload content response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UploadContentResponse {
    /// Success message
    pub message: String,

    /// Number of bytes uploaded
    pub bytes: usize,
}

/// Query request for RAG.
#[derive(Debug, Clone, Serialize)]
pub struct QueryRequest {
    /// Query text
    pub query: String,
}

/// Query response from RAG system.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryResponse {
    /// Generated answer
    pub answer: String,

    /// Source references
    pub sources: Vec<Source>,
}

/// Source reference in RAG response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Source {
    /// Document ID
    pub document_id: Uuid,

    /// Section title
    pub section: String,

    /// Content snippet
    pub content: String,
}

/// Delete document response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteDocumentResponse {
    /// Confirmation message
    pub message: String,
}

/// API error response.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiErrorResponse {
    /// Error message
    pub error: String,
}
