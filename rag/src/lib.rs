// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # vectorless-rag
//!
//! RAG service for vectorless - HTTP server.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]

pub mod models;
pub mod store;
pub mod pipeline;
pub mod handlers;
pub mod server;

pub use models::{Document, DocumentStatus, QueryRequest, QueryResponse, ApiError};
pub use server::run_server;
