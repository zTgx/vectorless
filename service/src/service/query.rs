// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Query service for retrieving relevant content.

use crate::dto::{ApiError, Source, DocumentStatus};
use crate::repository::{IndexRepository, MetadataRepository};
use vectorless_core::load;
use vectorless_llm::chat::ChatModel;

// Import retrieve function with explicit path to avoid naming conflicts
use vectorless_core::retriever::retrieve_simple;

/// Query service for RAG.
pub struct QueryService<M> {
    llm: M,
    metadata_repository: MetadataRepository,
    index_repository: IndexRepository,
}

impl<M: ChatModel> QueryService<M> {
    /// Create a new query service.
    pub fn new(llm: M, metadata_repository: MetadataRepository, index_repository: IndexRepository) -> Self {
        Self {
            llm,
            metadata_repository,
            index_repository,
        }
    }

    /// Query the RAG system.
    pub async fn query(&self, query: &str, max_results: Option<usize>) -> Result<QueryResult, ApiError> {
        let max_results = max_results.unwrap_or(3);

        // Get all ready documents
        let docs = self.metadata_repository.list_documents()?
            .into_iter()
            .filter(|d| d.status == DocumentStatus::Ready)
            .collect::<Vec<_>>();

        if docs.is_empty() {
            return Ok(QueryResult {
                answer: "No documents available for querying.".to_string(),
                sources: vec![],
            });
        }

        // Limit results
        let docs_to_search = docs.iter().take(max_results);

        // Collect relevant content from each document
        let mut all_contexts = Vec::new();
        let mut sources = Vec::new();

        for doc in docs_to_search {
            match self.query_document(query, doc).await {
                Ok(Some((content, sections))) => {
                    all_contexts.push(content);
                    sources.extend(sections);
                }
                Ok(None) => {}
                Err(e) => {
                    tracing::warn!("Failed to query document {}: {}", doc.id, e);
                }
            }
        }

        // Build answer from contexts
        let answer = if all_contexts.is_empty() {
            "No relevant information found.".to_string()
        } else {
            let combined_context = all_contexts.join("\n\n");
            self.generate_answer(query, &combined_context).await?
        };

        Ok(QueryResult { answer, sources })
    }

    /// Query a single document.
    async fn query_document(
        &self,
        query: &str,
        doc: &crate::dto::Document,
    ) -> Result<Option<(String, Vec<Source>)>, ApiError> {
        // Load the index
        let index_path = self.index_repository.get_index_path(doc.id);
        let root = load(&index_path)
            .map_err(|e| ApiError::Storage(format!("Failed to load index: {}", e)))?;

        // Retrieve relevant content
        let content = retrieve_simple(&self.llm, query, &root)
            .await
            .map_err(|e| ApiError::Query(format!("Retrieval failed: {}", e)))?;

        if content.is_empty() {
            return Ok(None);
        }

        // Create source reference
        let source = Source {
            document_id: doc.id.to_string(),
            section: doc.title.clone(),
            content: content.clone(),
        };

        Ok(Some((content, vec![source])))
    }

    /// Generate final answer from retrieved context.
    async fn generate_answer(&self, query: &str, context: &str) -> Result<String, ApiError> {
        use vectorless_llm::chat::{Message, Role, ChatOptions};

        let messages = vec![
            Message {
                role: Role::System,
                content: "You are a helpful assistant that answers questions based on the provided context. Be concise and accurate.".to_string(),
            },
            Message {
                role: Role::User,
                content: format!("Context:\n{}\n\nQuestion: {}\n\nAnswer:", context, query),
            },
        ];

        let response = self.llm.chat(&messages, &ChatOptions::default())
            .await
            .map_err(|e| ApiError::Query(format!("LLM call failed: {}", e)))?;

        Ok(response.content)
    }
}

/// Query result.
pub struct QueryResult {
    pub answer: String,
    pub sources: Vec<Source>,
}
