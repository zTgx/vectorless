// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! High-level client API for document indexing and retrieval.
//!
//! This module provides a `DocumentCollection` that manages multiple documents
//! with workspace persistence and lazy loading, similar to PageIndex's client.py.
//!
//! # Example
//!
//! ```no_run
//! use vectorless_core::client::DocumentCollection;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create collection with workspace persistence
//! let mut collection = DocumentCollection::with_workspace("./docs")?;
//!
//! // Index documents
//! let pdf_id = collection.index("./manual.pdf").await?;
//! let md_id = collection.index("./README.md").await?;
//!
//! // Query metadata
//! let meta = collection.get_document(&pdf_id);
//! println!("{}", meta);
//!
//! // Get document structure (triggers lazy load if needed)
//! let structure = collection.get_document_structure(&pdf_id);
//! println!("{}", structure);
//!
//! // Get page content for specific pages
//! let content = collection.get_page_content(&pdf_id, "5-7");
//! println!("{}", content);
//! # Ok(())
//! # }
//! ```

use crate::document::{Document, DocumentType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

// ============================================================
// Constants
// ============================================================

const META_FILE: &str = "_meta.json";

// ============================================================
// Core Structures
// ============================================================

/// Metadata entry for _meta.json (lightweight, no structure/pages).
///
/// This represents the minimal information stored in the registry
/// to enable quick lookups without loading full document content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaEntry {
    /// Document ID.
    pub id: String,

    /// Document type ("pdf" or "markdown").
    #[serde(rename = "type")]
    pub doc_type: String,

    /// Document name (filename).
    pub doc_name: String,

    /// Document description (LLM-generated).
    pub doc_description: String,

    /// Absolute path to the document file.
    pub path: PathBuf,

    /// Page count (for PDF documents).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_count: Option<usize>,

    /// Line count (for Markdown documents).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_count: Option<usize>,
}

/// Index mode for auto-detection or explicit format specification.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IndexMode {
    /// Auto-detect format from file extension.
    Auto,

    /// Force PDF indexing.
    Pdf,

    /// Force Markdown indexing.
    Markdown,
}

/// Main client for document indexing and retrieval.
///
/// Manages a collection of documents with optional workspace persistence.
/// Documents are stored with lightweight metadata in memory, and full content
/// (structure, pages) is lazy-loaded from disk when needed.
pub struct DocumentCollection {
    /// In-memory document registry (metadata only, structure/pages lazy-loaded).
    documents: HashMap<String, Document>,

    /// Workspace directory for persistence.
    workspace: Option<PathBuf>,

    /// LLM model to use for indexing operations.
    model: String,

    /// LLM model for retrieval queries.
    retrieve_model: String,
}

// ============================================================
// DocumentCollection Implementation
// ============================================================

impl DocumentCollection {
    /// Create a new document collection without workspace persistence.
    ///
    /// Documents are stored in memory only and will be lost when the
    /// collection is dropped. Use `with_workspace` for persistence.
    ///
    /// # Example
    ///
    /// ```
    /// use vectorless_core::client::DocumentCollection;
    ///
    /// let collection = DocumentCollection::new();
    /// ```
    pub fn new() -> Self {
        Self {
            documents: HashMap::new(),
            workspace: None,
            model: "gpt-4".to_string(),
            retrieve_model: "gpt-4".to_string(),
        }
    }

    /// Create a new document collection with workspace persistence.
    ///
    /// The workspace directory will be created if it doesn't exist.
    /// Document metadata and content will be persisted to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the workspace cannot be created.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vectorless_core::client::DocumentCollection;
    ///
    /// let collection = DocumentCollection::with_workspace("./docs").unwrap();
    /// ```
    pub fn with_workspace(workspace: impl AsRef<Path>) -> Result<Self, Error> {
        let workspace = workspace.as_ref();
        fs::create_dir_all(workspace)?;

        let mut collection = Self {
            documents: HashMap::new(),
            workspace: Some(workspace.to_path_buf()),
            model: "gpt-4".to_string(),
            retrieve_model: "gpt-4".to_string(),
        };

        // Load existing workspace
        collection.load_workspace()?;

        Ok(collection)
    }

