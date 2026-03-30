// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Serve command - Start the HTTP server.

use clap::Parser;
use std::path::PathBuf;
use vectorless_llm::zai::ZaiClient;
use vectorless_service::{repository::MetadataRepository, repository::IndexRepository};

/// Start the HTTP service server
#[derive(Parser, Debug)]
pub struct ServeArgs {
    /// Host to bind to
    #[arg(short, long, env = "SERVICE_HOST", default_value = "0.0.0.0")]
    host: String,

    /// Port to bind to
    #[arg(short, long, env = "SERVICE_PORT", default_value = "8080")]
    port: u16,

    /// Data directory
    #[arg(long, env = "SERVICE_DATA_DIR", default_value = "./data")]
    data_dir: PathBuf,

    /// Index directory
    #[arg(long, env = "SERVICE_INDEX_DIR", default_value = "./indices")]
    index_dir: PathBuf,

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
    #[arg(long, env = "SERVICE_SUBSECTION_THRESHOLD", default_value = "200")]
    subsection_threshold: usize,

    /// Max segment tokens
    #[arg(long, env = "SERVICE_MAX_SEGMENT_TOKENS", default_value = "4000")]
    max_segment_tokens: usize,

    /// Max summary tokens
    #[arg(long, env = "SERVICE_MAX_SUMMARY_TOKENS", default_value = "200")]
    max_summary_tokens: u32,
}

pub async fn run(args: ServeArgs) -> anyhow::Result<()> {
    // Create directories
    std::fs::create_dir_all(&args.data_dir)?;
    std::fs::create_dir_all(&args.index_dir)?;

    // Initialize repositories
    let db_path = args.data_dir.join("metadata.db");
    let metadata_repository = MetadataRepository::open(&db_path)?;
    let index_repository = IndexRepository::new(&args.index_dir);

    // Initialize LLM client
    let llm = ZaiClient::with_endpoint(&args.api_key, &args.endpoint);

    // Initialize indexer config
    let indexer_config = vectorless_core::IndexerConfig::builder()
        .subsection_threshold(args.subsection_threshold)
        .max_segment_tokens(args.max_segment_tokens)
        .summary_model(&args.model)
        .max_summary_tokens(args.max_summary_tokens)
        .build();

    // Create app state
    let state = vectorless_service::controllers::AppState {
        llm,
        metadata_repository,
        index_repository,
        indexer_config,
    };

    // Start server
    tracing::info!("🚀 Starting vectorless service server...");
    vectorless_service::run_server(state, &args.host, args.port).await?;

    Ok(())
}
