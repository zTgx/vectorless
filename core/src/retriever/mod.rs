// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Retriever module - Tree navigation and document retrieval.
//!
//! This module provides functionality for navigating document trees,
//! retrieving content by page range, and performing enhanced retrieval
//! with multiple strategies.

pub mod tree;
pub mod navigate;
pub mod retrieve;

// Re-exports for public API
pub use tree::{
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
pub use navigate::{retrieve as retrieve_simple};
pub use retrieve::{
    RetrieveMode,
    RetrieveResult,
    RetrievedSection,
    PathStep,
    RetrieveMetadata,
    retrieve_with_mode,
};

// Note: `retrieve` function is available via `retrieve::retrieve()`

// Error types
pub use navigate::Error as RetrieverError;
pub use retrieve::Error as RetrieveError;
