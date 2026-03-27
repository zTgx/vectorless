// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Basic usage example for vectorless.

use std::path::Path;
use vectorless_core::{
    build_summaries_with_config, load, parse_document_with_config, retrieve, save,
    IndexerConfig, PageNodeRef,
};
use vectorless_llm::chat::ChatModel;
use vectorless_llm::zai::ZaiClient;
use serde::Deserialize;

/// Configuration from .config.toml
#[derive(Debug, Deserialize)]
struct Config {
    llm: LlmConfig,
}

#[derive(Debug, Deserialize)]
struct LlmConfig {
    zai: ZaiConfig,
}

#[derive(Debug, Deserialize)]
struct ZaiConfig {
    api_key: String,
    #[serde(default)]
    endpoint: Option<String>,
    // Additional fields available for future use:
    // model: Option<String>,
}

/// Load configuration from .config.toml
fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = Path::new(".config.toml");

    if !config_path.exists() {
        return Err(format!(
            "Configuration file not found. Please create {}.\n\nExample:\n[llm.zai]\napi_key = \"your-api-key-here\"\nendpoint = \"https://api.z.ai/api/paas/v4\"",
            config_path.display()
        ).into());
    }

    let contents = std::fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&contents)?;
    println!("Configuration loaded successfully : {:?}", config);

    Ok(config)
}

/// Test API connection before processing
async fn test_api_connection(api_key: &str, endpoint: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing API connection to {}...", endpoint);

    let llm = ZaiClient::with_endpoint(api_key, endpoint);
    let messages = vec![
        vectorless_llm::chat::Message {
            role: vectorless_llm::chat::Role::User,
            content: "Hello".to_string(),
        },
    ];

    match llm.chat(&messages, &vectorless_llm::chat::ChatOptions {
        temperature: Some(0.0),
        max_tokens: Some(10),
    }).await {
        Ok(response) => {
            println!("API connection successful! Response: {}", response.content);
            Ok(())
        }
        Err(e) => {
            Err(format!("API connection failed: {}", e).into())
        }
    }
}

/// Build the index from a document with custom config.
async fn build_index(
    api_key: &str,
    endpoint: &str,
    doc_path: &str,
) -> Result<PageNodeRef, Box<dyn std::error::Error>> {
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
async fn ask(
    api_key: &str,
    endpoint: &str,
    query: &str,
) -> Result<String, Box<dyn std::error::Error>> {
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
    // Load configuration from .config.toml
    let config = match load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            println!("Error loading configuration: {}", e);
            println!("\nCreate a .config.toml file with your ZAI credentials:");
            println!("");
            println!("[llm.zai]");
            println!("api_key = \"your-api-key-here\"");
            println!("endpoint = \"https://api.z.ai/api/paas/v4\"");
            return Ok(());
        }
    };

    let zai_config = &config.llm.zai;
    let api_key = &zai_config.api_key;
    let endpoint = zai_config.endpoint.as_deref()
        .unwrap_or("https://api.z.ai/api/paas/v4");

    println!("Using endpoint: {}", endpoint);

    // Test API connection first
    println!("\nTesting API connection...");
    if let Err(e) = test_api_connection(api_key, endpoint).await {
        println!("API connection test failed: {}", e);
        println!("Please check:");
        println!("  1. Your API key is correct");
        println!("  2. The endpoint URL is correct");
        println!("  3. Your network connection is working");
        println!("  4. The API service is available");
        return Err(e);
    }
    println!("API connection test passed!\n");

    let doc_path = "docs/document.md";

    // Check if document exists
    if Path::new(doc_path).exists() {
        // Build the index
        println!("Building index...");
        match build_index(api_key, endpoint, doc_path).await {
            Ok(_) => println!("Index built successfully!"),
            Err(e) => {
                println!("Error building index: {}", e);
                println!("Note: Make sure api_key is correct and the API is accessible");
            }
        }
    } else {
        println!(
            "Document file '{}' not found. Skipping index build.",
            doc_path
        );
        println!("Create a document.md file to test the indexing pipeline.");
    }

    // Then query it (if index exists)
    if Path::new("index.json").exists() {
        let query = "What is this document about?";
        println!("\nQuery: {}", query);
        match ask(api_key, endpoint, query).await {
            Ok(answer) => println!("Answer: {}", answer),
            Err(e) => println!("Error: {}", e),
        }
    }

    Ok(())
}