    /// Set the LLM model for indexing operations.
    ///
    /// # Example
    ///
    /// ```
    /// use vectorless_core::client::DocumentCollection;
    ///
    /// let collection = DocumentCollection::new()
    ///     .with_model("gpt-4o");
    /// ```
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        let model = model.into();
        self.retrieve_model = model.clone();
        self.model = model;
        self
    }

    /// Set the LLM model for retrieval queries (separate from indexing model).
    pub fn with_retrieve_model(mut self, model: impl Into<String>) -> Self {
        self.retrieve_model = model.into();
        self
    }

    /// Index a document file (PDF or Markdown) and return doc_id.
    ///
    /// Automatically detects the document type from the file extension.
    /// For explicit type specification, use `index_with_mode`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file doesn't exist
    /// - The file format is not supported
    /// - LLM processing fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use vectorless_core::client::DocumentCollection;
    ///
    /// let mut collection = DocumentCollection::new();
    /// let doc_id = collection.index("./document.pdf").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn index(&mut self, file_path: impl AsRef<Path>) -> Result<String, Error> {
        self.index_with_mode(file_path, IndexMode::Auto).await
    }

    /// Index a document with explicit mode specification.
    pub async fn index_with_mode(
        &mut self,
        file_path: impl AsRef<Path>,
        mode: IndexMode,
    ) -> Result<String, Error> {
        let file_path = file_path.as_ref();

        // Resolve to absolute path
        let abs_path = file_path.canonicalize()
            .map_err(|e| Error::Io(e))?;

        // Check file exists
        if !abs_path.exists() {
            return Err(Error::DocumentNotFound(abs_path.display().to_string()));
        }

        // Determine document type
        let doc_type = match mode {
            IndexMode::Auto => detect_document_type(&abs_path)?,
            IndexMode::Pdf => DocumentType::Pdf,
            IndexMode::Markdown => DocumentType::Markdown,
        };

        // Dispatch to appropriate index method
        match doc_type {
            DocumentType::Pdf => self.index_pdf(&abs_path).await,
            DocumentType::Markdown => self.index_markdown(&abs_path).await,
        }
    }

    /// Get document metadata as a JSON string.
    ///
    /// Returns a JSON object with doc_id, doc_name, doc_description,
    /// type, status, and page_count or line_count.
    ///
    /// If the document is not found, returns a JSON object with an error field.
    pub fn get_document(&self, doc_id: &str) -> String {
        crate::document::get_document(&self.documents, doc_id)
    }

    /// Get document structure without text fields (triggers lazy load if needed).
    ///
    /// Returns a JSON representation of the document tree with all text
    /// fields removed to save tokens.
    pub fn get_document_structure(&mut self, doc_id: &str) -> String {
        // Ensure loaded first
        if let Ok(_) = self.ensure_loaded(doc_id) {
            crate::document::get_document_structure(&self.documents, doc_id)
        } else {
            crate::document::get_document_structure(&self.documents, doc_id)
        }
    }

    /// Get page content for specific pages (triggers lazy load if needed).
    ///
    /// Pages format: "5-7", "3,8", or "12"
    /// For PDF: physical page numbers (1-indexed)
    /// For Markdown: line numbers corresponding to node headers
    pub fn get_page_content(&mut self, doc_id: &str, pages: &str) -> String {
        // Ensure loaded first
        let _ = self.ensure_loaded(doc_id);
        crate::document::get_page_content(&self.documents, doc_id, pages)
    }

    /// List all document IDs in the collection.
    pub fn list_documents(&self) -> Vec<String> {
        self.documents.keys().cloned().collect()
    }

    /// Remove a document from the collection.
    ///
    /// If workspace is configured, also removes the document file from disk.
    pub fn remove_document(&mut self, doc_id: &str) -> Result<(), Error> {
        // Remove from memory
        self.documents.remove(doc_id);

        // Remove from workspace
        if let Some(ref workspace) = self.workspace {
            let doc_file = workspace.join(format!("{}.json", doc_id));
            if doc_file.exists() {
                fs::remove_file(doc_file)?;
            }

            // Update meta
            self.remove_from_meta(doc_id)?;
        }

        Ok(())
    }

    /// Reload workspace from disk.
    ///
    /// Loads all document metadata from _meta.json.
    /// Full content (structure/pages) is lazy-loaded on demand.
    pub fn load_workspace(&mut self) -> Result<(), Error> {
        // Ensure workspace is configured
        self.workspace.as_ref()
            .ok_or(Error::NoWorkspace)?;

        // Load meta
        let meta = self.load_meta()?;

        // Create lightweight documents from meta
        for (doc_id, entry) in meta {
            let doc = Document {
                id: doc_id.clone(),
                doc_type: if entry.doc_type == "pdf" {
                    DocumentType::Pdf
                } else {
                    DocumentType::Markdown
                },
                doc_name: entry.doc_name,
                doc_description: entry.doc_description,
                file_path: entry.path,
                page_count: entry.page_count,
                line_count: entry.line_count,
                status: crate::document::DocumentStatus::Completed,
                created_at: None,  // Will be loaded from full doc on demand
                modified_at: None,
                root: None,  // Will be lazy-loaded
                pages: None, // Will be lazy-loaded
            };
            self.documents.insert(doc_id, doc);
        }

        Ok(())
    }

    // ============================================================
    // Private Methods
    // ============================================================

    /// Index a PDF document.
    async fn index_pdf(&mut self, file_path: &Path) -> Result<String, Error> {
        let doc_id = self.generate_doc_id();

        // For PDF, we need to extract pages first
        // Note: This is a simplified version - full implementation would use actual PDF parsing
        let content = fs::read_to_string(file_path)
            .unwrap_or_else(|_| String::new());

        // Generate document description (simplified)
        let doc_description = if !content.is_empty() {
            format!("PDF document with {} bytes", content.len())
        } else {
            "PDF document".to_string()
        };

        // Create document with placeholder structure
        // In full implementation, this would call PDF parsing and TOC detection
        let now = chrono::Utc::now().to_rfc3339();
        let doc = Document {
            id: doc_id.clone(),
            doc_type: DocumentType::Pdf,
            doc_name: file_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            doc_description,
            file_path: file_path.to_path_buf(),
            page_count: None,  // Would be extracted from PDF
            line_count: None,
            status: crate::document::DocumentStatus::Completed,
            created_at: Some(now.clone()),
            modified_at: Some(now),
            root: None,        // Would be built from TOC
            pages: None,       // Would be extracted pages
        };

        self.documents.insert(doc_id.clone(), doc);

        if self.workspace.is_some() {
            self.save_document(&doc_id)?;
        }

        Ok(doc_id)
    }

    /// Index a Markdown document.
    async fn index_markdown(&mut self, file_path: &Path) -> Result<String, Error> {
        let doc_id = self.generate_doc_id();

        // Read file content
        let content = fs::read_to_string(file_path)?;

        // Get line count
        let line_count = content.lines().count();

        // Generate document description
        let doc_description = format!("Markdown document with {} lines", line_count);

        // Create document
        let now = chrono::Utc::now().to_rfc3339();
        let doc = Document {
            id: doc_id.clone(),
            doc_type: DocumentType::Markdown,
            doc_name: file_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            doc_description,
            file_path: file_path.to_path_buf(),
            page_count: None,
            line_count: Some(line_count),
            status: crate::document::DocumentStatus::Completed,
            created_at: Some(now.clone()),
            modified_at: Some(now),
            root: None,  // Would be built from markdown parser
            pages: None,
        };

        self.documents.insert(doc_id.clone(), doc);

        if self.workspace.is_some() {
            self.save_document(&doc_id)?;
        }

        Ok(doc_id)
    }

    /// Ensure document's full content (structure/pages) is loaded.
    fn ensure_loaded(&mut self, doc_id: &str) -> Result<(), Error> {
        // Check if already loaded
        if let Some(doc) = self.documents.get(doc_id) {
            if doc.root.is_some() {
                return Ok(());
            }
        }

        // Load from workspace
        let workspace = self.workspace.as_ref()
            .ok_or(Error::NoWorkspace)?;

        let doc_file = workspace.join(format!("{}.json", doc_id));
        if !doc_file.exists() {
            return Err(Error::DocumentNotFound(doc_id.to_string()));
        }

        // Read full document
        let content = fs::read_to_string(&doc_file)?;
        let full_doc: Document = serde_json::from_str(&content)?;

        // Update the in-memory document with loaded content
        if let Some(doc) = self.documents.get_mut(doc_id) {
            doc.root = full_doc.root;
            doc.pages = full_doc.pages;
        }

        Ok(())
    }

    /// Save document to workspace (both full JSON and meta entry).
    fn save_document(&self, doc_id: &str) -> Result<(), Error> {
        let workspace = self.workspace.as_ref()
            .ok_or(Error::NoWorkspace)?;

        let doc = self.documents.get(doc_id)
            .ok_or_else(|| Error::DocumentNotFound(doc_id.to_string()))?;

        // Save full document
        let doc_file = workspace.join(format!("{}.json", doc_id));
        let json = serde_json::to_string_pretty(doc)?;
        fs::write(&doc_file, json)?;

        // Update meta
        let meta_entry = MetaEntry {
            id: doc_id.to_string(),
            doc_type: if doc.doc_type == DocumentType::Pdf { "pdf".to_string() } else { "markdown".to_string() },
            doc_name: doc.doc_name.clone(),
            doc_description: doc.doc_description.clone(),
            path: doc.file_path.clone(),
            page_count: doc.page_count,
            line_count: doc.line_count,
        };

        self.update_meta(doc_id, &meta_entry)?;

        Ok(())
    }

    /// Update _meta.json with document metadata.
    fn update_meta(&self, doc_id: &str, entry: &MetaEntry) -> Result<(), Error> {
        let workspace = self.workspace.as_ref()
            .ok_or(Error::NoWorkspace)?;

        let meta_path = workspace.join(META_FILE);

        // Load existing meta or create new
        let mut meta = self.load_meta().unwrap_or_default();

        // Update entry
        meta.insert(doc_id.to_string(), entry.clone());

        // Write back
        let json = serde_json::to_string_pretty(&meta)?;
        fs::write(&meta_path, json)?;

        Ok(())
    }

    /// Remove document from _meta.json.
    fn remove_from_meta(&self, doc_id: &str) -> Result<(), Error> {
        let workspace = self.workspace.as_ref()
            .ok_or(Error::NoWorkspace)?;

        let meta_path = workspace.join(META_FILE);

        if !meta_path.exists() {
            return Ok(());
        }

        // Load existing meta
        let mut meta = self.load_meta()?;

        // Remove entry
        meta.remove(doc_id);

        // Write back
        let json = serde_json::to_string_pretty(&meta)?;
        fs::write(&meta_path, json)?;

        Ok(())
    }

    /// Load _meta.json from workspace.
    fn load_meta(&self) -> Result<HashMap<String, MetaEntry>, Error> {
        let workspace = self.workspace.as_ref()
            .ok_or(Error::NoWorkspace)?;

        let meta_path = workspace.join(META_FILE);

        if !meta_path.exists() {
            return Ok(HashMap::new());
        }

        let content = fs::read_to_string(&meta_path)?;
        let meta: HashMap<String, MetaEntry> = serde_json::from_str(&content)?;

        Ok(meta)
    }

    /// Generate a new unique document ID.
    fn generate_doc_id(&self) -> String {
        Uuid::new_v4().to_string()
    }
}

