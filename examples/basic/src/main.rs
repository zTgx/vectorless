// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Basic usage example for vectorless.

use std::path::Path;
use vectorless_core::{
    build_summaries_with_config, load, parse_document_with_config, retrieve, save,
    IndexerConfig, PageNodeRef,
};
use vectorless_llm::chat::{ChatModel, Message, Role, ChatOptions};

// A mock LLM implementation for demonstration
struct MockLlm;

#[async_trait::async_trait]
impl ChatModel for MockLlm {
    async fn chat(
        &self,
        _messages: &[Message],
        _options: &ChatOptions,
    ) -> Result<vectorless_llm::chat::ChatCompletion, vectorless_llm::chat::Error> {
        // Mock implementation - in real usage, replace with actual LLM client
        Ok(vectorless_llm::chat::ChatCompletion {
            content: "1".to_string(), // Return first child index
            finish_reason: Some("stop".to_string()),
        })
    }
}

/// Build the index from a document with custom config.
async fn build_index(doc_path: &str) -> Result<PageNodeRef, Box<dyn std::error::Error>> {
    println!("Parsing document...");
    let text = std::fs::read_to_string(doc_path)?;

    // Use custom config for better quality
    let config = IndexerConfig::builder()
        .subsection_threshold(200)  // More granular splitting
        .max_segment_tokens(4000)    // More context for segmentation
        .summary_model("gpt-4")  // Stronger model for summaries
        .max_summary_tokens(200)     // Longer summaries
        .build();

    let llm = MockLlm;
    let tree = parse_document_with_config(&llm, &text, &config).await?;

    println!("Building summaries (this makes LLM calls)...");
    build_summaries_with_config(&llm, &tree, &config).await?;

    let index_path = "index.json";
    println!("Saving index to {}", index_path);
    save(&tree, index_path)?;

    Ok(tree)
}

/// Or build with default config
async fn build_index_default(doc_path: &str) -> Result<PageNodeRef, Box<dyn std::error::Error>> {
    println!("Parsing document with default config...");
    let text = std::fs::read_to_string(doc_path)?;
    let llm = MockLlm;
    let tree = parse_document_with_config(&llm, &text, &IndexerConfig::default()).await?;

    println!("Building summaries...");
    build_summaries_with_config(&llm, &tree, &IndexerConfig::default()).await?;

    let index_path = "index.json";
    println!("Saving index to {}", index_path);
    save(&tree, index_path)?;

    Ok(tree)
}

/// Query the index and generate an answer.
async fn ask(query: &str) -> Result<String, Box<dyn std::error::Error>> {
    let index_path = "index.json";

    if !Path::new(index_path).exists() {
        return Err("Index not found. Run build_index() first.".into());
    }

    let tree = load(index_path)?;
    let llm = MockLlm;
    let context = retrieve(&llm, query, &tree).await?;

    let prompt = format!(
        "Answer using only the context below.\n\nContext:\n{}\n\nQuestion: {}",
        context, query
    );

    let response = llm
        .chat(
            &[Message {
                role: Role::User,
                content: prompt,
            }],
            &ChatOptions {
                temperature: Some(0.0),
                max_tokens: Some(500),
            },
        )
        .await?;

    Ok(response.content.trim().to_string())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // First time: build the index
    let doc_path = "document.md";
    if Path::new(doc_path).exists() {
        build_index(doc_path).await?;
    } else {
        println!("Document file '{}' not found. Skipping index build.", doc_path);
    }

    // Then query it
    let query = "Your Question";
    match ask(query).await {
        Ok(answer) => println!("Answer: {}", answer),
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}
