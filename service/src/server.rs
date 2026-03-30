// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! HTTP server and routing.

use crate::controllers;
use axum::{
    routing::{get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};

/// Create the application router.
pub fn create_router(state: controllers::AppState) -> Router {
    // CORS layer
    let cors = CorsLayer::new().allow_origin(Any);

    Router::new()
        .route("/health", get(controllers::health))
        .route("/documents", post(controllers::create_document).get(controllers::list_documents))
        .route("/documents/:id", get(controllers::get_document).delete(controllers::delete_document))
        .route("/documents/:id/content", post(controllers::upload_document_content))
        .route("/query", post(controllers::query))
        .layer(cors)
        .with_state(state)
}

/// Run the HTTP server.
pub async fn run_server(state: controllers::AppState, host: &str, port: u16) -> anyhow::Result<()> {
    let app = create_router(state);
    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    tracing::info!("Service server listening on {}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
