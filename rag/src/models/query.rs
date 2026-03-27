// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Query model.

use serde::{Deserialize, Serialize};

/// Query request.
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    pub query: String,
    #[serde(default)]
    pub max_results: Option<usize>,
}

/// Query response.
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub answer: String,
    pub sources: Vec<Source>,
}

/// Source reference.
#[derive(Debug, Clone, Serialize)]
pub struct Source {
    pub document_id: String,
    pub section: String,
    pub content: String,
}
