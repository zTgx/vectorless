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
    base_url: Option<String>,
    // Additional fields available for future use:
    // coding_url: Option<String>,
    // model: Option<String>,
}

/// Load configuration from .config.toml or .config.toml.local
fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Try .config.toml.local first (for local overrides), then .config.toml
    let config_path = Path::new(".config.toml.local");
    let fallback_path = Path::new(".config.toml");

    let path = if config_path.exists() {
        config_path
    } else {
        fallback_path
    };

    if !path.exists() {
        return Err(format!(
            "Configuration file not found. Please create {} or {}.\n\nExample:\n[llm.zai]\napi_key = \"your-api-key-here\"",
            config_path.display(),
            fallback_path.display()
        ).into());
    }

    let contents = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&contents)?;

    Ok(config)
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
            println!("# base_url = \"https://api.z.ai/api/paas/v4\"");
            println!("# coding_url = \"https://api.z.ai/api/coding/paas/v4\"");
            println!("# model = \"glm-5\"");
            return Ok(());
        }
    };

    let zai_config = &config.llm.zai;
    let api_key = &zai_config.api_key;
    let endpoint = zai_config.base_url.as_deref()
        .unwrap_or("https://api.z.ai/api/paas/v4");

    println!("Using endpoint: {}", endpoint);

    let doc_path = "document.md";

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
