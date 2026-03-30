// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # vectorless-core
//!
//! Core library for document parsing, indexing, and retrieval.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]

// ============================================================
// Core Modules
// ============================================================

pub mod node;
pub mod indexer;
pub mod retriever;
pub mod storage;
pub mod document;
pub mod client;

// ============================================================
// Re-exports - Node
// ============================================================

pub use node::{PageNode, PageNodeRef, PageNodeRefExt};

// ============================================================
// Re-exports - Indexer
// ============================================================

pub use indexer::{
    // Config
    IndexerConfig,
    IndexerConfigBuilder,
    // Parse
    parse_document,
    parse_document_with_config,
    // PDF
    Page,
    PdfDocument,
    PdfExtractor,
    PdfParser,
    TokenStrategy,
    estimate_tokens,
    mark_page_boundaries,
    parse_page_spec,
    // Markdown
    parse_markdown,
    parse_markdown_with_config,
    MdConfig,
    MdConfigBuilder,
    MdParseResult,
    // TOC
    TocEntry,
    TocResult,
    TocProcessor,
    TocConfig,
    TocConfigBuilder,
    // Summary
    build_summaries,
    build_summaries_with_config,
};

// ============================================================
// Re-exports - Retriever
// ============================================================

pub use retriever::{
    // Tree
    TreeBuilder,
    extract_page_number,
    extract_page_range,
    extract_page_range_with_boundaries,
    find_node_for_page,
    collect_nodes_in_page_range,
    get_path_to_node,
    validate_page_boundaries,
    ValidationError,
    // Navigate
    retrieve as retrieve_simple,
    // Retrieve
    RetrieveMode,
    RetrieveResult,
    RetrievedSection,
    PathStep,
    RetrieveMetadata,
    retrieve_with_mode,
};

// ============================================================
// Re-exports - Storage
// ============================================================

pub use storage::{
    save,
    load,
    Workspace,
    DocumentCache,
};

// ============================================================
// Re-exports - Document
// ============================================================

pub use document::{
    Document,
    DocumentType,
    DocumentStatus,
    DocumentSummary,
    CachedPage,
    StructureNodeDto,
    DocumentMetadata,
    MetaEntry,
    parse_page_range,
    get_document,
    get_document_structure,
    get_page_content,
    to_structure_dto,
};

// ============================================================
// Re-exports - Client
// ============================================================

pub use client::{
    DocumentCollection,
    IndexMode,
};

// ============================================================
// Error Types
// ============================================================

pub use indexer::ParseError;
pub use indexer::PdfError;
pub use indexer::MdError;
pub use indexer::TocError;
pub use indexer::IndexError;
pub use retriever::RetrieverError;
pub use retriever::RetrieveError;
pub use storage::StorageError;
pub use storage::WorkspaceError;
pub use storage::CacheError;
pub use document::DocumentError;
pub use client::ClientError;
