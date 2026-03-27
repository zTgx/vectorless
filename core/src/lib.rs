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

pub use node::{PageNode, PageNodeRef};
pub use parse::{parse_document, Error as ParseError, SUBSECTION_THRESHOLD};
pub use index::{build_summaries, Error as IndexError};
pub use storage::{save, load, Error as StorageError};
pub use retriever::{retrieve, Error as RetrieverError};
