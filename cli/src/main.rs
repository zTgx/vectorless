// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! vectorless CLI - Command-line interface for vectorless.

use clap::{Parser, Subcommand};
use tracing_subscriber::EnvFilter;

mod commands;

#[derive(Parser)]
#[command(name = "vectorless")]
#[command(about = "RAG without vector embeddings", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Index a document
    Index(commands::index::IndexArgs),
    /// Query the knowledge base
    Query(commands::query::QueryArgs),
    /// Start the HTTP server
    Serve(commands::serve::ServeArgs),
    /// Run an agent
    Agent(commands::agent::AgentArgs),
    /// Manage documents
    Docs(commands::docs::DocsArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Index(args) => commands::index::run(args).await?,
        Commands::Query(args) => commands::query::run(args).await?,
        Commands::Serve(args) => commands::serve::run(args).await?,
        Commands::Agent(args) => commands::agent::run(args).await?,
        Commands::Docs(args) => commands::docs::run(args).await?,
    }

    Ok(())
}
