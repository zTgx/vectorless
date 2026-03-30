// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Enhanced retrieval with multiple modes and context collection.
//!
//! This module provides advanced retrieval capabilities that go beyond
//! simple tree navigation, including multi-path traversal, page range
//! retrieval, and path context collection.
//!
//! # Retrieval Modes
//!
//! - **TreeNavigation**: Navigate down the tree to find the most relevant leaf (original mode)
//! - **PageRange**: Retrieve all content within a specific page range
//! - **MultiPath**: Collect content from multiple relevant branches
//! - **PathContext**: Collect context along the retrieval path for better understanding
//!
//! # Example
//!
//! ```no_run
//! use vectorless_core::retriever::{retrieve_with_mode, RetrieveMode};
//! use vectorless_llm::chat::ChatModel;
//!
//! # async fn example<M: ChatModel>(llm: M, root: &PageNodeRef, query: &str) -> Result<(), Box<dyn std::error::Error>> {
//! let result = retrieve_with_mode(&llm, query, root, RetrieveMode::MultiPath(3)).await?;
//! println!("Found {} relevant sections", result.sections.len());
//! # Ok(())
//! # }
//! ```

use crate::node::{PageNodeRef, PageNodeRefExt};
use vectorless_llm::chat::{ChatModel, Message, Role, ChatOptions};
use std::collections::HashSet;

// ============================================================
// Retrieval Modes
// ============================================================

/// Different retrieval strategies for navigating the document tree.
#[derive(Debug, Clone, Copy)]
pub enum RetrieveMode {
    /// Navigate down the tree to find the most relevant leaf node.
    /// Returns only the content of that leaf.
    TreeNavigation,

    /// Retrieve all content within a specific page range.
    /// Collects all nodes whose page ranges intersect with [start, end].
    PageRange { start: usize, end: usize },

    /// Collect content from multiple relevant branches.
    /// Explores top-K branches and collects their content.
    MultiPath { top_k: usize },

    /// Collect context along the retrieval path.
    /// Returns not just the target content, but also summaries
    /// of parent nodes to provide better context.
    PathContext,

    /// Comprehensive retrieval combining multiple strategies.
    /// Collects content from target, its path context, and related branches.
    Comprehensive { top_k: usize },
}

/// Result of a retrieval operation.
#[derive(Debug, Clone)]
pub struct RetrieveResult {
    /// The main answer/response content.
    pub answer: String,

    /// Individual sections that were retrieved.
    pub sections: Vec<RetrievedSection>,

    /// The path taken through the document tree.
    pub path: Vec<PathStep>,

    /// Metadata about the retrieval operation.
    pub metadata: RetrieveMetadata,
}

/// A single retrieved section from the document.
#[derive(Debug, Clone)]
pub struct RetrievedSection {
    /// Title of the section.
    pub title: String,

    /// Content of the section.
    pub content: String,

    /// Summary of the section.
    pub summary: String,

    /// Page range (if applicable).
    pub page_range: Option<(usize, usize)>,

    /// Node ID (if available).
    pub node_id: Option<String>,

    /// Depth in the tree.
    pub depth: usize,

    /// Relevance score (if available).
    pub relevance_score: Option<f32>,
}

/// A step in the retrieval path through the document.
#[derive(Debug, Clone)]
pub struct PathStep {
    /// Title of the section at this level.
    pub title: String,

    /// Summary of the section.
    pub summary: String,

    /// Depth in the tree.
    pub depth: usize,

    /// Node ID (if available).
    pub node_id: Option<String>,
}

/// Metadata about the retrieval operation.
#[derive(Debug, Clone)]
pub struct RetrieveMetadata {
    /// Total number of sections retrieved.
    pub section_count: usize,

    /// Total token count of retrieved content.
    pub total_tokens: usize,

    /// Pages covered by the retrieval.
    pub pages_covered: Vec<usize>,

    /// Retrieval mode used.
    pub mode: RetrieveMode,

    /// Whether retrieval was successful.
    pub success: bool,
}

impl Default for RetrieveMetadata {
    fn default() -> Self {
        Self {
            section_count: 0,
            total_tokens: 0,
            pages_covered: Vec::new(),
            mode: RetrieveMode::TreeNavigation,
            success: true,
        }
    }
}

// ============================================================
// Enhanced Retrieval Functions
// ============================================================

