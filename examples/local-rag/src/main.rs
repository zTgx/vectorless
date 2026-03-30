// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Local RAG example - Demonstrates using vectorless for document indexing and retrieval.
//!
//! This example shows how to:
//! - Create a DocumentCollection with workspace persistence
//! - Index PDF and Markdown documents
//! - Query documents using various retrieval modes
//! - Manage document metadata

// TODO: Uncomment when implementing
// use std::path::PathBuf;
// use vectorless_core::client::DocumentCollection;
// use vectorless_llm::zai::ZaiClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    println!("🚀 Local RAG Example");
    println!("====================\n");

    // TODO: Initialize LLM client
    // let api_key = std::env::var("ZAI_API_KEY")?;
    // let llm = ZaiClient::new(&api_key);

    // TODO: Create document collection with workspace
    // let mut collection = DocumentCollection::with_workspace("./workspace")?;

    // TODO: Index documents
    // let pdf_id = collection.index("./documents/manual.pdf").await?;
    // let md_id = collection.index("./documents/README.md").await?;

    // TODO: Query documents
    // let structure = collection.get_document_structure(&pdf_id);
    // let content = collection.get_page_content(&pdf_id, "5-7");

    println!("Example structure created - implementation pending.\n");
    println!("To implement:");
    println!("  1. Set ZAI_API_KEY environment variable");
    println!("  2. Add sample documents to ./documents/");
    println!("  3. Uncomment and customize the TODO sections above");

    Ok(())
}

// TODO: Add helper functions for:
// - Document ingestion workflow
// - Query and retrieval examples
// - Document management operations
// - Batch indexing
