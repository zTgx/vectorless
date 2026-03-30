// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Document-level API for querying document metadata and content.
//!
//! This module provides functions similar to PageIndex's retrieve.py:
//! - `get_document()` - Get document metadata
//! - `get_document_structure()` - Get document tree structure (without text fields)
//! - `get_page_content()` - Get content for specific page ranges
//!
//! # Example
//!
//! ```no_run
//! use std::collections::HashMap;
//! use vectorless_core::document::{Document, DocumentType, get_document, parse_page_range};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut documents = HashMap::new();
//!
//! // Query document metadata
//! let meta = get_document(&documents, "doc-id");
//! println!("{}", meta);
//!
//! // Parse page ranges
//! let pages = parse_page_range("5-7,3,8,12")?;
//! assert_eq!(pages, vec![3, 5, 6, 7, 8, 12]);
//! # Ok(())
//! # }
//! ```

use crate::node::PageNodeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================
// Core Structures
// ============================================================

/// Document type (PDF or Markdown).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocumentType {
    /// PDF document type.
    #[serde(rename = "pdf")]
    Pdf,
    /// Markdown document type.
    #[serde(rename = "markdown")]
    Markdown,
}

/// Document metadata and content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique document identifier.
    pub id: String,

    /// Document type.
    #[serde(rename = "type")]
    pub doc_type: DocumentType,

    /// Document name/title.
    pub doc_name: String,

    /// Document description (LLM-generated).
    pub doc_description: String,

    /// Path to the original file.
    pub file_path: PathBuf,

    /// Total page count (for PDF documents).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_count: Option<usize>,

    /// Total line count (for Markdown documents).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_count: Option<usize>,

    /// Document tree structure (lazy-loaded, may be None).
    /// Note: This field is skipped during serialization/deserialization
    /// because PageNodeRef cannot be directly serialized.
    #[serde(skip)]
    pub root: Option<PageNodeRef>,

    /// Cached page content for PDF documents (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<Vec<CachedPage>>,
}

/// Cached page content for PDF documents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedPage {
    /// Page number (1-based).
    pub page: usize,

    /// Page text content.
    pub content: String,
}

/// Lightweight document summary (metadata only).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSummary {
    /// Document ID.
    pub id: String,

    /// Document type.
    #[serde(rename = "type")]
    pub doc_type: DocumentType,

    /// Document name.
    pub doc_name: String,

    /// Document description.
    pub doc_description: String,

    /// Page count (for PDF).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_count: Option<usize>,

    /// Line count (for Markdown).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_count: Option<usize>,

    /// File path.
    pub file_path: PathBuf,
}

/// Structure node DTO (without text fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureNodeDto {
    /// Node title.
    pub title: String,

    /// Node ID (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,

    /// Summary (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,

    /// Depth in tree.
    pub depth: usize,

    /// Start page/line number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<usize>,

    /// End page/line number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_index: Option<usize>,

    /// Child nodes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub nodes: Vec<StructureNodeDto>,
}

/// Document metadata response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Document ID.
    pub doc_id: String,

    /// Document name.
    pub doc_name: String,

    /// Document description.
    pub doc_description: String,

    /// Document type.
    #[serde(rename = "type")]
    pub doc_type: String,

    /// Processing status.
    pub status: String,

    /// Page count (for PDF).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_count: Option<usize>,

    /// Line count (for Markdown).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_count: Option<usize>,
}

/// Error response wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error message.
    pub error: String,
}

// ============================================================
// Page Range Parsing
// ============================================================

