// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Table of Contents (TOC) detection and extraction for PDF documents.
//!
//! This module provides functionality to detect, extract, and process
//! the table of contents from PDF documents, following the PageIndex approach.
//!
//! # Overview
//!
//! The TOC processing pipeline consists of three main modes:
//!
//! 1. **With Page Numbers**: TOC contains page numbers that can be matched to actual pages
//! 2. **Without Page Numbers**: TOC has structure but no page numbers; LLM finds sections
//! 3. **No TOC**: No table of contents found; LLM generates document structure
//!
//! # Usage
//!
//! ```no_run
//! use vectorless_core::toc::{TocProcessor, TocConfig};
//! use vectorless_core::pdf::Page;
//!
//! # async fn example<M: vectorless_llm::chat::ChatModel>(llm: M, pages: Vec<Page>) -> Result<(), Box<dyn std::error::Error>> {
//! let config = TocConfig::default();
//! let processor = TocProcessor::new(llm, config);
//!
//! let result = processor.detect_and_extract(&pages).await?;
//!
//! if result.detected {
//!     println!("Found TOC with {} entries", result.entries.len());
//! }
//! # Ok(())
//! # }
//! ```

use crate::pdf::Page;
use regex::Regex;
use serde::{Deserialize, Serialize};
use vectorless_llm::chat::{ChatModel, Message, Role, ChatOptions};

// ============================================================
// Data Structures
// ============================================================

/// A single entry in the table of contents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    /// Title of the section
    pub title: String,

    /// Hierarchical level (1 = top level, 2 = subsection, etc.)
    pub level: usize,

    /// Page number from TOC (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<usize>,

    /// Physical page index after matching (e.g., "<physical_index_5>")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical_index: Option<String>,

    /// Structure index (e.g., "1", "1.1", "1.2.3")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structure: Option<String>,
}

impl TocEntry {
    /// Create a new TOC entry.
    pub fn new(title: impl Into<String>, level: usize) -> Self {
        Self {
            title: title.into(),
            level,
            page: None,
            physical_index: None,
            structure: None,
        }
    }

    /// Create with page number.
    pub fn with_page(title: impl Into<String>, level: usize, page: usize) -> Self {
        let mut entry = Self::new(title, level);
        entry.page = Some(page);
        entry
    }
}

/// Result of TOC detection and extraction.
#[derive(Debug, Clone)]
pub struct TocResult {
    /// Whether TOC was detected
    pub detected: bool,

    /// TOC pages (0-based indices into the document)
    pub toc_pages: Vec<usize>,

    /// Whether TOC contains page numbers
    pub has_page_numbers: bool,

    /// Raw TOC content
    pub raw_content: Option<String>,

    /// Extracted TOC entries
    pub entries: Vec<TocEntry>,
}

impl TocResult {
    /// Create a "no TOC found" result.
    pub fn not_found() -> Self {
        Self {
            detected: false,
            toc_pages: Vec::new(),
            has_page_numbers: false,
            raw_content: None,
            entries: Vec::new(),
        }
    }
}

/// Configuration for TOC processing.
#[derive(Debug, Clone)]
pub struct TocConfig {
    /// Number of pages to check for TOC (default: 20)
    pub toc_check_pages: usize,

    /// Maximum retries for LLM completion (default: 5)
    pub max_retries: usize,

    /// Whether to use verbose logging
    pub verbose: bool,
}

impl Default for TocConfig {
    fn default() -> Self {
        Self {
            toc_check_pages: 20,
            max_retries: 5,
            verbose: false,
        }
    }
}

impl TocConfig {
    /// Create a new builder for TocConfig.
    pub fn builder() -> TocConfigBuilder {
        TocConfigBuilder::default()
    }
}

/// Builder for TocConfig.
#[derive(Debug, Clone, Default)]
pub struct TocConfigBuilder {
    config: TocConfig,
}

impl TocConfigBuilder {
    /// Set the number of pages to check for TOC.
    pub fn toc_check_pages(mut self, value: usize) -> Self {
        self.config.toc_check_pages = value;
        self
    }

    /// Set the maximum retries for LLM completion.
    pub fn max_retries(mut self, value: usize) -> Self {
        self.config.max_retries = value;
        self
    }

    /// Enable verbose logging.
    pub fn verbose(mut self, value: bool) -> Self {
        self.config.verbose = value;
        self
    }

