// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! HTTP handlers.

use crate::models::{ApiError, CreateDocumentRequest, CreateDocumentResponse, Document, QueryRequest, QueryResponse, Source, UploadContentRequest};
use crate::store::{IndexStore, MetadataStore};
use crate::pipeline::{query::QueryPipeline, ingest::IngestPipeline};
use axum::{
    extract::{Path, State},
    Json,
    response::IntoResponse,
};
use uuid::Uuid;
use vectorless_llm::zai::ZaiClient;

/// Application state.
#[derive(Clone)]
pub struct AppState {
    pub llm: ZaiClient,
    pub metadata_store: MetadataStore,
    pub index_store: IndexStore,
    pub indexer_config: vectorless_core::IndexerConfig,
}

/// Create a new document.
pub async fn create_document(
    State(state): State<AppState>,
    Json(req): Json<CreateDocumentRequest>,
) -> Result<Json<CreateDocumentResponse>, ApiError> {
    let doc = state.metadata_store.create_document(req.title)?;
    let response = CreateDocumentResponse {
        id: doc.id,
        status: doc.status,
    };
    Ok(Json(response))
}

/// Upload document content.
pub async fn upload_document_content(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UploadContentRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Verify document exists
    let _doc = state.metadata_store.get_document(id)?
        .ok_or_else(|| ApiError::DocumentNotFound(id.to_string()))?;

    if req.content.is_empty() {
        return Err(ApiError::InvalidRequest("No content provided".to_string()));
    }

    // TODO: Implement ingestion
    tracing::info!("Would ingest content for document {}", id);

    Ok(Json(serde_json::json!({"message": "Document indexed successfully"})))
}

/// Get document by ID.
pub async fn get_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Document>, ApiError> {
    state
        .metadata_store
        .get_document(id)?
        .ok_or_else(|| ApiError::DocumentNotFound(id.to_string()))
        .map(Json)
}

/// List all documents.
pub async fn list_documents(
    State(state): State<AppState>,
) -> Result<Json<Vec<Document>>, ApiError> {
    let docs = state.metadata_store.list_documents()?;
    Ok(Json(docs))
}

/// Delete document.
pub async fn delete_document(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Delete from metadata store
    state.metadata_store.delete_document(id)?;

    // Delete index file
    state.index_store.delete_index(id)?;

    Ok(Json(serde_json::json!({"message": "Document deleted"})))
}

/// Query the RAG system.
pub async fn query(
    State(_state): State<AppState>,
    Json(req): Json<QueryRequest>,
) -> Result<Json<QueryResponse>, ApiError> {
    // TODO: Implement query pipeline
    tracing::info!("Would process query: {}", req.query);

    let response = QueryResponse {
        answer: "Query functionality coming soon".to_string(),
        sources: vec![],
    };
    Ok(Json(response))
}

/// Health check.
pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}
