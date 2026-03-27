// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Document ingestion pipeline.

use crate::models::{ApiError, DocumentStatus};
use crate::store::{IndexStore, MetadataStore};
use uuid::Uuid;
use vectorless_core::{parse::parse_document_with_config, index::build_summaries_with_config, storage::save, IndexerConfig};
use vectorless_llm::chat::ChatModel;

/// Document ingestion pipeline.
pub struct IngestPipeline<M> {
    llm: M,
    config: IndexerConfig,
    metadata_store: MetadataStore,
    index_store: IndexStore,
}

impl<M: ChatModel> IngestPipeline<M> {
    /// Create a new ingestion pipeline.
    pub fn new(llm: M, config: IndexerConfig, metadata_store: MetadataStore, index_store: IndexStore) -> Self {
        Self {
            llm,
            config,
            metadata_store,
            index_store,
        }
    }

    /// Ingest a document.
    pub async fn ingest(
        &self,
        document_id: Uuid,
        _title: String,
        content: &str,
    ) -> Result<(), ApiError> {
        // Update status to indexing
        self.metadata_store.update_status(document_id, DocumentStatus::Indexing)?;

        // Parse the document
        let root = parse_document_with_config(&self.llm, content, &self.config)
            .await
            .map_err(|e| ApiError::Parsing(format!("Failed to parse document: {}", e)))?;

        // Build summaries
        build_summaries_with_config(&self.llm, &root, &self.config)
            .await
            .map_err(|e| ApiError::Indexing(format!("Failed to build summaries: {}", e)))?;

        // Save the index
        let index_path = self.index_store.get_index_path(document_id);
        save(&root, &index_path)
            .map_err(|e| ApiError::Storage(format!("Failed to save index: {}", e)))?;

        // Update metadata with section count
        let section_count = self.count_sections(&root);
        if let Some(mut doc) = self.metadata_store.get_document(document_id)? {
            doc.section_count = section_count;
            doc.status = DocumentStatus::Ready;
            self.metadata_store.save_document(&doc)?;
        }

        Ok(())
    }

    /// Count the total number of sections in the document tree.
    fn count_sections(&self, root: &vectorless_core::node::PageNodeRef) -> usize {
        let node = root.borrow();
        let mut count = 1;
        for child in &node.children {
            count += self.count_sections(child);
        }
        count
    }
}
