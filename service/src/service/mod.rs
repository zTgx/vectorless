// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Service layer for business logic.

pub mod ingest;
pub mod query;

pub use ingest::IngestService;
pub use query::{QueryService, QueryResult};
