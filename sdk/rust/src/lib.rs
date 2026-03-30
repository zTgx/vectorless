// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # vectorless-sdk-rs
//!
//! Rust SDK for the vectorless service HTTP API.
//!
//! This SDK provides a type-safe Rust client for interacting with the vectorless
//! document indexing and RAG query service.
//!
//! # Quick Start
//!
//! ```no_run
//! use vectorless_sdk_rs::{Client, ClientConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ClientConfig::builder()
//!         .base_url("http://localhost:8080")
//!         .api_key("your-api-key")
//!         .build();
//!
//!     let client = Client::new(config)?;
//!
//!     let health = client.health().await?;
//!     println!("Service status: {}", health.status);
//!
//!     let doc = client.create_document("My Document").await?;
//!     println!("Created document: {}", doc.id);
//!
//!     let response = client.query("What is this about?").await?;
//!     println!("Answer: {}", response.answer);
//!
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]

pub mod client;
pub mod types;
pub mod error;

// Re-exports
pub use client::{Client, ClientConfig};
pub use error::{Error, Result};
pub use types::*;

/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