impl Default for DocumentCollection {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Helper Functions
// ============================================================

/// Detect document type from file extension.
fn detect_document_type(path: &Path) -> Result<DocumentType, Error> {
    let ext = path.extension()
        .and_then(|e| e.to_str())
        .ok_or(Error::UnknownFormat)?;

    match ext.to_lowercase().as_str() {
        "pdf" => Ok(DocumentType::Pdf),
        "md" | "markdown" => Ok(DocumentType::Markdown),
        _ => Err(Error::UnknownFormat),
    }
}

// ============================================================
// Error Types
// ============================================================

/// Client API error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Document not found.
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    /// Workspace not configured.
    #[error("Workspace not configured")]
    NoWorkspace,

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Unsupported document format.
    #[error("Unsupported document format")]
    UnknownFormat,

    /// Parsing failed.
    #[error("Parsing failed: {0}")]
    ParseFailed(String),

    /// Indexing failed.
    #[error("Indexing failed: {0}")]
    IndexFailed(String),
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_collection_new() {
        let _collection = DocumentCollection::new();
        let collection = DocumentCollection::new();
        assert!(collection.workspace.is_none());
        assert!(collection.documents.is_empty());
    }

    #[test]
    fn test_collection_with_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let collection = DocumentCollection::with_workspace(temp_dir.path()).unwrap();
        assert!(collection.workspace.is_some());
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_with_model() {
        let collection = DocumentCollection::new()
            .with_model("gpt-4o");
        assert_eq!(collection.model, "gpt-4o");
        assert_eq!(collection.retrieve_model, "gpt-4o");
    }

