// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Workspace management for document collections.
//!
//! This module provides functionality for managing a workspace directory
//! that stores document metadata and individual document files.

use crate::document::MetaEntry;
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::Path;

/// Workspace manager for document collections.
pub struct Workspace {
    /// Path to the workspace directory
    path: std::path::PathBuf,
}

impl Workspace {
    /// Create a new workspace manager.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Initialize the workspace directory.
    pub fn init(&self) -> Result<(), Error> {
        fs::create_dir_all(&self.path)?;
        Ok(())
    }

    /// Get the path to the metadata file (_meta.json).
    pub fn meta_path(&self) -> std::path::PathBuf {
        self.path.join("_meta.json")
    }

    /// Get the path to a document file.
    pub fn doc_path(&self, doc_id: &str) -> std::path::PathBuf {
        self.path.join(format!("{}.json", doc_id))
    }

    /// Save metadata entries to _meta.json.
    pub fn save_meta(&self, entries: &HashMap<String, MetaEntry>) -> Result<(), Error> {
        let path = self.meta_path();
        let json = serde_json::to_string_pretty(entries)
            .map_err(|e| Error::Json(e.to_string()))?;
        fs::write(&path, json).map_err(|e| Error::Io(e))?;
        Ok(())
    }

    /// Load metadata entries from _meta.json.
    pub fn load_meta(&self) -> Result<HashMap<String, MetaEntry>, Error> {
        let path = self.meta_path();
        if !path.exists() {
            return Ok(HashMap::new());
        }
        let json = fs::read_to_string(&path).map_err(|e| Error::Io(e))?;
        let entries: HashMap<String, MetaEntry> = serde_json::from_str(&json)
            .map_err(|e| Error::Json(e.to_string()))?;
        Ok(entries)
    }

    /// Save a document to its file.
    pub fn save_document(&self, doc_id: &str, content: &str) -> Result<(), Error> {
        let path = self.doc_path(doc_id);
        fs::write(&path, content).map_err(|e| Error::Io(e))?;
        Ok(())
    }

    /// Load a document from its file.
    pub fn load_document(&self, doc_id: &str) -> Result<String, Error> {
        let path = self.doc_path(doc_id);
        fs::read_to_string(&path).map_err(|e| Error::Io(e))
    }

    /// Check if a document file exists.
    pub fn document_exists(&self, doc_id: &str) -> bool {
        self.doc_path(doc_id).exists()
    }

    /// Delete a document file.
    pub fn delete_document(&self, doc_id: &str) -> Result<(), Error> {
        let path = self.doc_path(doc_id);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| Error::Io(e))?;
        }
        Ok(())
    }

    /// List all document IDs in the workspace.
    pub fn list_documents(&self) -> Result<Vec<String>, Error> {
        let mut ids = Vec::new();
        for entry in fs::read_dir(&self.path).map_err(|e| Error::Io(e))? {
            let entry = entry.map_err(|e| Error::Io(e))?;
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.ends_with(".json") && name_str != "_meta.json" {
                let id = name_str.strip_suffix(".json").unwrap_or(&name_str);
                ids.push(id.to_string());
            }
        }
        Ok(ids)
    }
}

/// Workspace error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("JSON error: {0}")]
    Json(String),
}
