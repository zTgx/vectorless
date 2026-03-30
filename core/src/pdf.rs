// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! PDF document parsing for vectorless indexing.
//!
//! This module provides functionality to parse PDF documents and extract
//! text content page by page, following the approach used in PageIndex.
//!
//! # Example
//!
//! ```no_run
//! use vectorless_core::pdf::{PdfExtractor, PdfParser};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let extractor = PdfExtractor::default();
//! let doc = extractor.parse("document.pdf")?;
//! for page in &doc.pages {
//!     println!("Page {}: {} tokens", page.index, page.tokens);
//! }
//! # Ok(())
//! # }
//! ```

use std::path::Path;

// ============================================================
// Data Structures
// ============================================================

/// A single page from a PDF document.
///
/// Represents one page with its extracted text content and metadata.
#[derive(Debug, Clone)]
pub struct Page {
    /// Page number (1-based, matching PDF convention)
    pub index: usize,

    /// Extracted text content from this page
    pub content: String,

    /// Estimated token count for the content
    pub tokens: usize,
}

impl Page {
    /// Create a new page with given content.
    pub fn new(index: usize, content: String) -> Self {
        let tokens = estimate_tokens(&content);
        Self { index, content, tokens }
    }

    /// Get the page content with boundary markers.
    ///
    /// Returns the content wrapped in `<physical_index_X>...</physical_index_X>` tags,
    /// following PageIndex convention for marking page boundaries.
    pub fn with_boundaries(&self) -> String {
        format!(
            "<physical_index_{}>\n{}\n<physical_index_{}>\n",
            self.index, self.content, self.index
        )
    }
}

/// Result of PDF parsing.
///
/// Contains all pages from the document with metadata.
#[derive(Debug, Clone)]
pub struct PdfDocument {
    /// All pages in the document
    pub pages: Vec<Page>,

    /// Total number of pages
    pub page_count: usize,

    /// Total token count across all pages
    pub total_tokens: usize,
}

impl PdfDocument {
    /// Create a new PDF document from pages.
    fn new(pages: Vec<Page>) -> Self {
        let page_count = pages.len();
        let total_tokens = pages.iter().map(|p| p.tokens).sum();
        Self {
            pages,
            page_count,
            total_tokens,
        }
    }

    /// Get the full document text with page boundary markers.
    ///
    /// Concatenates all pages with `<physical_index_X>` markers between them.
    pub fn to_marked_text(&self) -> String {
        self.pages
            .iter()
            .map(|p| p.with_boundaries())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get text for a specific page range.
    ///
    /// # Arguments
    ///
    /// * `start` - Start page (1-based, inclusive)
    /// * `end` - End page (1-based, inclusive)
    ///
    /// # Example
    ///
    /// Get pages 5 through 10 from a document:
    ///
    /// ```no_run
    /// # use vectorless_core::pdf::{PdfExtractor, PdfParser};
    /// # let extractor = PdfExtractor::new();
    /// # let doc = extractor.parse("document.pdf").unwrap();
    /// if let Some(text) = doc.get_page_range(5, 10) {
    ///     println!("Pages 5-10: {} characters", text.len());
    /// }
    /// ```
    pub fn get_page_range(&self, start: usize, end: usize) -> Option<String> {
        if start < 1 || end > self.page_count || start > end {
            return None;
        }

        self.pages[start - 1..end]
            .iter()
            .map(|p| p.content.as_str())
            .collect::<Vec<_>>()
            .join("\n\n")
            .into()
    }

    /// Get text for a specific page range with boundary markers.
    pub fn get_page_range_with_boundaries(&self, start: usize, end: usize) -> Option<String> {
        if start < 1 || end > self.page_count || start > end {
            return None;
        }

        self.pages[start - 1..end]
            .iter()
            .map(|p| p.with_boundaries())
            .collect::<Vec<_>>()
            .join("\n")
            .into()
    }

    /// Get a single page's content.
    pub fn get_page(&self, index: usize) -> Option<&Page> {
        if index < 1 || index > self.page_count {
            return None;
        }
        self.pages.get(index - 1)
    }
}

/// Token counting strategy.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TokenStrategy {
    /// Simple estimation: 1 token ≈ 4 characters (default)
    #[default]
    Simple,

