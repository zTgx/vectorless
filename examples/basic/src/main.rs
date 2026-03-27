// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Basic usage example for vectorless.

use std::path::Path;
use vectorless_core::{
    build_summaries_with_config, load, parse_document_with_config, retrieve, save,
    IndexerConfig, PageNodeRef,
};
use vectorless_llm::{chat::ChatModel, zai::ZaiClient, ZAI_API_BASE};

fn load_env() {
    // Load .env file if it exists
    dotenv::dotenv().ok();
}

fn get_api_key() -> Result<String, Box<dyn std::error::Error>> {
    std::env::var("ZAI_API_KEY").map_err(|_| {
        "ZAI_API_KEY not found in environment or .env file. Please set it before running.".into()
    })
}

fn get_endpoint() -> String {
    std::env::var("ZAI_ENDPOINT").unwrap_or_else(|_| ZAI_API_BASE.to_string())
}

/// Build the index from a document with custom config.
async fn build_index(api_key: &str, endpoint: &str, doc_path: &str) -> Result<PageNodeRef, Box<dyn std::error::Error>> {
    println!("Parsing document...");
    let text = std::fs::read_to_string(doc_path)?;

    // Use custom config for better quality
    let config = IndexerConfig::builder()
        .subsection_threshold(200)  // More granular splitting
        .max_segment_tokens(4000)    // More context for segmentation
        .summary_model("glm-5")      // ZAI model
        .max_summary_tokens(200)     // Longer summaries
        .build();

    // Use ZAI client with custom endpoint
    let llm = ZaiClient::with_endpoint(api_key, endpoint);
    let tree = parse_document_with_config(&llm, &text, &config).await?;

    println!("Building summaries (this makes LLM calls)...");
    build_summaries_with_config(&llm, &tree, &config).await?;

    let index_path = "index.json";
    println!("Saving index to {}", index_path);
    save(&tree, index_path)?;

    Ok(tree)
}

/// Query the index and generate an answer.
async fn ask(api_key: &str, endpoint: &str, query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let index_path = "index.json";

    if !Path::new(index_path).exists() {
        return Err("Index not found. Run build_index() first.".into());
    }

    let tree = load(index_path)?;
    let llm = ZaiClient::with_endpoint(api_key, endpoint);
    let context = retrieve(&llm, query, &tree).await?;

    let messages = vec![
        vectorless_llm::chat::Message {
            role: vectorless_llm::chat::Role::System,
            content: "Answer using only the context provided.".to_string(),
        },
        vectorless_llm::chat::Message {
            role: vectorless_llm::chat::Role::User,
            content: format!("Context:\n{}\n\nQuestion: {}", context, query),
        },
    ];

    let response = llm
        .chat(
            &messages,
            &vectorless_llm::chat::ChatOptions {
                temperature: Some(0.0),
                max_tokens: Some(500),
            },
        )
        .await?;

    Ok(response.content.trim().to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    load_env();

    // Get API key
    let api_key = match get_api_key() {
        Ok(key) => key,
        Err(e) => {
            println!("Error: {}", e);
            println!("\nCreate a .env file with:");
            println!("ZAI_API_KEY=your-api-key-here");
            println!("\nOr set the environment variable:");
            println!("export ZAI_API_KEY=your-api-key-here");
            return Ok(());
        }
    };

    let endpoint = get_endpoint();
    if endpoint != ZAI_API_BASE {
        println!("Using custom endpoint: {}", endpoint);
    }

    let doc_path = "document.md";

    // Check if document exists
    if Path::new(doc_path).exists() {
        // Build the index
        println!("Building index...");
        match build_index(&api_key, &endpoint, doc_path).await {
            Ok(_) => println!("Index built successfully!"),
            Err(e) => {
                println!("Error building index: {}", e);
                println!("Note: Make sure ZAI_API_KEY is set correctly and the API is accessible");
            }
        }
    } else {
        println!("Document file '{}' not found. Skipping index build.", doc_path);
        println!("Create a document.md file to test the indexing pipeline.");
    }

    // Then query it (if index exists)
    if Path::new("index.json").exists() {
        let query = "What is this document about?";
        println!("\nQuery: {}", query);
        match ask(&api_key, &endpoint, query).await {
            Ok(answer) => println!("Answer: {}", answer),
            Err(e) => println!("Error: {}", e),
        }
    }

    Ok(())
}
