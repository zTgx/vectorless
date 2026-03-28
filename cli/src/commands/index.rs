// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Index command - Index a document.

use clap::Parser;
use std::path::PathBuf;
use vectorless_core::{IndexerConfig, parse::parse_document_with_config, index::build_summaries_with_config, storage::save};
use vectorless_llm::zai::ZaiClient;

/// Index a document file
#[derive(Parser, Debug)]
pub struct IndexArgs {
    /// Path to the document file
    #[arg(short, long)]
    file: PathBuf,

    /// Output index file path
    #[arg(short, long, default_value = "index.json")]
    output: PathBuf,

    /// API key for LLM
    #[arg(long, env = "ZAI_API_KEY")]
    api_key: String,

    /// LLM endpoint
    #[arg(long, env = "ZAI_ENDPOINT", default_value = "https://api.z.ai/api/paas/v4")]
    endpoint: String,

    /// Model name
    #[arg(long, env = "ZAI_MODEL", default_value = "glm-5")]
    model: String,

    /// Subsection token threshold
    #[arg(long, default_value = "200")]
    subsection_threshold: usize,

    /// Max segment tokens
    #[arg(long, default_value = "4000")]
    max_segment_tokens: usize,

    /// Max summary tokens
    #[arg(long, default_value = "200")]
    max_summary_tokens: u32,
}

pub async fn run(args: IndexArgs) -> anyhow::Result<()> {
    tracing::info!("Reading document from: {}", args.file.display());

    // Read document content
    let content = tokio::fs::read_to_string(&args.file).await?;
    tracing::info!("Document size: {} bytes", content.len());

    // Initialize LLM client
    let llm = ZaiClient::with_options(&args.api_key, &args.model, &args.endpoint);

    // Configure indexer
    let config = IndexerConfig::builder()
        .subsection_threshold(args.subsection_threshold)
        .max_segment_tokens(args.max_segment_tokens)
        .summary_model(&args.model)
        .max_summary_tokens(args.max_summary_tokens)
        .build();

    // Parse document
    tracing::info!("Parsing document...");
    let root = parse_document_with_config(&llm, &content, &config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse document: {}", e))?;

    // Build summaries
    tracing::info!("Building summaries...");
    build_summaries_with_config(&llm, &root, &config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to build summaries: {}", e))?;

    // Save index
    tracing::info!("Saving index to: {}", args.output.display());
    save(&root, &args.output)
        .map_err(|e| anyhow::anyhow!("Failed to save index: {}", e))?;

    tracing::info!("✅ Index built successfully!");
    tracing::info!("Index saved to: {}", args.output.display());

    Ok(())
}