/// Parse a page range string like "5-7,3,8,12" into sorted unique page numbers.
///
/// # Supported Formats
///
/// - Single pages: `"5"`
/// - Ranges: `"5-7"` (inclusive)
/// - Mixed: `"5-7,3,8,12"`
///
/// # Examples
///
/// ```
/// use vectorless_core::document::parse_page_range;
///
/// let pages = parse_page_range("5-7,3,8,12").unwrap();
/// assert_eq!(pages, vec![3, 5, 6, 7, 8, 12]);
///
/// let single = parse_page_range("5").unwrap();
/// assert_eq!(single, vec![5]);
/// ```
///
/// # Errors
///
/// Returns an error if the format is invalid (e.g., non-numeric values,
/// invalid ranges like "7-5" where start > end).
pub fn parse_page_range(spec: &str) -> Result<Vec<usize>, Error> {
    let mut result = Vec::new();

    for part in spec.split(',') {
        let part = part.trim();

        if part.is_empty() {
            continue;
        }

        if let Some(idx) = part.find('-') {
            // Range: "5-7"
            let start: usize = part[..idx]
                .trim()
                .parse()
                .map_err(|_| Error::InvalidPageRange(format!("Invalid start number: {}", part)))?;

            let end: usize = part[idx + 1..]
                .trim()
                .parse()
                .map_err(|_| Error::InvalidPageRange(format!("Invalid end number: {}", part)))?;

            if start > end {
                return Err(Error::InvalidPageRange(format!(
                    "Invalid range '{}': start ({}) must be <= end ({})",
                    part, start, end
                )));
            }

            result.extend(start..=end);
        } else {
            // Single number
            let num: usize = part
                .parse()
                .map_err(|_| Error::InvalidPageRange(format!("Invalid page number: {}", part)))?;
            result.push(num);
        }
    }

    // Sort and deduplicate
    result.sort();
    result.dedup();

    Ok(result)
}

// ============================================================
// Document API Functions
// ============================================================

/// Get document metadata as a JSON string.
///
/// Returns a JSON object with:
/// - `doc_id`: Document ID
/// - `doc_name`: Document name
/// - `doc_description`: Document description
/// - `type`: Document type ("pdf" or "markdown")
/// - `status`: Processing status (always "completed")
/// - `page_count`: Page count (for PDF documents)
/// - `line_count`: Line count (for Markdown documents)
///
/// If the document is not found, returns a JSON object with an `error` field.
///
/// # Examples
///
/// ```no_run
/// use std::collections::HashMap;
/// use vectorless_core::document::get_document;
///
/// let documents = HashMap::new();
/// let meta = get_document(&documents, "non-existent");
/// assert!(meta.contains("\"error\":"));
/// ```
pub fn get_document(documents: &HashMap<String, Document>, doc_id: &str) -> String {
    match documents.get(doc_id) {
        Some(doc) => {
            let doc_type_str = match doc.doc_type {
                DocumentType::Pdf => "pdf",
                DocumentType::Markdown => "markdown",
            };

            let metadata = DocumentMetadata {
                doc_id: doc_id.to_string(),
                doc_name: doc.doc_name.clone(),
                doc_description: doc.doc_description.clone(),
                doc_type: doc_type_str.to_string(),
                status: "completed".to_string(),
                page_count: doc.page_count,
                line_count: doc.line_count,
            };

            serde_json::to_string(&metadata).unwrap_or_else(|_| {
                serde_json::json!({"error": "Failed to serialize metadata"}).to_string()
            })
        }
        None => {
            let error = ErrorResponse {
                error: format!("Document {} not found", doc_id),
            };
            serde_json::to_string(&error).unwrap()
        }
    }
}

/// Get document structure without text fields (saves tokens).
///
/// Returns a JSON representation of the document tree with all `text`/`content`
/// fields removed. This is useful for agents that need to understand the
/// document structure without loading all the content.
///
/// If the document is not found or has no structure, returns a JSON object
/// with an `error` field.
///
/// # Examples
///
/// ```no_run
/// use std::collections::HashMap;
/// use vectorless_core::document::get_document_structure;
///
/// let documents = HashMap::new();
/// let structure = get_document_structure(&documents, "doc-id");
/// // Returns JSON tree without text fields
/// ```
pub fn get_document_structure(documents: &HashMap<String, Document>, doc_id: &str) -> String {
    match documents.get(doc_id) {
        Some(doc) => {
            if let Some(ref root) = doc.root {
                let dto = to_structure_dto(root);
                serde_json::to_string(&dto).unwrap_or_else(|_| {
                    serde_json::json!({"error": "Failed to serialize structure"}).to_string()
                })
            } else {
                let error = ErrorResponse {
                    error: format!("Document {} has no structure loaded", doc_id),
                };
                serde_json::to_string(&error).unwrap()
            }
        }
        None => {
            let error = ErrorResponse {
                error: format!("Document {} not found", doc_id),
            };
            serde_json::to_string(&error).unwrap()
        }
    }
}