    /// Build the config.
    pub fn build(self) -> TocConfig {
        self.config
    }
}

// ============================================================
// TocProcessor
// ============================================================

/// TOC processor that detects and extracts table of contents.
pub struct TocProcessor<M: ChatModel> {
    llm: M,
    config: TocConfig,
}

impl<M: ChatModel> TocProcessor<M> {
    /// Create a new TOC processor.
    pub fn new(llm: M, config: TocConfig) -> Self {
        Self { llm, config }
    }

    /// Create with default config.
    pub fn with_defaults(llm: M) -> Self {
        Self::new(llm, TocConfig::default())
    }

    /// Detect and extract TOC from document pages.
    ///
    /// This is the main entry point that:
    /// 1. Scans first N pages for TOC
    /// 2. Extracts TOC content
    /// 3. Detects if TOC has page numbers
    /// 4. Returns structured result
    pub async fn detect_and_extract(&self, pages: &[Page]) -> Result<TocResult, Error> {
        // Step 1: Find TOC pages
        let toc_pages = self.find_toc_pages(pages).await?;

        if toc_pages.is_empty() {
            if self.config.verbose {
                println!("No TOC found in document");
            }
            return Ok(TocResult::not_found());
        }

        // Step 2: Extract TOC content
        let raw_content = self.extract_toc_content(pages, &toc_pages)?;

        // Step 3: Detect if TOC has page numbers
        let has_page_numbers = self.detect_page_numbers(&raw_content).await?;

        // Step 4: Extract structured entries
        let entries = if has_page_numbers {
            self.extract_entries_with_pages(&raw_content, pages).await?
        } else {
            self.extract_entries_without_pages(&raw_content).await?
        };

        Ok(TocResult {
            detected: true,
            toc_pages,
            has_page_numbers,
            raw_content: Some(raw_content),
            entries,
        })
    }

    /// Find pages that contain the table of contents.
    ///
    /// Scans the first N pages and identifies consecutive pages that form the TOC.
    async fn find_toc_pages(&self, pages: &[Page]) -> Result<Vec<usize>, Error> {
        let mut toc_pages = Vec::new();
        let mut last_was_yes = false;

        let check_limit = self.config.toc_check_pages.min(pages.len());

        for i in 0..check_limit {
            // Only continue checking if we're still finding TOC or within limit
            if i >= self.config.toc_check_pages && !last_was_yes {
                break;
            }

            let detected = self.detect_toc_on_page(&pages[i].content).await?;

            if detected {
                if self.config.verbose {
                    println!("TOC detected on page {}", i + 1);
                }
                toc_pages.push(i);
                last_was_yes = true;
            } else if last_was_yes {
                // TOC has ended
                if self.config.verbose {
                    println!("TOC ended at page {}", i);
                }
                break;
            }
        }

        if toc_pages.is_empty() && self.config.verbose {
            println!("No TOC pages found");
        }

        Ok(toc_pages)
    }

    /// Detect if a single page contains table of contents.
    async fn detect_toc_on_page(&self, content: &str) -> Result<bool, Error> {
        let prompt = format!(
            "Your job is to detect if there is a table of contents in the given text.\n\
            Note: abstract, summary, notation list, figure list, table list, etc. are NOT table of contents.\n\n\
            Given text: {}\n\n\
            Return JSON format:\n\
            {{\n\
                \"thinking\": \"<why do you think there is a table of contents>\",\n\
                \"toc_detected\": \"<yes or no>\"\n\
            }}\n\
            Directly return the final JSON structure. Do not output anything else.",
            content
        );

        let response = self
            .llm
            .chat(
                &[Message {
                    role: Role::User,
                    content: prompt,
                }],
                &ChatOptions {
                    temperature: Some(0.0),
                    max_tokens: Some(100),
                },
            )
            .await
            .map_err(|e| Error::Llm(e.to_string()))?;

        let result: TocDetectionResponse = extract_json(&response.content)?;
        Ok(result.toc_detected.to_lowercase() == "yes")
    }

    /// Extract raw TOC content from the TOC pages.
    fn extract_toc_content(&self, pages: &[Page], toc_pages: &[usize]) -> Result<String, Error> {
        let mut content = String::new();

        for &page_idx in toc_pages {
            content.push_str(&pages[page_idx].content);
            content.push('\n');
        }

        // Transform dots to colons (common in PDFs: "Chapter 1.......5")
        let content = transform_dots_to_colon(&content);

        Ok(content)
    }

