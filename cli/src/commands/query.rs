// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Query command - Query the knowledge base.

use clap::Parser;
use std::path::PathBuf;
use vectorless_core::{storage::load, retriever::retrieve};
use vectorless_llm::zai::ZaiClient;

/// Query a document index
#[derive(Parser, Debug)]
pub struct QueryArgs {
    /// Path to the index file
    #[arg(short, long, default_value = "index.json")]
    index: PathBuf,

    /// Query string
    #[arg(short, long)]
    query: String,

    /// API key for LLM
    #[arg(long, env = "ZAI_API_KEY")]
    api_key: String,

    /// LLM endpoint
    #[arg(long, env = "ZAI_ENDPOINT", default_value = "https://api.z.ai/api/paas/v4")]
    endpoint: String,

    /// Model name
    #[arg(long, env = "ZAI_MODEL", default_value = "glm-5")]
    model: String,
}

pub async fn run(args: QueryArgs) -> anyhow::Result<()> {
    tracing::info!("Loading index from: {}", args.index.display());

    // Load index
    let root = load(&args.index)
        .map_err(|e| anyhow::anyhow!("Failed to load index: {}", e))?;

    // Initialize LLM client
    let llm = ZaiClient::with_options(&args.api_key, &args.model, &args.endpoint);

    // Query
    tracing::info!("Querying: {}", args.query);
    let result = retrieve(&llm, &args.query, &root)
        .await
        .map_err(|e| anyhow::anyhow!("Query failed: {}", e))?;

    println!("\n📝 Answer:\n{}", result);

    Ok(())
}