    /// Word-based estimation: 1 token ≈ 0.75 words
    WordBased,

    /// Use exact byte count divided by 4
    ByteBased,
}

impl TokenStrategy {
    /// Estimate token count for the given text.
    pub fn count(self, text: &str) -> usize {
        if text.is_empty() {
            return 0;
        }

        match self {
            TokenStrategy::Simple => (text.len() / 4).max(1),
            TokenStrategy::WordBased => {
                let word_count = text.split_whitespace().count();
                (word_count * 3 / 4).max(1)
            }
            TokenStrategy::ByteBased => (text.len() / 4).max(1),
        }
    }
}

// ============================================================
// PdfParser Trait
// ============================================================

/// Trait for PDF parsing implementations.
///
/// This trait allows for different PDF parsing backends.
pub trait PdfParser {
    /// Parse a PDF file and extract all pages with text content.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the PDF file
    ///
    /// # Returns
    ///
    /// A `PdfDocument` containing all pages and metadata.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file cannot be read
    /// - The PDF format is invalid
    /// - Text extraction fails
    fn parse(&self, path: impl AsRef<Path>) -> Result<PdfDocument, Error>;

    /// Parse a PDF file with custom token counting strategy.
    fn parse_with_strategy(
        &self,
        path: impl AsRef<Path>,
        strategy: TokenStrategy,
    ) -> Result<PdfDocument, Error>;
}

// ============================================================
// PdfExtractor Implementation
// ============================================================

/// Default PDF extractor using pdf-extract library.
///
/// This is the standard implementation that works for most PDFs.
/// For more complex PDFs, consider implementing `PdfParser` with
/// a different backend (e.g., lopdf, poppler).
#[derive(Debug, Clone, Default)]
pub struct PdfExtractor {
    /// Token counting strategy to use.
    pub token_strategy: TokenStrategy,

    /// Whether to preserve layout (experimental).
    pub preserve_layout: bool,
}

impl PdfExtractor {
    /// Create a new PDF extractor with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new PDF extractor with custom token strategy.
    pub fn with_strategy(strategy: TokenStrategy) -> Self {
        Self {
            token_strategy: strategy,
            preserve_layout: false,
        }
    }

    /// Extract text from PDF document.
    ///
    /// Internal method that uses pdf-extract library.
    fn extract_from_path(&self, path: &Path) -> Result<Vec<String>, Error> {
        let path_str = path.to_str().ok_or_else(|| Error::InvalidPath)?;
        let data = std::fs::read(path).map_err(|e| Error::IoError(e.to_string()))?;

        self.extract_from_data(&data)
    }

    /// Extract text from PDF data in memory.
    fn extract_from_data(&self, data: &[u8]) -> Result<Vec<String>, Error> {
        use pdf_extract::extract_text_from_mem;

        let text = extract_text_from_mem(data)
            .map_err(|e| Error::ExtractionFailed(e.to_string()))?;

        // Split by form feed characters (page separators)
        // pdf-extract uses \f to separate pages
        let pages: Vec<String> = text
            .split('\x0c')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        if pages.is_empty() {
            return Err(Error::NoContent);
        }

        Ok(pages)
    }
}

impl PdfParser for PdfExtractor {
    fn parse(&self, path: impl AsRef<Path>) -> Result<PdfDocument, Error> {
        self.parse_with_strategy(path.as_ref(), self.token_strategy)
    }

    fn parse_with_strategy(
        &self,
        path: impl AsRef<Path>,
        strategy: TokenStrategy,
    ) -> Result<PdfDocument, Error> {
        let path_ref = path.as_ref();

        // Validate file exists
        if !path_ref.exists() {
            return Err(Error::FileNotFound(path_ref.display().to_string()));
        }

        // Extract text pages
        let text_pages = self.extract_from_path(path_ref)?;

        // Convert to Page structs
        let pages: Vec<Page> = text_pages
            .into_iter()
            .enumerate()
            .map(|(idx, content)| {
                let index = idx + 1; // 1-based
                let tokens = strategy.count(&content);
                Page {
                    index,
                    content,
                    tokens,
                }
            })
            .collect();

        Ok(PdfDocument::new(pages))
    }
}