    /// Detect if TOC contains page numbers.
    async fn detect_page_numbers(&self, toc_content: &str) -> Result<bool, Error> {
        let prompt = format!(
            "You will be given a table of contents.\n\n\
            Your job is to detect if there are page numbers/indices given within the table of contents.\n\n\
            Given text: {}\n\n\
            Return JSON format:\n\
            {{\n\
                \"thinking\": \"<why do you think there are page numbers>\",\n\
                \"page_index_given_in_toc\": \"<yes or no>\"\n\
            }}\n\
            Directly return the final JSON structure. Do not output anything else.",
            toc_content
        );

        let response = self
            .llm
            .chat(
                &[Message {
                    role: Role::User,
                    content: prompt,
                }],
                &ChatOptions {
                    temperature: Some(0.0),
                    max_tokens: Some(100),
                },
            )
            .await
            .map_err(|e| Error::Llm(e.to_string()))?;

        let result: PageNumberDetectionResponse = extract_json(&response.content)?;
        Ok(result.page_index_given_in_toc.to_lowercase() == "yes")
    }

    /// Extract TOC entries when page numbers are present.
    async fn extract_entries_with_pages(
        &self,
        toc_content: &str,
        pages: &[Page],
    ) -> Result<Vec<TocEntry>, Error> {
        // First, transform TOC to structured JSON with page numbers
        let transformed = self.transform_toc_with_pages(toc_content).await?;

        // Then match page numbers to actual physical pages
        self.match_page_numbers(&transformed, pages).await
    }

