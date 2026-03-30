// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Metadata repository for storing document metadata.

use crate::dto::{ApiError, Document, DocumentStatus};
use sled::Db;
use uuid::Uuid;

/// Metadata repository for documents.
#[derive(Clone)]
pub struct MetadataRepository {
    db: Db,
}

impl MetadataRepository {
    /// Open or create the metadata repository.
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, ApiError> {
        let db = sled::open(path)
            .map_err(|e| ApiError::Storage(format!("Failed to open repository: {}", e)))?;
        Ok(Self { db })
    }

    /// Create a new document metadata.
    pub fn create_document(&self, title: String) -> Result<Document, ApiError> {
        let id = Uuid::new_v4();
        let now = chrono::Utc::now();

        let doc = Document {
            id,
            title,
            status: DocumentStatus::Indexing,
            section_count: 0,
            created_at: now,
            updated_at: now,
        };

        self.save_document(&doc)?;
        Ok(doc)
    }

    /// Save document metadata.
    pub fn save_document(&self, doc: &Document) -> Result<(), ApiError> {
        let key = doc.id.as_bytes().to_vec();
        let value = serde_json::to_vec(doc)
            .map_err(|e| ApiError::Storage(format!("Failed to serialize: {}", e)))?;

        self.db.insert(&key, value)
            .map_err(|e| ApiError::Storage(format!("Failed to save: {}", e)))?;

        Ok(())
    }

    /// Get document by ID.
    pub fn get_document(&self, id: Uuid) -> Result<Option<Document>, ApiError> {
        let key = id.as_bytes();
        match self.db.get(key) {
            Ok(Some(value)) => {
                let doc: Document = serde_json::from_slice(&value)
                    .map_err(|e| ApiError::Storage(format!("Failed to deserialize: {}", e)))?;
                Ok(Some(doc))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(ApiError::Storage(format!("Failed to get: {}", e))),
        }
    }

    /// List all documents.
    pub fn list_documents(&self) -> Result<Vec<Document>, ApiError> {
        let mut docs = Vec::new();
        for result in self.db.iter() {
            let (_, value) = result
                .map_err(|e| ApiError::Storage(format!("Failed to iterate: {}", e)))?;
            let doc: Document = serde_json::from_slice(&value)
                .map_err(|e| ApiError::Storage(format!("Failed to deserialize: {}", e)))?;
            docs.push(doc);
        }
        Ok(docs)
    }

    /// Delete document.
    pub fn delete_document(&self, id: Uuid) -> Result<(), ApiError> {
        let key = id.as_bytes();
        self.db.remove(key)
            .map_err(|e| ApiError::Storage(format!("Failed to delete: {}", e)))?;
        Ok(())
    }

    /// Update document status.
    pub fn update_status(&self, id: Uuid, status: DocumentStatus) -> Result<(), ApiError> {
        if let Some(mut doc) = self.get_document(id)? {
            doc.status = status;
            doc.updated_at = chrono::Utc::now();
            self.save_document(&doc)?;
        }
        Ok(())
    }
}
