// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Index file storage.

use crate::models::ApiError;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Index store for managing index files.
#[derive(Clone)]
pub struct IndexStore {
    base_dir: PathBuf,
}

impl IndexStore {
    /// Create a new index store.
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
        }
    }

    /// Get the index file path for a document.
    pub fn get_index_path(&self, id: Uuid) -> PathBuf {
        self.base_dir.join(format!("{}.json", id))
    }

    /// Save index to file.
    pub fn save_index(&self, id: Uuid, data: &[u8]) -> Result<(), ApiError> {
        let path = self.get_index_path(id);
        std::fs::write(&path, data)
            .map_err(|e| ApiError::Storage(format!("Failed to write index: {}", e)))?;
        Ok(())
    }

    /// Load index from file.
    pub fn load_index(&self, id: Uuid) -> Result<Option<Vec<u8>>, ApiError> {
        let path = self.get_index_path(id);
        match std::fs::read(&path) {
            Ok(data) => Ok(Some(data)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(ApiError::Storage(format!("Failed to read index: {}", e))),
        }
    }

    /// Delete index file.
    pub fn delete_index(&self, id: Uuid) -> Result<(), ApiError> {
        let path = self.get_index_path(id);
        std::fs::remove_file(&path)
            .map_err(|e| ApiError::Storage(format!("Failed to delete index: {}", e)))?;
        Ok(())
    }

    /// Check if index exists.
    pub fn index_exists(&self, id: Uuid) -> bool {
        self.get_index_path(id).exists()
    }
}