/// Get page content for specific pages.
///
/// # Arguments
///
/// * `documents` - HashMap of documents
/// * `doc_id` - Document ID to query
/// * `pages` - Page range specification (e.g., "5-7", "3,8", "12")
///
/// # Page Format
///
/// - For **PDF documents**: Pages are physical page numbers (1-indexed)
/// - For **Markdown documents**: Pages are line numbers corresponding to node headers
///
/// # Returns
///
/// A JSON array of `{"page": number, "content": string}` objects.
///
/// If the document is not found or pages format is invalid, returns a JSON
/// object with an `error` field.
///
/// # Examples
///
/// ```no_run
/// use std::collections::HashMap;
/// use vectorless_core::document::get_page_content;
///
/// let documents = HashMap::new();
/// let content = get_page_content(&documents, "doc-id", "5-7,3,8,12");
/// // Returns JSON: [{"page": 3, "content": "..."}, ...]
/// ```
pub fn get_page_content(
    documents: &HashMap<String, Document>,
    doc_id: &str,
    pages: &str,
) -> String {
    // Get document
    let doc = match documents.get(doc_id) {
        Some(d) => d,
        None => {
            return serde_json::to_string(&ErrorResponse {
                error: format!("Document {} not found", doc_id),
            })
            .unwrap();
        }
    };

    // Parse page range
    let page_nums = match parse_page_range(pages) {
        Ok(nums) => nums,
        Err(e) => {
            return serde_json::to_string(&ErrorResponse {
                error: format!("Invalid pages format: {}. Use \"5-7\", \"3,8\", or \"12\". Error: {}", pages, e),
            })
            .unwrap();
        }
    };

    // Get content based on document type
    let content = match doc.doc_type {
        DocumentType::Pdf => get_pdf_page_content(doc, &page_nums),
        DocumentType::Markdown => get_md_page_content(doc, &page_nums),
    };

    serde_json::to_string(&content).unwrap_or_else(|_| {
        serde_json::json!({"error": "Failed to serialize page content"}).to_string()
    })
}

// ============================================================
// Helper Functions
// ============================================================

/// Extract page content from a PDF document.
///
/// For PDFs with cached pages, returns content from the cache.
/// Otherwise, returns an empty result (PDF re-parsing not yet implemented).
fn get_pdf_page_content(doc: &Document, page_nums: &[usize]) -> Vec<CachedPage> {
    let mut result = Vec::new();

    if let Some(ref cached_pages) = doc.pages {
        let page_map: std::collections::HashMap<usize, &CachedPage> = cached_pages
            .iter()
            .map(|p| (p.page, p))
            .collect();

        for &page in page_nums {
            if let Some(&cached) = page_map.get(&page) {
                result.push(CachedPage {
                    page,
                    content: cached.content.clone(),
                });
            }
        }
    }

    result
}

/// Extract page content from a Markdown document (by line numbers).
///
/// For Markdown, "pages" are line numbers. This function finds nodes
/// whose line_num falls within the requested range.
fn get_md_page_content(doc: &Document, line_nums: &[usize]) -> Vec<CachedPage> {
    let mut result = Vec::new();

    if let Some(ref root) = doc.root {
        let min_line = *line_nums.first().unwrap_or(&1);
        let max_line = *line_nums.last().unwrap_or(&min_line);
        let mut seen = std::collections::HashSet::new();

        collect_md_content_by_line(root, min_line, max_line, &mut result, &mut seen);
    }

    result.sort_by_key(|c| c.page);
    result
}

/// Recursively collect Markdown content by line number.
fn collect_md_content_by_line(
    node: &PageNodeRef,
    min_line: usize,
    max_line: usize,
    out: &mut Vec<CachedPage>,
    seen: &mut std::collections::HashSet<usize>,
) {
    let borrowed = node.borrow();

    // Check if this node has a line number in range
    // For Markdown, we use start_page as line_num
    if let Some(line_num) = borrowed.start_page {
        if line_num >= min_line && line_num <= max_line && !seen.contains(&line_num) {
            seen.insert(line_num);
            out.push(CachedPage {
                page: line_num,
                content: borrowed.content.clone(),
            });
        }
    }

    // Recurse into children
    for child in &borrowed.children {
        collect_md_content_by_line(child, min_line, max_line, out, seen);
    }
}

/// Convert a PageNode to a StructureNodeDto (without text fields).
pub fn to_structure_dto(node: &PageNodeRef) -> StructureNodeDto {
    let borrowed = node.borrow();

    StructureNodeDto {
        title: borrowed.title.clone(),
        node_id: borrowed.node_id.clone(),
        summary: if borrowed.summary.is_empty() {
            None
        } else {
            Some(borrowed.summary.clone())
        },
        depth: borrowed.depth,
        start_index: borrowed.start_page,
        end_index: borrowed.end_page,
        nodes: borrowed
            .children
            .iter()
            .map(|c| to_structure_dto(c))
            .collect(),
    }
}

