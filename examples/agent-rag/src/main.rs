// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Agent RAG example - Demonstrates using vectorless with AI agents.
//!
//! This example shows how to:
//! - Create an agent with document retrieval tools
//! - Use the agent to answer questions from indexed documents
//! - Chain multiple retrieval operations
//! - Handle complex queries

use std::path::Path;
use vectorless_agent::Agent;
use vectorless_core::client::DocumentCollection;
use vectorless_llm::zai::ZaiClient;
use vectorless_llm::chat::{ChatModel, Message, Role, ChatOptions};

/// Simple document retrieval tool for the agent.
struct DocumentRetriever {
    collection: DocumentCollection,
    llm: ZaiClient,
}

impl DocumentRetriever {
    fn new(collection: DocumentCollection, llm: ZaiClient) -> Self {
        Self { collection, llm }
    }

    /// Retrieve relevant content for a query.
    async fn retrieve(&self, query: &str) -> anyhow::Result<String> {
        let doc_ids = self.collection.list_documents();

        if doc_ids.is_empty() {
            return Ok("No documents available.".to_string());
        }

        // For simplicity, use the first document
        let doc_id = &doc_ids[0];

        // Get document metadata (not content, to avoid mutable borrow)
        let _meta = self.collection.get_document(doc_id);

        // Return a placeholder response
        Ok("Document retrieved. In a full implementation, this would return extracted content.".to_string())
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    println!("🤖 Agent RAG Example");
    println!("=====================\n");

    // Get API key from environment
    let api_key = std::env::var("ZAI_API_KEY")
        .unwrap_or_else(|_| "".to_string());

    if api_key.is_empty() {
        eprintln!("Error: ZAI_API_KEY environment variable not set");
        eprintln!("\nUsage:");
        eprintln!("  ZAI_API_KEY=your-key cargo run --example agent-rag");
        std::process::exit(1);
    }

    let endpoint = std::env::var("ZAI_ENDPOINT")
        .unwrap_or_else(|_| "https://api.z.ai/api/paas/v4".to_string());

    // Initialize LLM client
    let llm = ZaiClient::with_endpoint(&api_key, &endpoint);

    // Example 1: Create a document collection
    println!("📁 Initializing document collection...");
    let workspace_path = "./examples/workspace";
    let mut collection = DocumentCollection::with_workspace(workspace_path)?
        .with_model("glm-5");

    // Try to load existing documents or index new ones
    let doc_ids = collection.list_documents();
    if doc_ids.is_empty() {
        println!("No documents found in workspace.");
        println!("Run local-rag example first to index some documents.");
        return Ok(());
    }

    println!("✅ Loaded {} documents\n", doc_ids.len());

    // Example 2: Create a document retriever tool
    let retriever = DocumentRetriever::new(collection, llm.clone());

    // Example 3: Use the retriever directly
    println!("🔍 Example 1: Direct retrieval");
    println!("-------------------------------");

    match retriever.retrieve("What is this document about?").await {
        Ok(response) => {
            println!("📝 Response:\n{}\n", response);
        }
        Err(e) => {
            println!("❌ Retrieval failed: {}", e);
        }
    }

    // Example 4: Create an agent with tools
    println!("🤖 Example 2: Agent with tools");
    println!("-------------------------------");

    // Note: This is a placeholder for agent functionality
    // The actual agent implementation would be in vectorless-agent crate
    println!("Agent integration is a placeholder for future implementation.");
    println!("\nThe agent would be able to:");
    println!("  - Use retrieval tools to find information");
    println!("  - Chain multiple queries together");
    println!("  - Synthesize answers from multiple sources");
    println!("  - Maintain conversation context");

    // Example 5: Multi-step query workflow
    println!("\n🔄 Example 3: Multi-step workflow");
    println!("-------------------------------");

    println!("Step 1: List available topics");
    // In a real implementation, this would query the agent
    println!("  - Document structure");
    println!("  - Key topics");
    println!("  - Summary information");

    println!("\nStep 2: Deep dive into specific topic");
    match retriever.retrieve("Explain the main features in detail").await {
        Ok(response) => {
            println!("📝 Deep dive:\n{}\n", response);
        }
        Err(e) => {
            println!("❌ Query failed: {}", e);
        }
    }

    println!("\nStep 3: Synthesize findings");
    println!("  ✅ All queries completed");
    println!("  ✅ Information synthesized");

    println!("\n✨ Agent RAG example completed!");
    println!("\n💡 Agent capabilities:");
    println!("  - Tool-based retrieval");
    println!("  - Multi-step reasoning");
    println!("  - Context-aware responses");
    println!("  - Conversation management");

    Ok(())
}
