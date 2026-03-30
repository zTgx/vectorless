// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # vectorless-core
//!
//! Core library for document parsing, indexing, and retrieval.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]

pub mod node;
pub mod parse;
pub mod index;
pub mod storage;
pub mod retriever;
pub mod config;
pub mod markdown;
pub mod pdf;

pub use node::{PageNode, PageNodeRef};
pub use parse::{parse_document, parse_document_with_config, Error as ParseError};
pub use index::{build_summaries, build_summaries_with_config, Error as IndexError};
pub use storage::{save, load, Error as StorageError};
pub use retriever::{retrieve, Error as RetrieverError};
pub use config::{IndexerConfig, IndexerConfigBuilder};
pub use markdown::{
    parse_markdown,
    parse_markdown_with_config,
    MdConfig,
    MdConfigBuilder,
    MdParseResult,
    Error as MdError,
};
pub use pdf::{
    Page,
    PdfDocument,
    PdfParser,
    PdfExtractor,
    TokenStrategy,
    estimate_tokens,
    mark_page_boundaries,
    parse_page_spec,
    Error as PdfError,
};
