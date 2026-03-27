// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! HTTP server and routing.

use crate::handlers;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

/// Create the application router.
pub fn create_router(state: handlers::AppState) -> Router {
    // CORS layer
    let cors = CorsLayer::new().allow_origin(Any);

    Router::new()
        .route("/health", get(handlers::health))
        .route("/documents", post(handlers::create_document).get(handlers::list_documents))
        .route("/documents/:id", get(handlers::get_document).delete(handlers::delete_document))
        .route("/documents/:id/content", post(handlers::upload_document_content))
        .route("/query", post(handlers::query))
        .layer(cors)
        .with_state(state)
}

/// Run the HTTP server.
pub async fn run_server(state: handlers::AppState, host: &str, port: u16) -> anyhow::Result<()> {
    let app = create_router(state);
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("RAG server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
