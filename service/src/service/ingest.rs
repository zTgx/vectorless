// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Document ingestion service.

use crate::dto::{ApiError, DocumentStatus};
use crate::repository::{IndexRepository, MetadataRepository};
use uuid::Uuid;
use vectorless_core::{parse_document_with_config, build_summaries_with_config, save, IndexerConfig};
use vectorless_llm::chat::ChatModel;

/// Document ingestion service.
pub struct IngestService<M> {
    llm: M,
    config: IndexerConfig,
    metadata_repository: MetadataRepository,
    index_repository: IndexRepository,
}

impl<M: ChatModel> IngestService<M> {
    /// Create a new ingestion service.
    pub fn new(llm: M, config: IndexerConfig, metadata_repository: MetadataRepository, index_repository: IndexRepository) -> Self {
        Self {
            llm,
            config,
            metadata_repository,
            index_repository,
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
        self.metadata_repository.update_status(document_id, DocumentStatus::Indexing)?;

        // Parse the document
        let root = parse_document_with_config(&self.llm, content, &self.config)
            .await
            .map_err(|e| ApiError::Parsing(format!("Failed to parse document: {}", e)))?;

        // Build summaries
        build_summaries_with_config(&self.llm, &root, &self.config)
            .await
            .map_err(|e| ApiError::Indexing(format!("Failed to build summaries: {}", e)))?;

        // Save the index
        let index_path = self.index_repository.get_index_path(document_id);
        save(&root, &index_path)
            .map_err(|e| ApiError::Storage(format!("Failed to save index: {}", e)))?;

        // Update metadata with section count
        let section_count = self.count_sections(&root);
        if let Some(mut doc) = self.metadata_repository.get_document(document_id)? {
            doc.section_count = section_count;
            doc.status = DocumentStatus::Ready;
            self.metadata_repository.save_document(&doc)?;
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