    #[test]
    fn test_with_retrieve_model() {
        let collection = DocumentCollection::new()
            .with_model("gpt-4")
            .with_retrieve_model("gpt-3.5-turbo");
        assert_eq!(collection.model, "gpt-4");
        assert_eq!(collection.retrieve_model, "gpt-3.5-turbo");
    }

    #[test]
    fn test_detect_pdf() {
        let path = PathBuf::from("/test/document.pdf");
        let doc_type = detect_document_type(&path).unwrap();
        assert_eq!(doc_type, DocumentType::Pdf);
    }

    #[test]
    fn test_detect_markdown() {
        let path = PathBuf::from("/test/README.md");
        let doc_type = detect_document_type(&path).unwrap();
        assert_eq!(doc_type, DocumentType::Markdown);

        let path = PathBuf::from("/test/doc.markdown");
        let doc_type = detect_document_type(&path).unwrap();
        assert_eq!(doc_type, DocumentType::Markdown);
    }

    #[test]
    fn test_detect_unknown() {
        let path = PathBuf::from("/test/document.txt");
        let result = detect_document_type(&path);
        assert!(matches!(result, Err(Error::UnknownFormat)));

        let path = PathBuf::from("/test/no_extension");
        let result = detect_document_type(&path);
        assert!(matches!(result, Err(Error::UnknownFormat)));
    }

