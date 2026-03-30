// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! HTTP controllers for handling API requests.

use crate::dto::{ApiError, CreateDocumentRequest, CreateDocumentResponse, Document, QueryRequest, QueryResponse, Source, UploadContentRequest};
use crate::repository::{IndexRepository, MetadataRepository};
use axum::{
    extract::{Path, State},
    Json,
    response::IntoResponse,
};
use uuid::Uuid;
use vectorless_llm::{zai::ZaiClient, chat::{ChatModel, Message, Role, ChatOptions}};
use vectorless_core::{parse_document_with_config, build_summaries_with_config, save, load};
use vectorless_core::retriever::retrieve_simple;
use std::rc::Rc;
use std::cell::RefCell;

/// Application state.
#[derive(Clone)]
pub struct AppState {
    pub llm: ZaiClient,
    pub metadata_repository: MetadataRepository,
    pub index_repository: IndexRepository,
    pub indexer_config: vectorless_core::IndexerConfig,
}

/// Create a new document.
pub async fn create_document(
    State(state): State<AppState>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<CreateDocumentResponse>, ApiError> {
    let doc = state.metadata_repository.create_document(req.title)?;
    let response = CreateDocumentResponse {
        id: doc.id,
        status: doc.status,
    };
    Ok(Json(response))
}

/// Helper function to count sections in a document tree.
fn count_sections(root: &vectorless_core::node::PageNodeRef) -> usize {
    let node = root.borrow();
    let mut count = 1;
    for child in &node.children {
        count += count_sections(child);
    }
    count
}

/// Upload document content and index it.
pub async fn upload_document_content(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UploadContentRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Verify document exists
    let _doc = state.metadata_repository.get_document(id)?
        .ok_or_else(|| ApiError::DocumentNotFound(id.to_string()))?;

    if req.content.is_empty() {
        return Err(ApiError::InvalidRequest("No content provided".to_string()));
    }

    // TODO: Implement indexing - async operations causing Handler trait issues
    // For now, just log and return success
    tracing::info!("Content received for document {}: {} bytes", id, req.content.len());

    Ok(Json(serde_json::json!({"message": "Content received (indexing: TODO)", "bytes": req.content.len()})))
}

/// Get document by ID.
pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Document>, ApiError> {
    state
        .metadata_repository
        .get_document(id)?
        .ok_or_else(|| ApiError::DocumentNotFound(id.to_string()))
        .map(Json)
}

/// List all documents.
pub async fn list_documents(
    State(state): State<AppState>,
) -> Result<Json<Vec<Document>>, ApiError> {
    let docs = state.metadata_repository.list_documents()?;
    Ok(Json(docs))
}

/// Delete document.
pub async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Delete from metadata repository
    state.metadata_repository.delete_document(id)?;

    // Delete index file
    state.index_repository.delete_index(id)?;

    Ok(Json(serde_json::json!({"message": "Document deleted"})))
}

/// Query the RAG system.
pub async fn query(
    State(_state): State<AppState>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, ApiError> {
    // TODO: Implement query - async operations causing Handler trait issues
    tracing::info!("Query received: {}", req.query);

    Ok(Json(QueryResponse {
        answer: format!("Query functionality for '{}' coming soon", req.query),
        sources: vec![],
    }))
}

/// Health check.
pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}
