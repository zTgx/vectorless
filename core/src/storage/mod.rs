// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Storage module - Persistence and caching for document data.
//!
//! This module provides functionality for saving and loading document
//! trees, managing workspaces, and caching frequently accessed data.

pub mod persistence;
pub mod workspace;
pub mod cache;

// Re-exports for public API
pub use persistence::{save, load};
pub use workspace::Workspace;
pub use cache::DocumentCache;

// Error types
pub use persistence::Error as StorageError;
pub use workspace::Error as WorkspaceError;
pub use cache::Error as CacheError;
