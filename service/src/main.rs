// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! vectorless HTTP service

use std::path::PathBuf;
use vectorless_llm::zai::ZaiClient;
use vectorless_core::IndexerConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Load configuration
    let config = load_config()?;

    tracing::info!("Starting vectorless service...");
    tracing::info!("Data directory: {}", config.data_dir.display());

    // Create directories
    std::fs::create_dir_all(&config.data_dir)?;
    std::fs::create_dir_all(&config.index_dir)?;

    // Initialize repositories
    let db_path = config.data_dir.join("metadata.db");
    let metadata_repository = vectorless_service::MetadataRepository::open(&db_path)?;
    let index_repository = vectorless_service::IndexRepository::new(&config.index_dir);

    // Initialize LLM client
    let llm = ZaiClient::with_endpoint(&config.api_key, &config.endpoint);

    // Initialize indexer config
    let indexer_config = IndexerConfig::builder()
        .subsection_threshold(config.subsection_threshold)
        .max_segment_tokens(config.max_segment_tokens)
        .summary_model(&config.model)
        .max_summary_tokens(config.max_summary_tokens)
        .build();

    // Create app state
    let state = vectorless_service::controllers::AppState {
        llm,
        metadata_repository,
        index_repository,
        indexer_config,
    };

    // Start server
    vectorless_service::run_server(state, &config.host, config.port).await
}

/// Server configuration.
struct Config {
    host: String,
    port: u16,
    data_dir: PathBuf,
    index_dir: PathBuf,
    api_key: String,
    endpoint: String,
    model: String,
    subsection_threshold: usize,
    max_segment_tokens: usize,
    max_summary_tokens: u32,
}

/// Load configuration from environment or defaults.
fn load_config() -> Result<Config, anyhow::Error> {
    // Try to load from .config.toml
    let api_key = std::env::var("ZAI_API_KEY")
        .unwrap_or_else(|_| "".to_string());

    let endpoint = std::env::var("ZAI_ENDPOINT")
        .unwrap_or_else(|_| "https://api.z.ai/api/paas/v4".to_string());

    let model = std::env::var("ZAI_MODEL")
        .unwrap_or_else(|_| "glm-5".to_string());

    Ok(Config {
        host: std::env::var("SERVICE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
        port: std::env::var("SERVICE_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()?,
        data_dir: std::env::var("SERVICE_DATA_DIR")
            .unwrap_or_else(|_| "./data".to_string())
            .into(),
        index_dir: std::env::var("SERVICE_INDEX_DIR")
            .unwrap_or_else(|_| "./indices".to_string())
            .into(),
        api_key,
        endpoint,
        model,
        subsection_threshold: std::env::var("SERVICE_SUBSECTION_THRESHOLD")
            .unwrap_or_else(|_| "200".to_string())
            .parse()
            .unwrap_or(200),
        max_segment_tokens: std::env::var("SERVICE_MAX_SEGMENT_TOKENS")
            .unwrap_or_else(|_| "4000".to_string())
            .parse()
            .unwrap_or(4000),
        max_summary_tokens: std::env::var("SERVICE_MAX_SUMMARY_TOKENS")
            .unwrap_or_else(|_| "200".to_string())
            .parse()
            .unwrap_or(200),
    })
}