/// Retrieve content using a specific retrieval mode.
///
/// This is the main entry point for enhanced retrieval, supporting
/// multiple strategies beyond simple tree navigation.
pub async fn retrieve_with_mode<M>(
    llm: &M,
    query: &str,
    root: &PageNodeRef,
    mode: RetrieveMode,
) -> Result<RetrieveResult, Error>
where
    M: ChatModel,
{
    match mode {
        RetrieveMode::TreeNavigation => {
            retrieve_tree_navigation(llm, query, root).await
        }
        RetrieveMode::PageRange { start, end } => {
            retrieve_page_range(root, start, end).await
        }
        RetrieveMode::MultiPath { top_k } => {
            retrieve_multi_path(llm, query, root, top_k).await
        }
        RetrieveMode::PathContext => {
            retrieve_with_path_context(llm, query, root).await
        }
        RetrieveMode::Comprehensive { top_k } => {
            retrieve_comprehensive(llm, query, root, top_k).await
        }
    }
}

/// Tree navigation mode - navigate to the most relevant leaf.
async fn retrieve_tree_navigation<M>(
    llm: &M,
    query: &str,
    root: &PageNodeRef,
) -> Result<RetrieveResult, Error>
where
    M: ChatModel,
{
    let mut node = PageNodeRef::clone(root);
    let mut path = Vec::new();
    let mut total_tokens = 0;

    loop {
        let borrowed = node.borrow();

        // Add to path
        path.push(PathStep {
            title: borrowed.title.clone(),
            summary: borrowed.summary.clone(),
            depth: borrowed.depth,
            node_id: borrowed.node_id.clone(),
        });

        total_tokens += estimate_tokens(&borrowed.content);

        let is_leaf = borrowed.children.is_empty();

        if is_leaf {
            // Return the leaf content
            let content = borrowed.content.clone();
            drop(borrowed);

            return Ok(RetrieveResult {
                answer: content.clone(),
                sections: vec![RetrievedSection {
                    title: node.borrow().title.clone(),
                    content,
                    summary: node.borrow().summary.clone(),
                    page_range: node.page_range(),
                    node_id: node.borrow().node_id.clone(),
                    depth: node.borrow().depth,
                    relevance_score: Some(1.0),
                }],
                path,
                metadata: RetrieveMetadata {
                    section_count: 1,
                    total_tokens,
                    pages_covered: extract_pages_from_section(&node),
                    mode: RetrieveMode::TreeNavigation,
                    success: true,
                },
            });
        }

        // Pick the best child
        drop(borrowed);
        node = pick_child(llm, query, &node).await?;
    }
}

/// Page range mode - retrieve all content within a page range.
async fn retrieve_page_range(
    root: &PageNodeRef,
    start: usize,
    end: usize,
) -> Result<RetrieveResult, Error> {
    let mut sections = Vec::new();
    let mut total_tokens = 0;
    let mut pages_covered = HashSet::new();

    // Collect all nodes that intersect with the page range
    let nodes = collect_nodes_in_page_range_recursive(root, start, end);

    for node in nodes {
        let borrowed = node.borrow();

        let page_range = node.page_range();
        if let Some((node_start, node_end)) = page_range {
            // Add pages to coverage
            for page in node_start..=node_end {
                if page >= start && page <= end {
                    pages_covered.insert(page);
                }
            }
        }

        total_tokens += estimate_tokens(&borrowed.content);

        sections.push(RetrievedSection {
            title: borrowed.title.clone(),
            content: borrowed.content.clone(),
            summary: borrowed.summary.clone(),
            page_range,
            node_id: borrowed.node_id.clone(),
            depth: borrowed.depth,
            relevance_score: Some(1.0),
        });
    }

    // Combine all content
    let combined_content: Vec<String> = sections
        .iter()
        .map(|s| format!("## {}\n\n{}", s.title, s.content))
        .collect();

    let answer = combined_content.join("\n\n---\n\n");

    let section_count = sections.len();
    let has_content = !sections.is_empty();

    Ok(RetrieveResult {
        answer,
        sections,
        path: vec![],
        metadata: RetrieveMetadata {
            section_count,
            total_tokens,
            pages_covered: pages_covered.into_iter().collect(),
            mode: RetrieveMode::PageRange { start, end },
            success: has_content,
        },
    })
}

