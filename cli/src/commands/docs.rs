// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Docs command - Manage documents.

use clap::{Parser, Subcommand};

/// Manage documents in the knowledge base
#[derive(Parser, Debug)]
pub struct DocsArgs {
    #[command(subcommand)]
    command: DocsCommands,
}

#[derive(Subcommand, Debug)]
enum DocsCommands {
    /// List all documents
    List,
    /// Show document details
    Show {
        /// Document ID
        id: String,
    },
    /// Delete a document
    Delete {
        /// Document ID
        id: String,
    },
}

pub async fn run(args: DocsArgs) -> anyhow::Result<()> {
    match args.command {
        DocsCommands::List => {
            tracing::info!("📄 Document listing");
            tracing::warn!("⚠️  This will query the RAG service for documents");
        }
        DocsCommands::Show { id } => {
            tracing::info!("📄 Showing document: {}", id);
            tracing::warn!("⚠️  This will fetch document details from the RAG service");
        }
        DocsCommands::Delete { id } => {
            tracing::info!("🗑️  Deleting document: {}", id);
            tracing::warn!("⚠️  This will delete the document from the RAG service");
        }
    }

    tracing::info!("Make sure the RAG service is running on http://localhost:8080");

    Ok(())
}