    #[test]
    fn test_generate_doc_id() {
        let collection = DocumentCollection::new();
        let id1 = collection.generate_doc_id();
        let id2 = collection.generate_doc_id();
        assert_ne!(id1, id2);
        // UUID v4 format: 8-4-4-4-12 hex digits
        assert_eq!(id1.len(), 36);
        assert_eq!(id2.len(), 36);
    }

    #[test]
    fn test_list_documents_empty() {
        let collection = DocumentCollection::new();
        let docs = collection.list_documents();
        assert!(docs.is_empty());
    }

    #[test]
    fn test_get_document_not_found() {
        let collection = DocumentCollection::new();
        let result = collection.get_document("non-existent");
        assert!(result.contains("\"error\":"));
    }

    #[test]
    fn test_meta_entry_serialization() {
        let entry = MetaEntry {
            id: "test-id".to_string(),
            doc_type: "pdf".to_string(),
            doc_name: "Test.pdf".to_string(),
            doc_description: "A test document".to_string(),
            path: PathBuf::from("/test/Test.pdf"),
            page_count: Some(42),
            line_count: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"id\":\"test-id\""));
        assert!(json.contains("\"type\":\"pdf\""));
        assert!(json.contains("\"page_count\":42"));
        assert!(!json.contains("line_count"));
    }

    #[test]
    fn test_workspace_save_and_load_meta() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = temp_dir.path();

        // Add a document manually
        let doc_id = "test-doc-1";
        let doc = Document {
            id: doc_id.to_string(),
            doc_type: DocumentType::Pdf,
            doc_name: "Test.pdf".to_string(),
            doc_description: "Test".to_string(),
            file_path: PathBuf::from("/test/Test.pdf"),
            page_count: Some(10),
            line_count: None,
            status: crate::document::DocumentStatus::Completed,
            created_at: Some("2024-01-01T00:00:00Z".to_string()),
            modified_at: Some("2024-01-01T00:00:00Z".to_string()),
            root: None,
            pages: None,
        };

        let mut test_collection = DocumentCollection {
            documents: HashMap::new(),
            workspace: Some(workspace.to_path_buf()),
            model: "test".to_string(),
            retrieve_model: "test".to_string(),
        };

        test_collection.documents.insert(doc_id.to_string(), doc);

        // Save meta
        let entry = MetaEntry {
            id: doc_id.to_string(),
            doc_type: "pdf".to_string(),
            doc_name: "Test.pdf".to_string(),
            doc_description: "Test".to_string(),
            path: PathBuf::from("/test/Test.pdf"),
            page_count: Some(10),
            line_count: None,
        };

        test_collection.update_meta(doc_id, &entry).unwrap();

        // Verify meta file exists
        let meta_path = workspace.join(META_FILE);
        assert!(meta_path.exists());

        // Load meta
        let loaded = test_collection.load_meta().unwrap();
        assert_eq!(loaded.len(), 1);
        assert!(loaded.contains_key(doc_id));
        assert_eq!(loaded[doc_id].doc_name, "Test.pdf");
    }

    #[test]
    fn test_remove_from_meta() {
        let temp_dir = TempDir::new().unwrap();
        let workspace = temp_dir.path();

        let mut collection = DocumentCollection {
            documents: HashMap::new(),
            workspace: Some(workspace.to_path_buf()),
            model: "test".to_string(),
            retrieve_model: "test".to_string(),
        };

        // Add meta entry
        let entry = MetaEntry {
            id: "test-doc".to_string(),
            doc_type: "pdf".to_string(),
            doc_name: "Test.pdf".to_string(),
            doc_description: "Test".to_string(),
            path: PathBuf::from("/test/Test.pdf"),
            page_count: Some(10),
            line_count: None,
        };

        collection.update_meta("test-doc", &entry).unwrap();

        // Verify it exists
        let loaded = collection.load_meta().unwrap();
        assert_eq!(loaded.len(), 1);

        // Remove it
        collection.remove_from_meta("test-doc").unwrap();

        // Verify it's gone
        let loaded = collection.load_meta().unwrap();
        assert_eq!(loaded.len(), 0);
    }
}
