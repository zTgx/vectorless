// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Service client example - Demonstrates using the vectorless HTTP API.
//!
//! This example shows how to:
//! - Connect to the vectorless service
//! - Create and manage documents
//! - Upload document content
//! - Query the RAG system
//! - Handle API responses

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Vectorless service client.
struct VectorlessClient {
    base_url: String,
    client: Client,
    api_key: Option<String>,
}

impl VectorlessClient {
    /// Create a new client.
    fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
            api_key: None,
        }
    }

    /// Set API key for authentication.
    fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Get the default headers for requests.
    fn get_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        if let Some(api_key) = &self.api_key {
            headers.insert(
                reqwest::header::HeaderName::from_static("x-api-key"),
                reqwest::header::HeaderValue::from_str(api_key).unwrap(),
            );
        }

        headers
    }

    /// Check service health.
    async fn health(&self) -> anyhow::Result<HealthResponse> {
        let response = self
            .client
            .get(format!("{}/health", self.base_url))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Health check failed: {}", response.status()))
        }
    }

    /// List all documents.
    async fn list_documents(&self) -> anyhow::Result<Vec<Document>> {
        let response = self
            .client
            .get(format!("{}/documents", self.base_url))
            .headers(self.get_headers())
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Failed to list documents: {}", response.status()))
        }
    }

    /// Create a new document.
    async fn create_document(&self, title: impl Into<String>) -> anyhow::Result<CreateDocumentResponse> {
        let request = CreateDocumentRequest { title: title.into() };

        let response = self
            .client
            .post(format!("{}/documents", self.base_url))
            .headers(self.get_headers())
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let error: ApiError = response.json().await.unwrap_or_else(|_| ApiError {
                error: "Unknown error".to_string(),
            });
            Err(anyhow::anyhow!("Failed to create document: {}", error.error))
        }
    }

    /// Get document by ID.
    async fn get_document(&self, id: &str) -> anyhow::Result<Document> {
        let response = self
            .client
            .get(format!("{}/documents/{}", self.base_url, id))
            .headers(self.get_headers())
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Failed to get document: {}", response.status()))
        }
    }

    /// Upload document content.
    async fn upload_content(&self, id: &str, content: &str) -> anyhow::Result<serde_json::Value> {
        let request = UploadContentRequest {
            content: content.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/documents/{}/content", self.base_url, id))
            .headers(self.get_headers())
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Failed to upload content: {}", response.status()))
        }
    }

    /// Delete a document.
    async fn delete_document(&self, id: &str) -> anyhow::Result<serde_json::Value> {
        let response = self
            .client
            .delete(format!("{}/documents/{}", self.base_url, id))
            .headers(self.get_headers())
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Failed to delete document: {}", response.status()))
        }
    }

    /// Query the RAG system.
    async fn query(&self, query: &str) -> anyhow::Result<QueryResponse> {
        let request = QueryRequest {
            query: query.to_string(),
        };

        let response = self
            .client
            .post(format!("{}/query", self.base_url))
            .headers(self.get_headers())
            .json(&request)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Query failed: {}", response.status()))
        }
    }
}

// ============================================================
// API Types
// ============================================================

#[derive(Debug, Deserialize)]
struct HealthResponse {
    status: String,
}

#[derive(Debug, Serialize)]
struct CreateDocumentRequest {
    title: String,
}

#[derive(Debug, Deserialize)]
struct CreateDocumentResponse {
    id: String,
    status: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Document {
    id: String,
    #[serde(rename = "type")]
    doc_type: String,
    title: String,
    status: String,
    created_at: Option<String>,
    modified_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct UploadContentRequest {
    content: String,
}

#[derive(Debug, Serialize)]
struct QueryRequest {
    query: String,
}

#[derive(Debug, Deserialize)]
struct QueryResponse {
    answer: String,
    sources: Vec<Source>,
}

#[derive(Debug, Deserialize)]
struct Source {
    document_id: String,
    section: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    error: String,
}

// ============================================================
// Main
// ============================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    println!("🌐 Service Client Example");
    println!("=======================\n");

    // Get service URL from environment
    let service_url = std::env::var("SERVICE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Get optional API key
    let api_key = std::env::var("SERVICE_API_KEY").ok();

    // Create client
    let mut client = VectorlessClient::new(&service_url);
    if let Some(key) = api_key {
        client = client.with_api_key(&key);
        println!("🔑 Using API key authentication");
    }

    println!("📍 Service URL: {}\n", service_url);

    // Example 1: Health check
    println!("❤️  Example 1: Health Check");
    println!("----------------------------");

    match client.health().await {
        Ok(health) => println!("✅ Service status: {}", health.status),
        Err(e) => {
            println!("❌ Service unavailable: {}", e);
            println!("\nMake sure the vectorless service is running:");
            println!("  docker compose -f docker/docker-compose.yml up -d");
            println!("  Or: cargo run -p vectorless-service");
            return Ok(());
        }
    }

    println!();

    // Example 2: List documents
    println!("📚 Example 2: List Documents");
    println!("------------------------------");

    match client.list_documents().await {
        Ok(docs) => {
            println!("✅ Found {} document(s)", docs.len());
            for doc in &docs {
                println!("  - {} ({})", doc.id, doc.title);
            }
        }
        Err(e) => println!("❌ Failed to list documents: {}", e),
    }

    println!();

    // Example 3: Create a document
    println!("📄 Example 3: Create Document");
    println!("------------------------------");

    match client.create_document("Example Document").await {
        Ok(response) => {
            println!("✅ Created document: {}", response.id);
            println!("   Status: {}", response.status);

            // Example 4: Upload content
            println!();
            println!("📝 Example 4: Upload Content");
            println!("------------------------------");

            let sample_content = r#"# Example Document

This is a sample document for the vectorless service.

## Getting Started

Vectorless provides tree-based RAG without vector databases.

## Features

- Fast indexing
- Intelligent retrieval
- Simple API
"#;

            match client.upload_content(&response.id, sample_content).await {
                Ok(result) => println!("✅ Content uploaded"),
                Err(e) => println!("❌ Failed to upload: {}", e),
            }
        }
        Err(e) => println!("❌ Failed to create document: {}", e),
    }

    println!();

    // Example 5: Query the RAG system
    println!("🔍 Example 5: Query RAG");
    println!("------------------------");

    match client.query("What are the main features?").await {
        Ok(response) => {
            println!("✅ Query successful");
            println!("📝 Answer:\n{}", response.answer);
            println!("📚 Sources: {} found", response.sources.len());
        }
        Err(e) => println!("❌ Query failed: {}", e),
    }

    println!();

    // Example 6: Delete a document
    println!("🗑️  Example 6: Delete Document");
    println!("------------------------------");

    // List documents again
    if let Ok(docs) = client.list_documents().await {
        if let Some(first_doc) = docs.first() {
            match client.delete_document(&first_doc.id).await {
                Ok(_) => println!("✅ Deleted document: {}", first_doc.id),
                Err(e) => println!("❌ Failed to delete: {}", e),
            }
        }
    }

    println!();
    println!("✨ Service client example completed!");
    println!("\n💡 Tips:");
    println!("  - Use SERVICE_URL to point to different endpoints");
    println!("  - Set SERVICE_API_KEY for authenticated requests");
    println!("  - The service must be running before executing this example");

    Ok(())
}
