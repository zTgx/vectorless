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
pub mod toc;
pub mod tree_builder;
pub mod retrieve;
pub mod document;
pub mod client;

pub use node::{PageNode, PageNodeRef, PageNodeRefExt};
pub use parse::{parse_document, parse_document_with_config, Error as ParseError};
pub use index::{build_summaries, build_summaries_with_config, Error as IndexError};
pub use storage::{save, load, Error as StorageError};
pub use retriever::{retrieve as retrieve_simple, Error as RetrieverError};
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
pub use toc::{
    TocEntry,
    TocResult,
    TocProcessor,
    TocConfig,
    TocConfigBuilder,
    Error as TocError,
};
pub use tree_builder::{
    TreeBuilder,
    extract_page_number,
    extract_page_range,
    extract_page_range_with_boundaries,
    find_node_for_page,
    collect_nodes_in_page_range,
    get_path_to_node,
    validate_page_boundaries,
    ValidationError,
};
pub use retrieve::{
    RetrieveMode,
    RetrieveResult,
    RetrievedSection,
    PathStep,
    RetrieveMetadata,
    retrieve_with_mode,
    retrieve,
    Error as RetrieveError,
};
pub use document::{
    Document,
    DocumentType,
    DocumentSummary,
    CachedPage,
    StructureNodeDto,
    DocumentMetadata,
    parse_page_range,
    get_document,
    get_document_structure,
    get_page_content,
    to_structure_dto,
    Error as DocumentError,
};
pub use client::{
    DocumentCollection,
    IndexMode,
    MetaEntry,
    Error as ClientError,
};
