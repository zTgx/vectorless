// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Indexer module - Document parsing and indexing pipeline.
//!
//! This module provides functionality for parsing documents (PDF, Markdown),
//! detecting table of contents, building tree structures, and generating summaries.

pub mod config;
pub mod parse;
pub mod pdf;
pub mod markdown;
pub mod toc;
pub mod summary;

// Re-exports for public API
pub use config::{IndexerConfig, IndexerConfigBuilder};
pub use parse::{parse_document, parse_document_with_config};
pub use pdf::{
    Page,
    PdfDocument,
    PdfExtractor,
    PdfParser,
    TokenStrategy,
    estimate_tokens,
    mark_page_boundaries,
    parse_page_spec,
};
pub use markdown::{
    parse_markdown,
    parse_markdown_with_config,
    MdConfig,
    MdConfigBuilder,
    MdParseResult,
};
pub use toc::{
    TocEntry,
    TocResult,
    TocProcessor,
    TocConfig,
    TocConfigBuilder,
};
pub use summary::{build_summaries, build_summaries_with_config};

// Error types
pub use parse::Error as ParseError;
pub use pdf::Error as PdfError;
pub use markdown::Error as MdError;
pub use toc::Error as TocError;
pub use summary::Error as IndexError;
