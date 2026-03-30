// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Data Transfer Objects (DTOs) for the service API.

pub mod document;
pub mod query;
pub mod error;

pub use document::{Document, DocumentStatus, CreateDocumentRequest, CreateDocumentResponse, UploadContentRequest};
pub use query::{QueryRequest, QueryResponse, Source};
pub use error::ApiError;