/// Multi-path mode - collect content from multiple relevant branches.
async fn retrieve_multi_path<M>(
    llm: &M,
    query: &str,
    root: &PageNodeRef,
    top_k: usize,
) -> Result<RetrieveResult, Error>
where
    M: ChatModel,
{
    let children: Vec<PageNodeRef> = {
        let borrowed = root.borrow();
        borrowed.children.iter().map(|c| PageNodeRef::clone(c)).collect()
    };

    if children.is_empty() {
        return Err(Error::NoChildren);
    }

    // Score each child based on its summary
    let mut scored_children: Vec<(PageNodeRef, f32)> = Vec::new();
    for child in &children {
        let child_borrowed = child.borrow();
        let score = score_relevance(llm, query, &child_borrowed.summary).await?;
        scored_children.push((child.clone(), score));
    }

    // Sort by score (descending)
    scored_children.sort_by(|a, b| {
        let (_, score_a) = a;
        let (_, score_b) = b;
        score_b.partial_cmp(score_a).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Take top K
    let top_children: Vec<_> = scored_children
        .into_iter()
        .take(top_k)
        .collect();

    // Collect content from top K branches
    let mut sections = Vec::new();
    let mut total_tokens = 0;
    let mut pages_covered = HashSet::new();

    for (child, score) in top_children {
        let borrowed = child.borrow();

        let page_range = child.page_range();
        if let Some((start, end)) = page_range {
            for page in start..=end {
                pages_covered.insert(page);
            }
        }

        total_tokens += estimate_tokens(&borrowed.content);

        sections.push(RetrievedSection {
            title: borrowed.title.clone(),
            content: borrowed.content.clone(),
            summary: borrowed.summary.clone(),
            page_range,
            node_id: borrowed.node_id.clone(),
            depth: borrowed.depth,
            relevance_score: Some(score),
        });
    }

    // Combine content
    let answer: Vec<String> = sections
        .iter()
        .map(|s| format!("## {} (relevance: {:.2})\n\n{}", s.title, s.relevance_score.unwrap_or(0.0), s.content))
        .collect();

    let answer = answer.join("\n\n---\n\n");

    let section_count = sections.len();
    let has_content = !sections.is_empty();

    Ok(RetrieveResult {
        answer,
        sections,
        path: vec![],
        metadata: RetrieveMetadata {
            section_count,
            total_tokens,
            pages_covered: pages_covered.into_iter().collect(),
            mode: RetrieveMode::MultiPath { top_k },
            success: has_content,
        },
    })
}

/// Path context mode - retrieve with context along the path.
async fn retrieve_with_path_context<M>(
    llm: &M,
    query: &str,
    root: &PageNodeRef,
) -> Result<RetrieveResult, Error>
where
    M: ChatModel,
{
    // First do tree navigation to find the target
    let target_result = retrieve_tree_navigation(llm, query, root).await?;

    // Build enhanced path context
    let path_context = build_enhanced_path_context(root, &target_result.path);

    // Combine target content with path context
    let mut sections = target_result.sections;

    // Add path context sections
    for step in &path_context {
        sections.push(RetrievedSection {
            title: step.title.clone(),
            content: String::new(), // Path context has no content
            summary: step.summary.clone(),
            page_range: None,
            node_id: step.node_id.clone(),
            depth: step.depth,
            relevance_score: Some(0.5), // Lower relevance for context
        });
    }

    // Build answer with context
    let context_parts: Vec<String> = path_context
        .iter()
        .map(|step| format!("{}: {}", step.title, step.summary))
        .collect();

    let context = context_parts.join(" → ");

    let answer = format!(
        "Document Path: {}\n\n---\n\n{}",
        context,
        target_result.answer
    );

    let section_count = sections.len();

    Ok(RetrieveResult {
        answer,
        sections,
        path: target_result.path,
        metadata: RetrieveMetadata {
            section_count,
            total_tokens: target_result.metadata.total_tokens,
            pages_covered: target_result.metadata.pages_covered,
            mode: RetrieveMode::PathContext,
            success: true,
        },
    })
}

/// Comprehensive mode - combine multiple strategies.
async fn retrieve_comprehensive<M>(
    llm: &M,
    query: &str,
    root: &PageNodeRef,
    top_k: usize,
) -> Result<RetrieveResult, Error>
where
    M: ChatModel,
{
    // Get path context result
    let path_result = retrieve_with_path_context(llm, query, root).await?;

    // Get multi-path result for additional context
    let multi_result = retrieve_multi_path(llm, query, root, top_k).await?;

    // Combine results
    let mut sections = path_result.sections;

    // Add multi-path sections that aren't already included
    let existing_ids: HashSet<_> = sections
        .iter()
        .filter_map(|s| s.node_id.as_ref())
        .cloned()
        .collect();

    for section in multi_result.sections {
        if let Some(ref id) = section.node_id {
            if !existing_ids.contains(id) {
                sections.push(section);
            }
        }
    }

    let section_count = sections.len();

    // Build comprehensive answer
    let answer = format!(
        "Primary Result:\n{}\n\n---\n\nAdditional Relevant Sections:\n{}",
        path_result.answer,
        multi_result.answer
    );

    Ok(RetrieveResult {
        answer,
        sections,
        path: path_result.path,
        metadata: RetrieveMetadata {
            section_count,
            total_tokens: path_result.metadata.total_tokens + multi_result.metadata.total_tokens,
            pages_covered: {
                let mut set = HashSet::new();
                set.extend(path_result.metadata.pages_covered);
                set.extend(multi_result.metadata.pages_covered);
                set.into_iter().collect()
            },
            mode: RetrieveMode::Comprehensive { top_k },
            success: true,
        },
    })
}

// ============================================================
// Helper Functions
// ============================================================

/// Pick the most relevant child node using LLM.
async fn pick_child<M>(llm: &M, query: &str, node: &PageNodeRef) -> Result<PageNodeRef, Error>
where
    M: ChatModel,
{
    let (title, children): (String, Vec<PageNodeRef>) = {
        let borrowed = node.borrow();
        let title = borrowed.title.clone();
        let children = borrowed.children.iter().map(|c| PageNodeRef::clone(c)).collect();
        (title, children)
    };

    if children.is_empty() {
        return Err(Error::NoChildren);
    }

    // Build options list with summaries
    let options: Vec<String> = {
        let mut opts = Vec::new();
        for (i, child) in children.iter().enumerate() {
            let borrowed = child.borrow();
            opts.push(format!("{}. [{}]: {}", i + 1, borrowed.title, borrowed.summary));
        }
        opts
    };

    let options_text = options.join("\n");

    let prompt = format!(
        r#"You are navigating a document tree to find the answer to a question.

Current section: "{}"
Question: {}

Children of this section:
{}

Which child section most likely contains the answer? Reply with only the number."#,
        title, query, options_text
    );

    let response = llm
        .chat(
            &[Message {
                role: Role::User,
                content: prompt,
            }],
            &ChatOptions {
                temperature: Some(0.0),
                max_tokens: Some(5),
            },
        )
        .await
        .map_err(|e| Error::Llm(e.to_string()))?;

    // Parse the LLM response to get the index
    let content = response.content.trim();
    let index = content
        .parse::<usize>()
        .map_err(|_| Error::InvalidResponse(format!("Not a number: {}", content)))?
        .checked_sub(1)
        .ok_or_else(|| Error::InvalidResponse("Index must be >= 1".into()))?;

    children
        .get(index)
        .cloned()
        .ok_or_else(|| Error::InvalidResponse(format!("Index {} out of bounds", index + 1)))
}

/// Score the relevance of a summary to a query.
async fn score_relevance<M>(llm: &M, query: &str, summary: &str) -> Result<f32, Error>
where
    M: ChatModel,
{
    let prompt = format!(
        "On a scale of 0.0 to 1.0, how relevant is the following summary to the question?\n\n\
        Question: {}\n\
        Summary: {}\n\
        Reply with only a number between 0.0 and 1.0.",
        query, summary
    );

    let response = llm
        .chat(
            &[Message {
                role: Role::User,
                content: prompt,
            }],
            &ChatOptions {
                temperature: Some(0.0),
                max_tokens: Some(10),
            },
        )
        .await
        .map_err(|e| Error::Llm(e.to_string()))?;

    let content = response.content.trim();
    let score = content
        .parse::<f32>()
        .map_err(|_| Error::InvalidResponse(format!("Not a number: {}", content)))?;

    Ok(score.clamp(0.0, 1.0))
}

/// Collect nodes whose page ranges intersect with [start, end].
fn collect_nodes_in_page_range_recursive(root: &PageNodeRef, start: usize, end: usize) -> Vec<PageNodeRef> {
    let mut result = Vec::new();

    fn traverse(node: &PageNodeRef, start: usize, end: usize, out: &mut Vec<PageNodeRef>) {
        let borrowed = node.borrow();

        // Check if this node intersects with the range
        let intersects = match (borrowed.start_page, borrowed.end_page) {
            (Some(node_start), Some(node_end)) => {
                node_start <= end && node_end >= start
            }
            _ => true, // No page info, include it
        };

        if intersects {
            out.push(node.clone());
        }

        for child in &borrowed.children {
            traverse(child, start, end, out);
        }
    }

    traverse(root, start, end, &mut result);
    result
}

/// Extract page numbers from a section.
fn extract_pages_from_section(node: &PageNodeRef) -> Vec<usize> {
    let borrowed = node.borrow();
    match (borrowed.start_page, borrowed.end_page) {
        (Some(start), Some(end)) => (start..=end).collect(),
        _ => Vec::new(),
    }
}

/// Build enhanced path context from root to target.
fn build_enhanced_path_context(root: &PageNodeRef, path: &[PathStep]) -> Vec<PathStep> {
    // Start from root and build path
    let mut result = Vec::new();
    let mut current = Some(PageNodeRef::clone(root));

    while let Some(node) = current {
        let borrowed = node.borrow();
        result.push(PathStep {
            title: borrowed.title.clone(),
            summary: borrowed.summary.clone(),
            depth: borrowed.depth,
            node_id: borrowed.node_id.clone(),
        });

        // Move to next level
        current = if result.len() < path.len() {
            // Try to find the child that matches the next path step
            let next_step = &path[result.len() - 1];
            let mut found = None;
            for child in &borrowed.children {
                let child_borrowed = child.borrow();
                if child_borrowed.title == next_step.title {
                    found = Some(child.clone());
                    break;
                }
            }
            found
        } else {
            None
        };
    }

    result
}

/// Estimate token count for text.
fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    (text.len() / 4).max(1)
}

// ============================================================
// Error Types
// ============================================================

/// Enhanced retrieval error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Invalid LLM response: {0}")]
    InvalidResponse(String),

    #[error("Node has no children")]
    NoChildren,

    #[error("Retrieval failed: {0}")]
    RetrievalFailed(String),
}

