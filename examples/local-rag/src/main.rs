// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Local RAG example - Demonstrates using vectorless for document indexing and retrieval.
//!
//! This example shows how to:
//! - Create a DocumentCollection with workspace persistence
//! - Index PDF and Markdown documents
//! - Query documents using various retrieval modes
//! - Manage document metadata

use std::path::Path;
use vectorless_core::client::DocumentCollection;
use vectorless_llm::zai::ZaiClient;

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

    // Get API key from environment
    let api_key = std::env::var("ZAI_API_KEY")
        .unwrap_or_else(|_| "".to_string());

    if api_key.is_empty() {
        eprintln!("Error: ZAI_API_KEY environment variable not set");
        eprintln!("\nUsage:");
        eprintln!("  ZAI_API_KEY=your-key cargo run -p local-rag");
        std::process::exit(1);
    }

    let endpoint = std::env::var("ZAI_ENDPOINT")
        .unwrap_or_else(|_| "https://api.z.ai/api/paas/v4".to_string());

    // Initialize LLM client
    let llm = ZaiClient::with_endpoint(&api_key, &endpoint);

    // Create document collection with workspace persistence
    let workspace_path = "./examples/workspace";
    let mut collection = DocumentCollection::with_workspace(workspace_path)?
        .with_model("glm-5");

    println!("📁 Workspace: {}", workspace_path);
    println!("📝 Existing documents: {}\n", collection.list_documents().len());

    // Example 1: Index a Markdown document
    println!("📄 Example 1: Indexing Markdown document");
    println!("-------------------------------------------");

    let md_path = "./examples/documents/sample.md";
    if Path::new(md_path).exists() {
        match collection.index(md_path).await {
            Ok(doc_id) => {
                println!("✅ Indexed Markdown document: {}", doc_id);

                // Get document metadata
                let meta = collection.get_document(&doc_id);
                println!("📊 Metadata: {}", meta);

                // Get document structure
                let structure = collection.get_document_structure(&doc_id);
                println!("🌳 Structure: {}", structure);
            }
            Err(e) => {
                println!("❌ Failed to index Markdown: {}", e);
            }
        }
    } else {
        println!("⚠️  Markdown file not found: {}", md_path);
        println!("   Creating sample file...");
        create_sample_documents()?;
        let doc_id = collection.index(md_path).await?;
        println!("✅ Indexed sample document: {}", doc_id);
    }

    println!();

    // Example 2: Index a PDF document
    println!("📄 Example 2: Indexing PDF document");
    println!("-------------------------------------------");

    let pdf_path = "./examples/documents/sample.pdf";
    if Path::new(pdf_path).exists() {
        match collection.index(pdf_path).await {
            Ok(doc_id) => {
                println!("✅ Indexed PDF document: {}", doc_id);

                // Get page content
                let content = collection.get_page_content(&doc_id, "1-3");
                println!("📄 Pages 1-3 preview: {} chars", content.len());
            }
            Err(e) => {
                println!("❌ Failed to index PDF: {}", e);
            }
        }
    } else {
        println!("⚠️  PDF file not found: {}", pdf_path);
    }

    println!();

    // Example 3: List all documents
    println!("📚 Example 3: Listing all documents");
    println!("-------------------------------------------");

    let doc_ids = collection.list_documents();
    println!("Total documents: {}", doc_ids.len());
    for doc_id in &doc_ids {
        let meta = collection.get_document(doc_id);
        // Just show the doc_id since metadata is JSON
        println!("  - {}", doc_id);
    }

    println!();

    // Example 4: Query using retriever
    println!("🔍 Example 4: Query documents");
    println!("------------------------------");

    if !doc_ids.is_empty() {
        use vectorless_core::retriever::retrieve_simple;

        let first_doc = &doc_ids[0];
        println!("Querying document: {}", first_doc);

        // Get content for querying
        let content = collection.get_page_content(first_doc, "1-100");

        // Simple query: check if keywords exist in content
        let query = "features";
        if content.to_lowercase().contains(&query.to_lowercase()) {
            println!("✅ Found content matching: '{}'", query);
        }
    }

    println!();

    // Example 5: Remove a document
    println!("🗑️  Example 5: Document management");
    println!("------------------------------");

    if doc_ids.len() > 1 {
        let last_doc = &doc_ids[doc_ids.len() - 1];
        println!("Removing document: {}", last_doc);
        match collection.remove_document(last_doc) {
            Ok(_) => println!("✅ Document removed"),
            Err(e) => println!("❌ Failed to remove: {}", e),
        }
    }

    println!();
    println!("✨ Example completed!");
    println!("\n💡 Tips:");
    println!("  - Documents are persisted in {}", workspace_path);
    println!("  - Re-run to load existing documents from workspace");
    println!("  - Check workspace/_meta.json for document registry");

    Ok(())
}

// Helper function to create sample documents for testing
fn create_sample_documents() -> anyhow::Result<()> {
    use std::fs;

    let docs_dir = "./examples/documents";
    fs::create_dir_all(docs_dir)?;

    // Create sample Markdown document
    let sample_md = r#"# Sample Document

## Introduction

This is a sample Markdown document for testing the vectorless library.

## Features

- **Tree-based indexing** - Documents are organized hierarchically
- **LLM navigation** - Intelligent content retrieval
- **No vector database** - Simplified infrastructure

## Getting Started

1. Create a DocumentCollection
2. Index your documents
3. Query using natural language

## Conclusion

Vectorless provides a simple yet powerful RAG system.
"#;

    fs::write(format!("{}/sample.md", docs_dir), sample_md)?;

    println!("📝 Created sample document: {}/sample.md", docs_dir);

    Ok(())
}
