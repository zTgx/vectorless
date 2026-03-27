// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Configuration for the indexing pipeline.

/// Configuration for document indexing and retrieval.
///
/// All fields have sensible defaults. Create with `IndexerConfig::default()`
/// and customize as needed.
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    // === Parsing ===
    /// Word count threshold for splitting sections into subsections.
    /// Sections longer than this will be recursively segmented.
    /// Default: 300
    pub subsection_threshold: usize,

    /// Maximum tokens to send in a single segmentation request.
    /// Helps avoid mid-thought splits by giving the LLM more context.
    /// Default: 3000
    pub max_segment_tokens: usize,

    // === Indexing ===
    /// Model name to use for summary generation.
    /// Default: "gpt-4-mini"
    pub summary_model: String,

    /// Maximum tokens for each summary.
    /// Default: 150
    pub max_summary_tokens: u32,

    // === Retrieval ===
    /// Maximum content tokens allowed in a leaf node.
    /// If exceeded, the section should be split further.
    /// Default: 1500
    pub max_content_tokens: u32,
}

impl Default for IndexerConfig {
    fn default() -> Self {
        Self {
            subsection_threshold: 300,
            max_segment_tokens: 3000,
            summary_model: "gpt-4-mini".to_string(),
            max_summary_tokens: 150,
            max_content_tokens: 1500,
        }
    }
}

impl IndexerConfig {
    /// Create a builder for constructing config with custom values.
    pub fn builder() -> IndexerConfigBuilder {
        IndexerConfigBuilder::default()
    }
}

/// Builder for `IndexerConfig`.
#[derive(Debug, Clone, Default)]
pub struct IndexerConfigBuilder {
    config: IndexerConfig,
}

impl IndexerConfigBuilder {
    /// Set the subsection threshold.
    pub fn subsection_threshold(mut self, value: usize) -> Self {
        self.config.subsection_threshold = value;
        self
    }

    /// Set the max segment tokens.
    pub fn max_segment_tokens(mut self, value: usize) -> Self {
        self.config.max_segment_tokens = value;
        self
    }

    /// Set the summary model name.
    pub fn summary_model(mut self, value: impl Into<String>) -> Self {
        self.config.summary_model = value.into();
        self
    }

    /// Set the max summary tokens.
    pub fn max_summary_tokens(mut self, value: u32) -> Self {
        self.config.max_summary_tokens = value;
        self
    }

    /// Set the max content tokens.
    pub fn max_content_tokens(mut self, value: u32) -> Self {
        self.config.max_content_tokens = value;
        self
    }

    /// Build the config.
    pub fn build(self) -> IndexerConfig {
        self.config
    }
}
