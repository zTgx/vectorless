// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # vectorless
//!
//! A lightweight document indexing engine without vectorization.
//!
//! ## Overview
//!
//! `vectorless` provides efficient document indexing and search capabilities
//! without relying on vector embeddings. It uses traditional indexing techniques
//! such as inverted indices, tokenization, and BM25 ranking for fast and
//! accurate full-text search.
//!
//! ## Features
//!
//! - **Zero-dependency core**: No vector database required
//! - **Fast indexing**: Optimized for quick document ingestion
//! - **BM25 ranking**: Industry-standard relevance scoring
//! - **Memory efficient**: Designed for low-memory environments
//! - **Flexible tokenization**: Pluggable tokenizers for different languages
//!
//! ## Quick Start
//!
//! ```rust
//! use vectorless::{Engine, Document};
//!
//! fn main() {
//!     println!("hello, vectorless");
//!
//!     let mut engine = Engine::new();
//!     engine.add(Document::new(1, "Hello world"));
//!     engine.add(Document::new(2, "Goodbye world"));
//!
//!     let results = engine.search("world");
//!     for doc in results {
//!         println!("Found: {}", doc.content());
//!     }
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unsafe_code)]

/// Hello function that prints a greeting.
///
/// # Example
///
/// ```
/// use vectorless::hello;
///
/// hello(); // prints: hello, vectorless
/// ```
pub fn hello() {
    println!("hello, vectorless");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() {
        // Just verify it compiles and doesn't panic
        hello();
    }
}
