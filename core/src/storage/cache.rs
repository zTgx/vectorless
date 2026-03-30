// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Cache management for document structures and pages.
//!
//! This module provides functionality for caching frequently accessed
//! document data to improve performance.

use crate::document::StructureNodeDto;
use serde_json;
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};

/// Cache for document structures and pages.
pub struct DocumentCache {
    /// Path to the cache directory
    cache_dir: PathBuf,

    /// In-memory cache for structure data
    structure_cache: HashMap<String, StructureNodeDto>,

    /// Maximum number of items to keep in memory
    max_memory_items: usize,
}

impl DocumentCache {
    /// Create a new document cache.
    pub fn new<P: AsRef<Path>>(cache_dir: P, max_memory_items: usize) -> Self {
        Self {
            cache_dir: cache_dir.as_ref().to_path_buf(),
            structure_cache: HashMap::new(),
            max_memory_items,
        }
    }

    /// Initialize the cache directory.
    pub fn init(&self) -> Result<(), Error> {
        std::fs::create_dir_all(&self.cache_dir)?;
        Ok(())
    }

    /// Get the cache path for a document.
    fn cache_path(&self, doc_id: &str) -> PathBuf {
        self.cache_dir.join(format!("{}.json", doc_id))
    }

    /// Put structure data in the cache.
    pub fn put_structure(&mut self, doc_id: &str, structure: &StructureNodeDto) -> Result<(), Error> {
        // Save to disk
        let path = self.cache_path(doc_id);
        let json = serde_json::to_string_pretty(structure)
            .map_err(|e| Error::Json(e.to_string()))?;
        std::fs::write(&path, json).map_err(|e| Error::Io(e))?;

        // Update in-memory cache
        self.structure_cache.insert(doc_id.to_string(), structure.clone());

        // Evict if necessary
        if self.structure_cache.len() > self.max_memory_items {
            // Simple FIFO eviction
            if let Some(key) = self.structure_cache.keys().next().cloned() {
                self.structure_cache.remove(&key);
            }
        }

        Ok(())
    }

    /// Get structure data from the cache.
    pub fn get_structure(&mut self, doc_id: &str) -> Result<Option<StructureNodeDto>, Error> {
        // Check in-memory cache first
        if let Some(structure) = self.structure_cache.get(doc_id) {
            return Ok(Some(structure.clone()));
        }

        // Check disk cache
        let path = self.cache_path(doc_id);
        if !path.exists() {
            return Ok(None);
        }

        let json = std::fs::read_to_string(&path).map_err(|e| Error::Io(e))?;
        let structure: StructureNodeDto = serde_json::from_str(&json)
            .map_err(|e| Error::Json(e.to_string()))?;

        // Add to in-memory cache
        self.structure_cache.insert(doc_id.to_string(), structure.clone());

        Ok(Some(structure))
    }

    /// Remove a document from the cache.
    pub fn remove(&mut self, doc_id: &str) -> Result<(), Error> {
        // Remove from memory
        self.structure_cache.remove(doc_id);

        // Remove from disk
        let path = self.cache_path(doc_id);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| Error::Io(e))?;
        }

        Ok(())
    }

    /// Clear all cached data.
    pub fn clear(&mut self) -> Result<(), Error> {
        // Clear memory
        self.structure_cache.clear();

        // Clear disk cache
        if self.cache_dir.exists() {
            for entry in std::fs::read_dir(&self.cache_dir).map_err(|e| Error::Io(e))? {
                let entry = entry.map_err(|e| Error::Io(e))?;
                std::fs::remove_file(entry.path()).map_err(|e| Error::Io(e))?;
            }
        }

        Ok(())
    }
}

/// Cache error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(String),
}