// Re-export original retrieve function and error for backward compatibility
pub use crate::retriever::retrieve;

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::PageNode;

    #[test]
    fn test_retrieve_mode_tree_navigation() {
        let mode = RetrieveMode::TreeNavigation;
        assert!(matches!(mode, RetrieveMode::TreeNavigation));
    }

    #[test]
    fn test_retrieve_mode_page_range() {
        let mode = RetrieveMode::PageRange { start: 5, end: 10 };
        match mode {
            RetrieveMode::PageRange { start, end } => {
                assert_eq!(start, 5);
                assert_eq!(end, 10);
            }
            _ => panic!("Wrong mode"),
        }
    }

    #[test]
    fn test_retrieve_mode_multi_path() {
        let mode = RetrieveMode::MultiPath { top_k: 3 };
        match mode {
            RetrieveMode::MultiPath { top_k } => assert_eq!(top_k, 3),
            _ => panic!("Wrong mode"),
        }
    }

    #[test]
    fn test_retrieve_result_creation() {
        let result = RetrieveResult {
            answer: "Test answer".to_string(),
            sections: vec![],
            path: vec![],
            metadata: RetrieveMetadata::default(),
        };
        assert_eq!(result.answer, "Test answer");
    }

    #[test]
    fn test_retrieved_section_creation() {
        let section = RetrievedSection {
            title: "Test Section".to_string(),
            content: "Test content".to_string(),
            summary: "Test summary".to_string(),
            page_range: Some((1, 5)),
            node_id: Some("0001".to_string()),
            depth: 2,
            relevance_score: Some(0.9),
        };

        assert_eq!(section.title, "Test Section");
        assert_eq!(section.page_range, Some((1, 5)));
        assert_eq!(section.depth, 2);
        assert_eq!(section.relevance_score, Some(0.9));
    }

    #[test]
    fn test_estimate_tokens() {
        let text = "This is a test string with some words.";
        let tokens = estimate_tokens(text);
        assert!(tokens > 0);
        assert!(tokens <= text.len());
    }

    #[test]
    fn test_collect_nodes_in_page_range() {
        let root = PageNode::new("root", "");

        let child1 = PageNode::with_pages("Chapter 1", "", 1, 5);
        child1.borrow_mut().parent = Some(PageNodeRef::clone(&root));

        let child2 = PageNode::with_pages("Chapter 2", "", 6, 10);
        child2.borrow_mut().parent = Some(PageNodeRef::clone(&root));

        root.borrow_mut().children.push(child1.clone());
        root.borrow_mut().children.push(child2.clone());

        // Collect nodes in range 1-5
        let nodes = collect_nodes_in_page_range_recursive(&root, 1, 5);
        assert_eq!(nodes.len(), 2); // root + Chapter 1

        // Collect nodes in range 3-8
        let nodes = collect_nodes_in_page_range_recursive(&root, 3, 8);
        assert_eq!(nodes.len(), 3); // root + Chapter 1 + Chapter 2
    }
}
