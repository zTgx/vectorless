// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! HTTP server and routing.

use crate::controllers;
use crate::middleware::{request_logging, cors_layer, require_api_key, ApiKeyAuth};
use axum::{
    routing::{get, post},
    Router,
};

/// Create the application router.
pub fn create_router(state: controllers::AppState, api_keys: Vec<String>) -> Router {
    // API key authentication state
    let auth_state = ApiKeyAuth::new(api_keys);

    Router::new()
        // Health check (no auth required)
        .route("/health", get(controllers::health))
        // Document routes
        .route("/documents", post(controllers::create_document).get(controllers::list_documents))
        .route("/documents/:id", get(controllers::get_document).delete(controllers::delete_document))
        .route("/documents/:id/content", post(controllers::upload_document_content))
        // Query route
        .route("/query", post(controllers::query))
        // Apply middleware stack
        .layer(axum::middleware::from_fn(request_logging))
        .layer(cors_layer())
        .layer(axum::middleware::from_fn_with_state(auth_state, require_api_key))
        .with_state(state)
}

/// Run the HTTP server.
pub async fn run_server(state: controllers::AppState, host: &str, port: u16, api_keys: Vec<String>) -> anyhow::Result<()> {
    let app = create_router(state, api_keys);
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("🚀 vectorless service server listening on http://{}", addr);
    tracing::info!("📚 API endpoints:");
    tracing::info!("  GET    /health              - Health check");
    tracing::info!("  GET    /documents           - List all documents");
    tracing::info!("  POST   /documents           - Create a new document");
    tracing::info!("  GET    /documents/:id       - Get document by ID");
    tracing::info!("  DELETE /documents/:id       - Delete document");
    tracing::info!("  POST   /documents/:id/content - Upload document content");
    tracing::info!("  POST   /query              - Query the RAG system");

    axum::serve(listener, app).await?;

    Ok(())
}
