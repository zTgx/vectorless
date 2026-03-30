// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Document DTOs.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Document status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentStatus {
    Indexing,
    Ready,
    Failed,
}

/// Document metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub id: Uuid,
    pub title: String,
    pub status: DocumentStatus,
    pub section_count: usize,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Create document request.
#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub title: String,
}

/// Create document response.
#[derive(Debug, Serialize)]
pub struct CreateDocumentResponse {
    pub id: Uuid,
    pub status: DocumentStatus,
}

/// Upload document content request.
#[derive(Debug, Deserialize)]
pub struct UploadContentRequest {
    pub content: String,
}
