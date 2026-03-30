// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Tree builder for creating document structures from TOC entries.
//!
//! This module provides functionality to build hierarchical tree structures
//! from table of contents entries, incorporating page boundary information
//! following the PageIndex approach.

use crate::node::{PageNode, PageNodeRef};
use crate::toc::TocEntry;
use crate::pdf::Page;

// ============================================================
// Tree Builder
// ============================================================

/// Builder for creating document tree structures from TOC entries.
pub struct TreeBuilder {
    /// Next available node ID.
    next_id: usize,

    /// Whether to assign node IDs automatically.
    assign_ids: bool,

    /// Whether to include page boundaries in node content.
    include_boundaries: bool,
}

impl TreeBuilder {
    /// Create a new tree builder with default settings.
    pub fn new() -> Self {
        Self {
            next_id: 1,
            assign_ids: true,
            include_boundaries: false,
        }
    }

    /// Create a builder that doesn't assign node IDs.
    pub fn without_ids(mut self) -> Self {
        self.assign_ids = false;
        self
    }

    /// Include page boundary markers in node content.
    pub fn include_boundaries(mut self, value: bool) -> Self {
        self.include_boundaries = value;
        self
    }

    /// Build a tree structure from TOC entries.
    ///
    /// Creates a hierarchical tree from flat TOC entries, using
    /// the level field to determine parent-child relationships.
    pub fn build_from_toc(&mut self, entries: &[TocEntry]) -> PageNodeRef {
        let root = PageNode::new("root", "");
        root.borrow_mut().depth = 0;

        if entries.is_empty() {
            return root;
        }

        // Stack to track current path: (node, level)
        let mut stack: Vec<(PageNodeRef, usize)> = vec![(PageNodeRef::clone(&root), 0)];

        for entry in entries {
            let level = entry.level;

            // Create node for this entry
            let node = if let Some((start, end)) = self.get_page_range(&entry) {
                PageNode::with_pages(&entry.title, "", start, end)
            } else {
                PageNode::new(&entry.title, "")
            };

            // Set physical index if available
            if let Some(ref physical_idx) = entry.physical_index {
                node.borrow_mut().physical_index = Some(physical_idx.clone());
            }

            // Set node ID if enabled
            if self.assign_ids {
                let id = format!("{:04}", self.next_id);
                node.borrow_mut().node_id = Some(id);
                self.next_id += 1;
            }

            // Set depth
            node.borrow_mut().depth = level;

            // Pop stack until we find the parent (level < current level)
            while let Some((_, stack_level)) = stack.last() {
                if *stack_level < level {
                    break;
                }
                stack.pop();
            }

            // Add to parent
            if let Some((parent, _)) = stack.last() {
                parent.borrow_mut().children.push(PageNodeRef::clone(&node));
                node.borrow_mut().parent = Some(PageNodeRef::clone(parent));
            }

            // Push to stack
            stack.push((node, level));
        }

        root
    }

    /// Build a tree structure from TOC entries and populate content from pages.
    ///
    /// This version also extracts and assigns text content from the
    /// document pages based on the page boundaries.
    pub fn build_from_toc_with_content(
        &mut self,
        entries: &[TocEntry],
        pages: &[Page],
    ) -> PageNodeRef {
        let root = self.build_from_toc(entries);

        // Populate content from pages
        self.populate_content_from_pages(&root, pages);

        root
    }

    /// Populate node content from document pages.
    ///
    /// Extracts text from pages based on each node's page range
    /// and assigns it to the node's content field.
    pub fn populate_content_from_pages(&self, root: &PageNodeRef, pages: &[Page]) {
        let nodes = self.collect_all_nodes(root);

        for node in nodes {
            let borrowed = node.borrow();
            if let (Some(start), Some(end)) = (borrowed.start_page, borrowed.end_page) {
                drop(borrowed);

                // Extract content from pages
                let content = extract_page_range(pages, start, end);
                node.borrow_mut().content = content;
            }
        }
    }

