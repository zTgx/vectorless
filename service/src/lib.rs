// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # vectorless-service
//!
//! HTTP service layer for vectorless - provides REST API for document indexing and RAG queries.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]

pub mod dto;
pub mod repository;
pub mod service;
pub mod controllers;
pub mod middleware;
pub mod server;

pub use dto::{Document, DocumentStatus, QueryRequest, QueryResponse, ApiError};
pub use repository::{MetadataRepository, IndexRepository};
pub use service::{IngestService, QueryService, QueryResult};
pub use middleware::{ApiKeyAuth, CorsConfig, request_logging, require_api_key};
pub use server::run_server;