// ============================================================
// Error Types
// ============================================================

/// Document API error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Document not found.
    #[error("Document not found: {0}")]
    DocumentNotFound(String),

    /// Invalid page range format.
    #[error("Invalid page range: {0}")]
    InvalidPageRange(String),

    /// Page content not available for this document type.
    #[error("Page content not available for document type")]
    ContentNotAvailable,

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::PageNode;
    use std::path::PathBuf;

    #[test]
    fn test_parse_page_range_single() {
        let pages = parse_page_range("5").unwrap();
        assert_eq!(pages, vec![5]);
    }

    #[test]
    fn test_parse_page_range_range() {
        let pages = parse_page_range("5-7").unwrap();
        assert_eq!(pages, vec![5, 6, 7]);
    }

    #[test]
    fn test_parse_page_range_mixed() {
        let pages = parse_page_range("5-7,3,8,12").unwrap();
        assert_eq!(pages, vec![3, 5, 6, 7, 8, 12]);
    }

    #[test]
    fn test_parse_page_range_duplicates() {
        let pages = parse_page_range("5-7,5,6").unwrap();
        assert_eq!(pages, vec![5, 6, 7]); // Duplicates removed
    }

    #[test]
    fn test_parse_page_range_invalid_start() {
        let result = parse_page_range("abc-7");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_page_range_invalid_end() {
        let result = parse_page_range("5-abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_page_range_inverted_range() {
        let result = parse_page_range("7-5");
        assert!(result.is_err());
    }

    #[test]
    fn test_document_type_serialization() {
        let pdf = DocumentType::Pdf;
        let json = serde_json::to_string(&pdf).unwrap();
        assert_eq!(json, "\"pdf\"");

        let md = DocumentType::Markdown;
        let json = serde_json::to_string(&md).unwrap();
        assert_eq!(json, "\"markdown\"");
    }

    #[test]
    fn test_get_document_not_found() {
        let documents = HashMap::new();
        let result = get_document(&documents, "non-existent");
        assert!(result.contains("\"error\":"));
        assert!(result.contains("not found"));
    }

    #[test]
    fn test_get_document_success() {
        let mut documents = HashMap::new();

        let doc = Document {
            id: "test-doc-1".to_string(),
            doc_type: DocumentType::Pdf,
            doc_name: "Test Document".to_string(),
            doc_description: "A test PDF".to_string(),
            file_path: PathBuf::from("/test/doc.pdf"),
            page_count: Some(10),
            line_count: None,
            root: None,
            pages: None,
        };

        documents.insert(doc.id.clone(), doc);

        let result = get_document(&documents, "test-doc-1");
        assert!(result.contains("\"doc_id\":\"test-doc-1\""));
        assert!(result.contains("\"doc_name\":\"Test Document\""));
        assert!(result.contains("\"type\":\"pdf\""));
        assert!(result.contains("\"page_count\":10"));
    }

    #[test]
    fn test_get_document_structure_not_found() {
        let documents = HashMap::new();
        let result = get_document_structure(&documents, "non-existent");
        assert!(result.contains("\"error\":"));
    }

    #[test]
    fn test_get_document_structure_no_root() {
        let mut documents = HashMap::new();

        let doc = Document {
            id: "test-doc-2".to_string(),
            doc_type: DocumentType::Pdf,
            doc_name: "Test Doc 2".to_string(),
            doc_description: "Test".to_string(),
            file_path: PathBuf::from("/test/doc2.pdf"),
            page_count: None,
            line_count: None,
            root: None,
            pages: None,
        };

        documents.insert(doc.id.clone(), doc);

        let result = get_document_structure(&documents, "test-doc-2");
        assert!(result.contains("\"error\":"));
        assert!(result.contains("no structure loaded"));
    }

    #[test]
    fn test_get_document_structure_with_root() {
        let mut documents = HashMap::new();

        let root = PageNode::new("Root", "Root content");
        root.borrow_mut().node_id = Some("0001".to_string());
        root.borrow_mut().summary = "Root summary".to_string();
        root.borrow_mut().depth = 0;
        root.borrow_mut().start_page = Some(1);
        root.borrow_mut().end_page = Some(10);

        let doc = Document {
            id: "test-doc-3".to_string(),
            doc_type: DocumentType::Pdf,
            doc_name: "Test Doc 3".to_string(),
            doc_description: "Test".to_string(),
            file_path: PathBuf::from("/test/doc3.pdf"),
            page_count: Some(10),
            line_count: None,
            root: Some(root.clone()),
            pages: None,
        };

        documents.insert(doc.id.clone(), doc);

        let result = get_document_structure(&documents, "test-doc-3");

        // Should contain structure but not text content
        assert!(result.contains("\"title\":\"Root\""));
        assert!(result.contains("\"summary\":\"Root summary\""));
        assert!(result.contains("\"depth\":0"));
        assert!(result.contains("\"start_index\":1"));
        assert!(result.contains("\"end_index\":10"));

        // Should NOT contain the actual text content
        assert!(!result.contains("Root content"));
    }

    #[test]
    fn test_get_page_content_not_found() {
        let documents = HashMap::new();
        let result = get_page_content(&documents, "non-existent", "1-3");
        assert!(result.contains("\"error\":"));
    }

    #[test]
    fn test_get_page_content_invalid_range() {
        let mut documents = HashMap::new();

        let doc = Document {
            id: "test-doc-4".to_string(),
            doc_type: DocumentType::Pdf,
            doc_name: "Test Doc 4".to_string(),
            doc_description: "Test".to_string(),
            file_path: PathBuf::from("/test/doc4.pdf"),
            page_count: None,
            line_count: None,
            root: None,
            pages: None,
        };

        documents.insert(doc.id.clone(), doc);

        let result = get_page_content(&documents, "test-doc-4", "invalid");
        assert!(result.contains("\"error\":"));
        assert!(result.contains("Invalid pages format"));
    }

    #[test]
    fn test_get_page_content_pdf_cached() {
        let mut documents = HashMap::new();

        let pages = vec![
            CachedPage {
                page: 1,
                content: "Page 1 content".to_string(),
            },
            CachedPage {
                page: 2,
                content: "Page 2 content".to_string(),
            },
            CachedPage {
                page: 3,
                content: "Page 3 content".to_string(),
            },
        ];

        let doc = Document {
            id: "test-doc-5".to_string(),
            doc_type: DocumentType::Pdf,
            doc_name: "Test Doc 5".to_string(),
            doc_description: "Test".to_string(),
            file_path: PathBuf::from("/test/doc5.pdf"),
            page_count: Some(3),
            line_count: None,
            root: None,
            pages: Some(pages),
        };

        documents.insert(doc.id.clone(), doc);

        let result = get_page_content(&documents, "test-doc-5", "1-2");
        assert!(result.contains("\"page\":1"));
        assert!(result.contains("\"page\":2"));
        assert!(result.contains("Page 1 content"));
        assert!(result.contains("Page 2 content"));
        assert!(!result.contains("Page 3 content"));
    }

    #[test]
    fn test_to_structure_dto_no_text() {
        let root = PageNode::new("Root", "Secret content");
        root.borrow_mut().summary = "Public summary".to_string();
        root.borrow_mut().depth = 0;

        let dto = to_structure_dto(&root);

        assert_eq!(dto.title, "Root");
        assert_eq!(dto.summary, Some("Public summary".to_string()));
        assert_eq!(dto.depth, 0);
        // The dto should not have a content/text field
    }

    #[test]
    fn test_to_structure_dto_with_children() {
        let root = PageNode::new("Root", "Root content");
        root.borrow_mut().depth = 0;

        let child1 = PageNode::new("Chapter 1", "Chapter 1 content");
        child1.borrow_mut().depth = 1;
        child1.borrow_mut().node_id = Some("0001".to_string());

        let child2 = PageNode::new("Chapter 2", "Chapter 2 content");
        child2.borrow_mut().depth = 1;
        child2.borrow_mut().node_id = Some("0002".to_string());

        root.borrow_mut().children.push(child1);
        root.borrow_mut().children.push(child2);

        let dto = to_structure_dto(&root);

        assert_eq!(dto.title, "Root");
        assert_eq!(dto.nodes.len(), 2);
        assert_eq!(dto.nodes[0].title, "Chapter 1");
        assert_eq!(dto.nodes[1].title, "Chapter 2");
        assert_eq!(dto.nodes[0].node_id, Some("0001".to_string()));
        assert_eq!(dto.nodes[1].node_id, Some("0002".to_string()));
    }
}