    /// Get page range for a TOC entry.
    fn get_page_range(&self, entry: &TocEntry) -> Option<(usize, usize)> {
        // Try to extract from physical_index first
        if let Some(ref physical_idx) = entry.physical_index {
            if let Some(page_num) = extract_page_number(physical_idx) {
                // For single page references, we need to determine the range
                // This is a simplified approach - in practice, you'd need
                // to look at the next entry to determine the end
                return Some((page_num, page_num));
            }
        }

        // Fall back to page field
        entry.page.map(|p| (p, p))
    }

    /// Collect all nodes in the tree (BFS).
    fn collect_all_nodes(&self, root: &PageNodeRef) -> Vec<PageNodeRef> {
        let mut result = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(PageNodeRef::clone(root));

        while let Some(node) = queue.pop_front() {
            {
                let borrowed = node.borrow();
                for child in &borrowed.children {
                    queue.push_back(PageNodeRef::clone(child));
                }
            } // Drop borrow before moving node
            result.push(node);
        }

        result
    }

    /// Assign sequential node IDs to all nodes in the tree.
    pub fn assign_node_ids(&mut self, root: &PageNodeRef) {
        let nodes = self.collect_all_nodes(root);
        for (i, node) in nodes.iter().enumerate() {
            let id = format!("{:04}", i + 1);
            node.borrow_mut().node_id = Some(id);
        }
        self.next_id = nodes.len() + 1;
    }
}

impl Default for TreeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Helper Functions
// ============================================================

/// Extract page number from physical_index marker.
///
/// Parses strings like "<physical_index_5>" to extract the number 5.
pub fn extract_page_number(physical_index: &str) -> Option<usize> {
    let re = regex::Regex::new(r"<physical_index_(\d+)>").unwrap();
    re.captures(physical_index)?
        .get(1)
        .map(|m| m.as_str().parse().ok())
        .flatten()
}