// ============================================================
// Helper Functions
// ============================================================

/// Estimate token count for text using default strategy.
///
/// This is a simple approximation: 1 token ≈ 4 characters.
/// For more accurate counting, consider integrating tiktoken-rs.
pub fn estimate_tokens(text: &str) -> usize {
    TokenStrategy::Simple.count(text)
}

/// Mark page boundaries in text using PageIndex convention.
///
/// Wraps each page's content in `<physical_index_X>` tags.
/// This is useful when you need to process the full document
/// while preserving page boundary information.
///
/// # Example
///
/// ```
/// use vectorless_core::pdf::mark_page_boundaries;
///
/// let pages = vec![
///     ("First page content".to_string()),
///     ("Second page content".to_string()),
/// ];
/// let marked = mark_page_boundaries(&pages);
/// // marked contains:
/// // "<physical_index_1>\nFirst page content\n<physical_index_1>\n
/// //  <physical_index_2>\nSecond page content\n<physical_index_2>\n"
/// ```
pub fn mark_page_boundaries(pages: &[String]) -> String {
    pages
        .iter()
        .enumerate()
        .map(|(idx, content)| {
            let page_num = idx + 1;
            format!(
                "<physical_index_{}>\n{}\n<physical_index_{}>\n",
                page_num, content, page_num
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parse page specification string into a list of page numbers.
///
/// Supports formats like:
/// - "5" -> [5]
/// - "5-7" -> [5, 6, 7]
/// - "3,8" -> [3, 8]
/// - "3,5-7,10" -> [3, 5, 6, 7, 10]
///
/// # Example
///
/// ```
/// use vectorless_core::pdf::parse_page_spec;
///
/// let pages = parse_page_spec("3,5-7,10").unwrap();
/// assert_eq!(pages, vec![3, 5, 6, 7, 10]);
/// ```
pub fn parse_page_spec(spec: &str) -> Result<Vec<usize>, Error> {
    let mut result = Vec::new();

    for part in spec.split(',') {
        let part = part.trim();
        if let Some(range_idx) = part.find('-') {
            // Range: "5-7"
            let start: usize = part[..range_idx]
                .trim()
                .parse()
                .map_err(|_| Error::InvalidPageSpec(format!("Invalid range start: {}", part)))?;
            let end: usize = part[range_idx + 1..]
                .trim()
                .parse()
                .map_err(|_| Error::InvalidPageSpec(format!("Invalid range end: {}", part)))?;

            if start > end {
                return Err(Error::InvalidPageSpec(format!(
                    "Range start > end: {} > {}",
                    start, end
                )));
            }

            result.extend(start..=end);
        } else {
            // Single page
            let page: usize = part
                .parse()
                .map_err(|_| Error::InvalidPageSpec(format!("Invalid page number: {}", part)))?;
            result.push(page);
        }
    }

    // Remove duplicates and sort
    result.sort();
    result.dedup();

    Ok(result)
}

// ============================================================
// Error Types
// ============================================================

/// PDF parsing error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// File not found at the specified path.
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// Invalid file path.
    #[error("Invalid path")]
    InvalidPath,

    /// I/O error during file reading.
    #[error("I/O error: {0}")]
    IoError(String),

    /// PDF text extraction failed.
    #[error("Text extraction failed: {0}")]
    ExtractionFailed(String),

    /// No content could be extracted from the PDF.
    #[error("No content found in PDF")]
    NoContent,

    /// Invalid page specification string.
    #[error("Invalid page specification: {0}")]
    InvalidPageSpec(String),
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_page_creation() {
        let page = Page::new(1, "Test content".to_string());
        assert_eq!(page.index, 1);
        assert_eq!(page.content, "Test content");
        assert!(page.tokens > 0);
    }

    #[test]
    fn test_page_with_boundaries() {
        let page = Page::new(1, "Test content".to_string());
        let marked = page.with_boundaries();
        // PageIndex uses same format for both start and end tags
        assert!(marked.contains("<physical_index_1>"));
        assert!(marked.contains("Test content"));
        // Should have two occurrences of the marker (start and end)
        assert_eq!(marked.matches("<physical_index_1>").count(), 2);
    }

    #[test]
    fn test_estimate_tokens() {
        let text = "This is a test string with some words.";
        let tokens = estimate_tokens(text);
        assert!(tokens > 0);
        assert!(tokens < text.len()); // Should be less than character count
    }

    #[test]
    fn test_token_strategy_simple() {
        let text = "Hello world";
        let count = TokenStrategy::Simple.count(text);
        assert_eq!(count, 2); // 11 chars / 4 = 2
    }

    #[test]
    fn test_token_strategy_word_based() {
        let text = "Hello world test";
        let count = TokenStrategy::WordBased.count(text);
        assert_eq!(count, 2); // 3 words * 3/4 = 2
    }

    #[test]
    fn test_mark_page_boundaries() {
        let pages = vec![
            "First page".to_string(),
            "Second page".to_string(),
        ];
        let marked = mark_page_boundaries(&pages);
        assert!(marked.contains("<physical_index_1>"));
        assert!(marked.contains("<physical_index_2>"));
        assert!(marked.contains("First page"));
        assert!(marked.contains("Second page"));
    }

    #[test]
    fn test_parse_page_spec_single() {
        let pages = parse_page_spec("5").unwrap();
        assert_eq!(pages, vec![5]);
    }

    #[test]
    fn test_parse_page_spec_range() {
        let pages = parse_page_spec("5-7").unwrap();
        assert_eq!(pages, vec![5, 6, 7]);
    }

    #[test]
    fn test_parse_page_spec_mixed() {
        let pages = parse_page_spec("3,5-7,10").unwrap();
        assert_eq!(pages, vec![3, 5, 6, 7, 10]);
    }

    #[test]
    fn test_parse_page_spec_invalid_range() {
        let result = parse_page_spec("7-5");
        assert!(result.is_err());
    }

    #[test]
    fn test_pdf_document_get_page() {
        let pages = vec![
            Page::new(1, "First".to_string()),
            Page::new(2, "Second".to_string()),
            Page::new(3, "Third".to_string()),
        ];
        let doc = PdfDocument::new(pages);

        assert_eq!(doc.page_count, 3);
        assert_eq!(doc.get_page(1).unwrap().content, "First");
        assert_eq!(doc.get_page(2).unwrap().content, "Second");
        assert!(doc.get_page(0).is_none());
        assert!(doc.get_page(4).is_none());
    }

    #[test]
    fn test_pdf_document_get_page_range() {
        let pages = vec![
            Page::new(1, "First".to_string()),
            Page::new(2, "Second".to_string()),
            Page::new(3, "Third".to_string()),
        ];
        let doc = PdfDocument::new(pages);

        let range = doc.get_page_range(1, 2).unwrap();
        assert!(range.contains("First"));
        assert!(range.contains("Second"));
        assert!(!range.contains("Third"));

        // Invalid range
        assert!(doc.get_page_range(0, 2).is_none());
        assert!(doc.get_page_range(2, 1).is_none());
        assert!(doc.get_page_range(1, 10).is_none());
    }

    #[test]
    fn test_pdf_document_to_marked_text() {
        let pages = vec![
            Page::new(1, "First".to_string()),
            Page::new(2, "Second".to_string()),
        ];
        let doc = PdfDocument::new(pages);
        let marked = doc.to_marked_text();

        assert!(marked.contains("<physical_index_1>"));
        assert!(marked.contains("<physical_index_2>"));
        assert!(marked.contains("First"));
        assert!(marked.contains("Second"));
    }

    #[test]
    fn test_pdf_extractor_default() {
        let extractor = PdfExtractor::new();
        assert_eq!(extractor.token_strategy, TokenStrategy::Simple);
        assert!(!extractor.preserve_layout);
    }

    #[test]
    fn test_pdf_extractor_with_strategy() {
        let extractor = PdfExtractor::with_strategy(TokenStrategy::WordBased);
        assert_eq!(extractor.token_strategy, TokenStrategy::WordBased);
    }
}