    /// Transform raw TOC content to structured entries (with page numbers).
    async fn transform_toc_with_pages(&self, toc_content: &str) -> Result<Vec<TocEntry>, Error> {
        let prompt = format!(
            "You are given a table of contents. Your job is to transform it into a JSON format.\n\
            structure is the numeric system representing the hierarchy. For example:\n\
            - First section: \"1\"\n\
            - First subsection: \"1.1\"\n\
            - Second subsection: \"1.2\"\n\n\
            Return JSON format:\n\
            {{\n\
                \"table_of_contents\": [\n\
                    {{\n\
                        \"structure\": \"<x.x.x or null>\",\n\
                        \"title\": \"<section title>\",\n\
                        \"page\": \"<page number or null>\"\n\
                    }}\n\
                ]\n\
            }}\n\
            Transform the full table of contents. Directly return the JSON structure.\n\n\
            Given table of contents:\n{}",
            toc_content
        );

        let response = self
            .llm
            .chat(
                &[Message {
                    role: Role::User,
                    content: prompt,
                }],
                &ChatOptions {
                    temperature: Some(0.0),
                    max_tokens: Some(2000),
                },
            )
            .await
            .map_err(|e| Error::Llm(e.to_string()))?;

        let result: TocTransformationResponse = extract_json(&response.content)?;

        // Convert to TocEntry
        let mut entries = Vec::new();
        for item in result.table_of_contents {
            let level = calculate_level_from_structure(&item.structure);
            let page = item.page.and_then(|p| p.parse().ok());

            let mut entry = TocEntry::new(item.title, level);
            entry.page = page;
            entry.structure = Some(item.structure);
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Match TOC page numbers to actual physical pages.
    async fn match_page_numbers(
        &self,
        entries: &[TocEntry],
        pages: &[Page],
    ) -> Result<Vec<TocEntry>, Error> {
        // Find the first few entries with page numbers to determine offset
        let mut matched_pairs = Vec::new();

        for entry in entries.iter().take(5) {
            if let Some(toc_page) = entry.page {
                // Search for the title in pages around the expected location
                if let Some(physical_idx) = self.find_title_in_pages(&entry.title, pages, toc_page, toc_page + 5).await? {
                    matched_pairs.push((toc_page, physical_idx));
                }
            }
        }

        // Calculate page offset (e.g., TOC says page 5 but it's actually page 7)
        let offset = if !matched_pairs.is_empty() {
            // Most common offset
            let mut offsets = Vec::new();
            for (toc_page, physical_idx) in &matched_pairs {
                offsets.push(*physical_idx as i32 - *toc_page as i32);
            }
            // Find mode
            let mut counts = std::collections::HashMap::new();
            for offset in &offsets {
                *counts.entry(offset).or_insert(0) += 1;
            }
            *counts.into_iter().max_by_key(|(_, count)| *count).map(|(offset, _)| offset).unwrap_or(&0)
        } else {
            0
        };

        // Apply offset to all entries
        let mut result = Vec::new();
        for entry in entries {
            let mut entry = entry.clone();
            if let Some(toc_page) = entry.page {
                let physical_page = (toc_page as i32 + offset) as usize;
                if physical_page > 0 && physical_page <= pages.len() {
                    entry.physical_index = Some(format!("<physical_index_{}>", physical_page));
                }
            }
            result.push(entry);
        }

        Ok(result)
    }

    /// Find a title within a range of pages.
    async fn find_title_in_pages(
        &self,
        title: &str,
        pages: &[Page],
        start: usize,
        end: usize,
    ) -> Result<Option<usize>, Error> {
        let end = end.min(pages.len());
        let start = start.saturating_sub(1); // Convert to 0-based, allow some leeway

        for i in start..end {
            if self.title_appears_on_page(title, &pages[i].content).await? {
                return Ok(Some(i + 1)); // Return 1-based page number
            }
        }

        Ok(None)
    }

    /// Check if a title appears on a specific page.
    async fn title_appears_on_page(&self, title: &str, content: &str) -> Result<bool, Error> {
        // Simple string matching with normalization
        let normalized_title = normalize_title(title);
        let normalized_content = normalize_title(content);

        Ok(normalized_content.contains(&normalized_title))
    }

    /// Extract TOC entries when no page numbers are present.
    async fn extract_entries_without_pages(
        &self,
        toc_content: &str,
    ) -> Result<Vec<TocEntry>, Error> {
        let prompt = format!(
            "You are given a table of contents. Extract all sections with their hierarchy levels.\n\n\
            Given text: {}\n\n\
            Return JSON format:\n\
            [\n\
                {{\n\
                    \"structure\": \"<x.x.x>\",\n\
                    \"title\": \"<section title>\",\n\
                    \"level\": <1-6>\n\
                }}\n\
            ]\n\
            Directly return the JSON structure.",
            toc_content
        );

        let response = self
            .llm
            .chat(
                &[Message {
                    role: Role::User,
                    content: prompt,
                }],
                &ChatOptions {
                    temperature: Some(0.0),
                    max_tokens: Some(2000),
                },
            )
            .await
            .map_err(|e| Error::Llm(e.to_string()))?;

        let entries: Vec<TocEntryExtraction> = extract_json(&response.content)?;

        Ok(entries
            .into_iter()
            .map(|e| TocEntry {
                title: e.title,
                level: e.level,
                page: None,
                physical_index: None,
                structure: Some(e.structure),
            })
            .collect())
    }
}

// ============================================================
// Helper Functions
// ============================================================

/// Transform dots to colons (common in PDF TOCs).
fn transform_dots_to_colon(text: &str) -> String {
    // "Chapter 1.......5" -> "Chapter 1: 5"
    let re = Regex::new(r"\.{5,}").unwrap();
    let text = re.replace_all(text, ": ");

    // Handle "Chapter 1 . . . . . 5" (dots with spaces)
    let re = Regex::new(r"(?:\. ){5,}\.?").unwrap();
    let text = re.replace_all(&text, ": ");

    text.to_string()
}

/// Normalize title for matching (remove extra spaces, lowercase).
fn normalize_title(title: &str) -> String {
    title.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Calculate hierarchy level from structure string.
fn calculate_level_from_structure(structure: &str) -> usize {
    structure.matches('.').count() + 1
}

/// Extract JSON from LLM response (handles markdown code blocks).
fn extract_json<T: for<'de> Deserialize<'de>>(text: &str) -> Result<T, Error> {
    let text = text.trim();

    // Check if entire text is valid JSON
    if let Ok(value) = serde_json::from_str::<T>(text) {
        return Ok(value);
    }

    // Try to find JSON object boundaries
    let start = text.find('{').ok_or_else(|| Error::InvalidJson("No JSON object found".into()))?;
    let end = text.rfind('}').ok_or_else(|| Error::InvalidJson("No JSON object found".into()))?;

    if start >= end {
        return Err(Error::InvalidJson("Invalid JSON object bounds".into()));
    }

    let json_str = &text[start..=end];

    // Clean up common issues
    let json_str = json_str.replace("None", "null");
    let json_str = json_str.replace('\n', " ").replace('\r', " ");

    serde_json::from_str::<T>(&json_str).map_err(|e| Error::InvalidJson(format!("JSON parse error: {}", e)))
}

// ============================================================
// Response Types for LLM
// ============================================================

#[derive(Debug, Deserialize)]
struct TocDetectionResponse {
    #[serde(rename = "toc_detected")]
    toc_detected: String,
}

#[derive(Debug, Deserialize)]
struct PageNumberDetectionResponse {
    #[serde(rename = "page_index_given_in_toc")]
    page_index_given_in_toc: String,
}

#[derive(Debug, Deserialize)]
struct TocTransformationResponse {
    #[serde(rename = "table_of_contents")]
    table_of_contents: Vec<TocTransformationItem>,
}

#[derive(Debug, Deserialize)]
struct TocTransformationItem {
    structure: String,
    title: String,
    #[serde(default)]
    page: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TocEntryExtraction {
    structure: String,
    title: String,
    level: usize,
}

// ============================================================
// Error Types
// ============================================================

/// TOC processing error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Invalid JSON response: {0}")]
    InvalidJson(String),

    #[error("No TOC found in document")]
    NoTocFound,

    #[error("TOC extraction failed: {0}")]
    ExtractionFailed(String),

    #[error("Page matching failed: {0}")]
    PageMatchingFailed(String),
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_toc_entry_creation() {
        let entry = TocEntry::new("Chapter 1", 1);
        assert_eq!(entry.title, "Chapter 1");
        assert_eq!(entry.level, 1);
        assert!(entry.page.is_none());
    }

    #[test]
    fn test_toc_entry_with_page() {
        let entry = TocEntry::with_page("Chapter 1", 1, 5);
        assert_eq!(entry.page, Some(5));
    }

    #[test]
    fn test_transform_dots_to_colon() {
        let input = "Chapter 1.......5";
        let output = transform_dots_to_colon(input);
        assert!(output.contains("Chapter 1:"));
    }

    #[test]
    fn test_transform_spaced_dots() {
        let input = "Chapter 1. . . . . . 5";
        let output = transform_dots_to_colon(input);
        assert!(output.contains("Chapter 1:"));
    }

    #[test]
    fn test_normalize_title() {
        let title = "  Hello   World  ";
        let normalized = normalize_title(title);
        assert_eq!(normalized, "hello world");
    }

    #[test]
    fn test_calculate_level_from_structure() {
        assert_eq!(calculate_level_from_structure("1"), 1);
        assert_eq!(calculate_level_from_structure("1.1"), 2);
        assert_eq!(calculate_level_from_structure("1.2.3"), 3);
    }

    #[test]
    fn test_toc_result_not_found() {
        let result = TocResult::not_found();
        assert!(!result.detected);
        assert!(result.entries.is_empty());
    }

    #[test]
    fn test_toc_config_default() {
        let config = TocConfig::default();
        assert_eq!(config.toc_check_pages, 20);
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_toc_config_builder() {
        let config = TocConfig::builder()
            .toc_check_pages(10)
            .max_retries(3)
            .verbose(true)
            .build();

        assert_eq!(config.toc_check_pages, 10);
        assert_eq!(config.max_retries, 3);
        assert!(config.verbose);
    }

    #[test]
    fn test_extract_json_simple() {
        let json = r#"{"toc_detected": "yes"}"#;
        let result: TocDetectionResponse = extract_json(json).unwrap();
        assert_eq!(result.toc_detected, "yes");
    }

    #[test]
    fn test_extract_json_with_markdown() {
        let json = r#"```json
        {"toc_detected": "yes"}
        ```"#;
        let result: TocDetectionResponse = extract_json(json).unwrap();
        assert_eq!(result.toc_detected, "yes");
    }

    #[test]
    fn test_extract_json_with_extra_text() {
        let json = r#"Some text before {"toc_detected": "yes"} some text after"#;
        let result: TocDetectionResponse = extract_json(json).unwrap();
        assert_eq!(result.toc_detected, "yes");
    }
}