/// Extract text content from a range of pages.
///
/// Concatenates content from pages start to end (inclusive, 1-based).
pub fn extract_page_range(pages: &[Page], start: usize, end: usize) -> String {
    if start < 1 || end > pages.len() || start > end {
        return String::new();
    }

    pages[start - 1..end]
        .iter()
        .map(|p| p.content.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Extract text content from a range of pages with boundary markers.
///
/// Includes `<physical_index_X>` markers between pages.
pub fn extract_page_range_with_boundaries(pages: &[Page], start: usize, end: usize) -> String {
    if start < 1 || end > pages.len() || start > end {
        return String::new();
    }

    pages[start - 1..end]
        .iter()
        .map(|p| p.with_boundaries())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Find the node that contains a specific page.
///
/// Traverses the tree to find the node whose page range includes
/// the given page number.
pub fn find_node_for_page(root: &PageNodeRef, page: usize) -> Option<PageNodeRef> {
    use crate::node::PageNodeRefExt;

    // Check if root itself contains the page
    if root.contains_page(page) {
        // But root is a special case, we want the actual content node
        // So continue to children
    }

    let borrowed = root.borrow();
    for child in &borrowed.children {
        if child.contains_page(page) {
            // Check if this child has children that might be more specific
            let child_result = find_node_for_page(child, page);
            if let Some(found) = child_result {
                return Some(found);
            }
            // If no more specific child found, return this child
            return Some(PageNodeRef::clone(child));
        }
    }

    None
}

/// Collect all nodes within a specific page range.
///
/// Returns all nodes that intersect with the given page range.
pub fn collect_nodes_in_page_range(root: &PageNodeRef, start: usize, end: usize) -> Vec<PageNodeRef> {
    let mut result = Vec::new();
    collect_nodes_in_range_recursive(root, start, end, &mut result);
    result
}

fn collect_nodes_in_range_recursive(
    node: &PageNodeRef,
    start: usize,
    end: usize,
    result: &mut Vec<PageNodeRef>,
) {
    let borrowed = node.borrow();

    // Check if this node intersects with the range
    let intersects = match (borrowed.start_page, borrowed.end_page) {
        (Some(node_start), Some(node_end)) => {
            node_start <= end && node_end >= start
        }
        _ => false,
    };

    if intersects {
        result.push(PageNodeRef::clone(node));
    }

    for child in &borrowed.children {
        collect_nodes_in_range_recursive(child, start, end, result);
    }
}

/// Get the path from root to a specific node.
///
/// Returns a list of node titles from root to the target node.
pub fn get_path_to_node(node: &PageNodeRef) -> Vec<String> {
    let mut path = Vec::new();
    let mut current = Some(PageNodeRef::clone(node));

    while let Some(n) = current {
        let borrowed = n.borrow();
        path.push(borrowed.title.clone());
        current = borrowed.parent.as_ref().map(|p| PageNodeRef::clone(p));
    }

    path.reverse();
    path
}

/// Validate that all page boundaries are within document range.
///
/// Checks that all page numbers are valid for the given document.
pub fn validate_page_boundaries(root: &PageNodeRef, total_pages: usize) -> Result<(), ValidationError> {
    let nodes = collect_all_nodes_bfs(root);

    for node in nodes {
        let borrowed = node.borrow();
        if let (Some(start), Some(end)) = (borrowed.start_page, borrowed.end_page) {
            if start < 1 || end > total_pages {
                return Err(ValidationError::PageOutOfRange {
                    title: borrowed.title.clone(),
                    start,
                    end,
                    total_pages,
                });
            }
            if start > end {
                return Err(ValidationError::InvalidPageRange {
                    title: borrowed.title.clone(),
                    start,
                    end,
                });
            }
        }
    }

    Ok(())
}

/// Collect all nodes using BFS (helper function).
fn collect_all_nodes_bfs(root: &PageNodeRef) -> Vec<PageNodeRef> {
    let mut result = Vec::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(PageNodeRef::clone(root));

    while let Some(node) = queue.pop_front() {
        {
            let borrowed = node.borrow();
            for child in &borrowed.children {
                queue.push_back(PageNodeRef::clone(child));
            }
        } // Drop borrow before moving node
        result.push(node);
    }

    result
}

// ============================================================
// Error Types
// ============================================================

/// Page boundary validation error.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Page number out of range for node '{title}': range {start}-{end}, document has {total_pages} pages")]
    PageOutOfRange {
        title: String,
        start: usize,
        end: usize,
        total_pages: usize,
    },

    #[error("Invalid page range for node '{title}': start {start} > end {end}")]
    InvalidPageRange {
        title: String,
        start: usize,
        end: usize,
    },
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::toc::TocEntry;

    #[test]
    fn test_extract_page_number() {
        let physical = "<physical_index_5>";
        assert_eq!(extract_page_number(physical), Some(5));
    }

    #[test]
    fn test_extract_page_number_invalid() {
        let physical = "<physical_index_abc>";
        assert_eq!(extract_page_number(physical), None);
    }

    #[test]
    fn test_extract_page_number_none() {
        let physical = "no index here";
        assert_eq!(extract_page_number(physical), None);
    }

    #[test]
    fn test_extract_page_range() {
        use crate::pdf::Page;

        let pages = vec![
            Page::new(1, "Page 1 content".to_string()),
            Page::new(2, "Page 2 content".to_string()),
            Page::new(3, "Page 3 content".to_string()),
        ];

        let result = extract_page_range(&pages, 1, 2);
        assert!(result.contains("Page 1 content"));
        assert!(result.contains("Page 2 content"));
        assert!(!result.contains("Page 3 content"));
    }

    #[test]
    fn test_extract_page_range_with_boundaries() {
        use crate::pdf::Page;

        let pages = vec![
            Page::new(1, "Page 1".to_string()),
            Page::new(2, "Page 2".to_string()),
        ];

        let result = extract_page_range_with_boundaries(&pages, 1, 2);
        assert!(result.contains("<physical_index_1>"));
        assert!(result.contains("<physical_index_2>"));
    }

    #[test]
    fn test_tree_builder_basic() {
        let entries = vec![
            TocEntry::new("Chapter 1", 1),
            TocEntry::new("Section 1.1", 2),
            TocEntry::new("Chapter 2", 1),
        ];

        let mut builder = TreeBuilder::new();
        let root = builder.build_from_toc(&entries);

        let borrowed = root.borrow();
        assert_eq!(borrowed.children.len(), 2); // Two chapters

        // First chapter
        let ch1 = &borrowed.children[0];
        assert_eq!(ch1.borrow().title, "Chapter 1");
        assert_eq!(ch1.borrow().children.len(), 1); // One section

        // Section 1.1
        let sec = &ch1.borrow().children[0];
        assert_eq!(sec.borrow().title, "Section 1.1");
    }

    #[test]
    fn test_tree_builder_with_ids() {
        let entries = vec![
            TocEntry::new("Chapter 1", 1),
            TocEntry::new("Chapter 2", 1),
        ];

        let mut builder = TreeBuilder::new().without_ids();
        let root = builder.build_from_toc(&entries);

        // Root doesn't get an ID
        assert!(root.borrow().node_id.is_none());
    }

    #[test]
    fn test_find_node_for_page() {
        let root = PageNode::new("root", "");

        let child1 = PageNode::with_pages("Chapter 1", "", 1, 5);
        child1.borrow_mut().parent = Some(PageNodeRef::clone(&root));

        let child2 = PageNode::with_pages("Section 1.1", "", 2, 3);
        child2.borrow_mut().parent = Some(PageNodeRef::clone(&child1));

        root.borrow_mut().children.push(child1.clone());
        child1.borrow_mut().children.push(child2.clone());

        // Find page 2 - should return Section 1.1
        let found = find_node_for_page(&root, 2);
        assert!(found.is_some());
        assert_eq!(found.unwrap().borrow().title, "Section 1.1");
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
        let nodes = collect_nodes_in_page_range(&root, 1, 5);
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].borrow().title, "Chapter 1");

        // Collect nodes in range 3-8
        let nodes = collect_nodes_in_page_range(&root, 3, 8);
        assert_eq!(nodes.len(), 2); // Both chapters intersect
    }

    #[test]
    fn test_get_path_to_node() {
        let root = PageNode::new("root", "");

        let child1 = PageNode::new("Chapter 1", "");
        child1.borrow_mut().parent = Some(PageNodeRef::clone(&root));

        let child2 = PageNode::new("Section 1.1", "");
        child2.borrow_mut().parent = Some(PageNodeRef::clone(&child1));

        root.borrow_mut().children.push(child1.clone());
        child1.borrow_mut().children.push(child2.clone());

        let path = get_path_to_node(&child2);
        assert_eq!(path, vec!["root", "Chapter 1", "Section 1.1"]);
    }

    #[test]
    fn test_validate_page_boundaries() {
        let root = PageNode::new("root", "");

        let child1 = PageNode::with_pages("Chapter 1", "", 1, 10);
        child1.borrow_mut().parent = Some(PageNodeRef::clone(&root));

        root.borrow_mut().children.push(child1.clone());

        // Valid - should pass
        assert!(validate_page_boundaries(&root, 10).is_ok());

        // Invalid - page out of range
        let result = validate_page_boundaries(&root, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_invalid_range() {
        let root = PageNode::new("root", "");

        let child1 = PageNode::with_pages("Chapter 1", "", 10, 5); // start > end
        child1.borrow_mut().parent = Some(PageNodeRef::clone(&root));

        root.borrow_mut().children.push(child1.clone());

        let result = validate_page_boundaries(&root, 10);
        assert!(result.is_err());
    }
}
