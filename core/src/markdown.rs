// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Markdown document parser - builds index tree from Markdown header structure.
//!
//! This module provides functionality to parse Markdown documents and build
//! a hierarchical tree structure based on the header levels (# ## ### etc.).
//!
//! # Example
//!
//! ```no_run
//! use vectorless_core::markdown::{parse_markdown_with_config, MdConfig};
//! use vectorless_llm::openai::OpenAIClient;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let content = std::fs::read_to_string("document.md")?;
//! let llm = OpenAIClient::new(std::env::var("OPENAI_API_KEY")?);
//!
//! let config = MdConfig {
//!     thinning: true,
//!     thinning_threshold: 5000,
//!     generate_summary: true,
//!     summary_threshold: 200,
//! };
//!
//! let result = parse_markdown_with_config(&llm, &content, &config).await?;
//! println!("Parsed {} lines into {} nodes", result.line_count, result.node_count);
//! # Ok(())
//! # }
//! ```

use crate::node::PageNodeRef;
use regex::Regex;
use std::collections::HashSet;
use vectorless_llm::chat::{ChatModel, Message, Role, ChatOptions};

// ============================================================
// Data Structures
// ============================================================

/// Raw Markdown node (flat list stage)
#[derive(Debug, Clone)]
struct MdRawNode {
    title: String,
    level: usize,
    line_num: usize,
    text: String,
    text_token_count: Option<usize>,
}

/// Result of Markdown parsing
pub struct MdParseResult {
    /// Root of the document tree
    pub root: PageNodeRef,
    /// Total number of lines in the document
    pub line_count: usize,
    /// Total number of nodes in the tree (excluding root)
    pub node_count: usize,
}

/// Configuration for Markdown parsing
#[derive(Debug, Clone)]
pub struct MdConfig {
    /// Whether to perform tree thinning (merge small nodes into parent)
    pub thinning: bool,
    /// Token threshold: nodes with fewer tokens are merged into parent
    pub thinning_threshold: usize,
    /// Whether to generate summaries
    pub generate_summary: bool,
    /// Summary token threshold: only generate summary if content exceeds this
    pub summary_threshold: usize,
}

impl Default for MdConfig {
    fn default() -> Self {
        Self {
            thinning: false,
            thinning_threshold: 5000,
            generate_summary: true,
            summary_threshold: 200,
        }
    }
}

impl MdConfig {
    /// Create a new builder for MdConfig
    pub fn builder() -> MdConfigBuilder {
        MdConfigBuilder::default()
    }
}

/// Builder for MdConfig
#[derive(Debug, Clone, Default)]
pub struct MdConfigBuilder {
    config: MdConfig,
}

impl MdConfigBuilder {
    /// Set thinning flag
    pub fn thinning(mut self, value: bool) -> Self {
        self.config.thinning = value;
        self
    }

    /// Set thinning threshold
    pub fn thinning_threshold(mut self, value: usize) -> Self {
        self.config.thinning_threshold = value;
        self
    }

    /// Set generate summary flag
    pub fn generate_summary(mut self, value: bool) -> Self {
        self.config.generate_summary = value;
        self
    }

    /// Set summary threshold
    pub fn summary_threshold(mut self, value: usize) -> Self {
        self.config.summary_threshold = value;
        self
    }

    /// Build the config
    pub fn build(self) -> MdConfig {
        self.config
    }
}

// ============================================================
// Core Parsing Functions
// ============================================================

/// Parse Markdown content and build tree structure with default config
pub async fn parse_markdown<M>(
    llm: &M,
    content: &str,
) -> Result<MdParseResult, Error>
where
    M: ChatModel,
{
    parse_markdown_with_config(llm, content, &MdConfig::default()).await
}

/// Parse Markdown content and build tree structure with custom config
pub async fn parse_markdown_with_config<M>(
    llm: &M,
    content: &str,
    config: &MdConfig,
) -> Result<MdParseResult, Error>
where
    M: ChatModel,
{
    let lines: Vec<&str> = content.lines().collect();
    let line_count = lines.len();

    // Step 1: Extract header nodes
    let mut nodes = extract_header_nodes(&lines)?;

    // Step 2: Extract text content for each node
    extract_node_text(&mut nodes, &lines);

    // Step 3: Calculate token counts (recursively including children)
    update_token_counts(&mut nodes, llm);

    // Step 4: Optional - tree thinning
    if config.thinning {
        thin_tree(&mut nodes, config.thinning_threshold, llm);
    }

    // Step 5: Build tree structure
    let root = build_tree(nodes)?;
    let node_count = count_nodes(&root);

    // Step 6: Optional - generate summaries
    if config.generate_summary {
        generate_summaries(llm, &root, config.summary_threshold).await?;
    }

    Ok(MdParseResult {
        root,
        line_count,
        node_count,
    })
}

// ============================================================
// Step 1: Extract Header Nodes
// ============================================================

/// Extract header nodes from Markdown lines
fn extract_header_nodes(lines: &[&str]) -> Result<Vec<MdRawNode>, Error> {
    let header_re = Regex::new(r"^(#{1,6})\s+(.+)$").unwrap();
    let code_block_re = Regex::new(r"^```").unwrap();

    let mut nodes = Vec::new();
    let mut in_code_block = false;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = line.trim();

        // Detect code block boundaries
        if code_block_re.is_match(trimmed) {
            in_code_block = !in_code_block;
            continue;
        }

        // Skip empty lines and content inside code blocks
        if trimmed.is_empty() || in_code_block {
            continue;
        }

        // Extract header
        if let Some(caps) = header_re.captures(trimmed) {
            let level = caps[1].len();
            let title = caps[2].trim().to_string();

            nodes.push(MdRawNode {
                title,
                level,
                line_num,
                text: String::new(),
                text_token_count: None,
            });
        }
    }

    if nodes.is_empty() {
        return Err(Error::NoHeaders);
    }

    Ok(nodes)
}

// ============================================================
// Step 2: Extract Node Text Content
// ============================================================

/// Extract text content for each node (from header to next same-level/higher-level header)
fn extract_node_text(nodes: &mut [MdRawNode], lines: &[&str]) {
    for i in 0..nodes.len() {
        let start_line = nodes[i].line_num - 1; // Convert to 0-based

        // Find end position: next same-level or higher-level header
        let end_line = if i + 1 < nodes.len() {
            nodes[i + 1].line_num - 1
        } else {
            lines.len()
        };

        // Extract text
        let text: String = lines[start_line..end_line].join("\n");
        nodes[i].text = text;
    }
}

// ============================================================
// Step 3: Calculate Token Counts (Recursive)
// ============================================================

/// Update token counts for each node (recursively including all children)
fn update_token_counts<M: ChatModel>(nodes: &mut [MdRawNode], llm: &M) {
    // Process from back to front to ensure children are processed first
    for i in (0..nodes.len()).rev() {
        let children = find_all_children(i, nodes);

        // Own token count
        let own_tokens = count_tokens(&nodes[i].text, llm);

        // Add all children's token counts
        let total_tokens = own_tokens
            + children
                .iter()
                .map(|child_idx| nodes[*child_idx].text_token_count.unwrap_or(0))
                .sum::<usize>();

        nodes[i].text_token_count = Some(total_tokens);
    }
}

/// Find all children (direct and indirect) of a node
fn find_all_children(parent_idx: usize, nodes: &[MdRawNode]) -> Vec<usize> {
    let parent_level = nodes[parent_idx].level;
    let mut children = Vec::new();

    for i in (parent_idx + 1)..nodes.len() {
        if nodes[i].level <= parent_level {
            break; // Encountered same or higher level, stop
        }
        children.push(i);
    }

    children
}

/// Estimate token count (1 token ≈ 4 characters)
fn count_tokens<M: ChatModel>(text: &str, _llm: &M) -> usize {
    if text.is_empty() {
        return 0;
    }
    // Simple estimation: 1 token ≈ 4 characters
    // For more accurate counting, integrate tiktoken-rs
    (text.len() / 4).max(1)
}

// ============================================================
// Step 4: Tree Thinning
// ============================================================

/// Tree thinning: merge small nodes into parent
fn thin_tree<M: ChatModel>(
    nodes: &mut Vec<MdRawNode>,
    threshold: usize,
    llm: &M,
) {
    let mut to_remove = HashSet::new();

    // Process from back to front
    for i in (0..nodes.len()).rev() {
        if to_remove.contains(&i) {
            continue;
        }

        let tokens = nodes[i].text_token_count.unwrap_or(0);

        if tokens < threshold {
            let children = find_all_children(i, nodes);

            // Merge child node texts into current node
            let mut merged_text = nodes[i].text.clone();
            let mut sorted_children: Vec<_> = children.iter().collect();
            sorted_children.sort();
            for child_idx in sorted_children {
                if !to_remove.contains(child_idx) {
                    if !merged_text.is_empty() {
                        merged_text.push_str("\n\n");
                    }
                    merged_text.push_str(&nodes[*child_idx].text);
                    to_remove.insert(*child_idx);
                }
            }
            let token_count = count_tokens(&merged_text, llm);
            nodes[i].text = merged_text;
            nodes[i].text_token_count = Some(token_count);
        }
    }

    // Remove merged nodes
    let new_nodes: Vec<MdRawNode> = nodes
        .iter()
        .enumerate()
        .filter(|(i, _)| !to_remove.contains(i))
        .map(|(_, node)| node.clone())
        .collect();

    *nodes = new_nodes;
}

// ============================================================
// Step 5: Build Tree Structure
// ============================================================

/// Convert flat node list to tree structure
fn build_tree(nodes: Vec<MdRawNode>) -> Result<PageNodeRef, Error> {
    use crate::node::PageNode;

    let root = PageNode::new("root", "");
    root.borrow_mut().depth = 0;

    let mut stack: Vec<(PageNodeRef, usize)> = vec![(PageNodeRef::clone(&root), 0)];

    for raw_node in nodes {
        let level = raw_node.level;

        // Create new node
        let node = PageNode::new(&raw_node.title, &raw_node.text);
        node.borrow_mut().depth = level;

        // Pop nodes with level >= current level from stack
        while let Some((_, stack_level)) = stack.last() {
            if *stack_level >= level {
                stack.pop();
            } else {
                break;
            }
        }

        // Add to parent node
        if let Some((parent, _)) = stack.last() {
            parent.borrow_mut().children.push(PageNodeRef::clone(&node));
            node.borrow_mut().parent = Some(PageNodeRef::clone(parent));
        }

        // Push to stack
        stack.push((node, level));
    }

    Ok(root)
}

/// Count total nodes in tree (excluding root)
fn count_nodes(root: &PageNodeRef) -> usize {
    let borrowed = root.borrow();
    let mut count = 0;
    for child in &borrowed.children {
        count += count_nodes_recursive(child);
    }
    count
}

fn count_nodes_recursive(node: &PageNodeRef) -> usize {
    let borrowed = node.borrow();
    let mut count = 1;
    for child in &borrowed.children {
        count += count_nodes_recursive(child);
    }
    count
}

// ============================================================
// Step 6: Generate Summaries
// ============================================================

/// Generate summaries for all nodes in the tree
async fn generate_summaries<M: ChatModel>(
    llm: &M,
    root: &PageNodeRef,
    threshold: usize,
) -> Result<(), Error> {
    use futures::future::join_all;

    let nodes = collect_all_nodes(root);

    // Concurrently generate summaries
    let tasks: Vec<_> = nodes
        .iter()
        .filter_map(|node| {
            let borrowed = node.borrow();
            let text = borrowed.content.clone();
            let title = borrowed.title.clone();
            let tokens = text.len() / 4;
            drop(borrowed);

            if tokens >= threshold {
                Some(summarize_node(llm, node.clone(), text, title))
            } else {
                node.borrow_mut().summary = text.clone();
                None
            }
        })
        .collect();

    join_all(tasks).await;

    Ok(())
}

/// Collect all nodes in the tree
fn collect_all_nodes(root: &PageNodeRef) -> Vec<PageNodeRef> {
    let mut result = Vec::new();
    let mut stack = vec![PageNodeRef::clone(root)];

    while let Some(node) = stack.pop() {
        {
            let borrowed = node.borrow();
            for child in &borrowed.children {
                stack.push(PageNodeRef::clone(child));
            }
        } // Drop borrow before moving node
        result.push(node);
    }

    result
}

/// Generate summary for a single node
async fn summarize_node<M: ChatModel>(
    llm: &M,
    node: PageNodeRef,
    text: String,
    title: String,
) {
    let truncated = if text.len() > 3000 {
        format!("{}...", &text[..3000])
    } else {
        text.clone()
    };

    let prompt = format!(
        "Summarize the following section titled '{}' in 2-3 sentences. Be specific and factual:\n\n{}",
        title, truncated
    );

    let response = llm
        .chat(
            &[Message {
                role: Role::User,
                content: prompt,
            }],
            &ChatOptions {
                temperature: Some(0.0),
                max_tokens: Some(150),
            },
        )
        .await;

    if let Ok(resp) = response {
        node.borrow_mut().summary = resp.content.trim().to_string();
    }
}

// ============================================================
// Error Types
// ============================================================

/// Markdown parsing error types
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No headers found in Markdown content")]
    NoHeaders,

    #[error("Invalid header at line {0}")]
    InvalidHeader(usize),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Tree build failed: {0}")]
    TreeBuildFailed(String),
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Mock LLM for testing
    struct MockLlm;

    #[async_trait::async_trait]
    impl vectorless_llm::chat::ChatModel for MockLlm {
        async fn chat(
            &self,
            _messages: &[vectorless_llm::chat::Message],
            _options: &vectorless_llm::chat::ChatOptions,
        ) -> Result<vectorless_llm::chat::ChatCompletion, vectorless_llm::chat::Error> {
            Ok(vectorless_llm::chat::ChatCompletion {
                content: "Test summary".to_string(),
                finish_reason: Some("stop".to_string()),
            })
        }
    }

    #[test]
    fn test_extract_header_nodes() {
        let content = r#"
# Title 1

Some content.

## Title 1.1

More content.

### Title 1.1.1

Details.

# Title 2

Final content.
"#;

        let lines: Vec<&str> = content.lines().collect();
        let nodes = extract_header_nodes(&lines).unwrap();

        assert_eq!(nodes.len(), 4);
        assert_eq!(nodes[0].level, 1);
        assert_eq!(nodes[0].title, "Title 1");
        assert_eq!(nodes[1].level, 2);
        assert_eq!(nodes[1].title, "Title 1.1");
    }

    #[test]
    fn test_extract_header_nodes_skips_code_blocks() {
        let content = r#"
# Title 1

Content before code.

```
# This is not a header
# Also not a header
```

## Title 1.1

Content after code.
"#;

        let lines: Vec<&str> = content.lines().collect();
        let nodes = extract_header_nodes(&lines).unwrap();

        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].title, "Title 1");
        assert_eq!(nodes[1].title, "Title 1.1");
    }

    #[test]
    fn test_extract_node_text() {
        let content = r#"
# Title 1

Content under title 1.

## Title 1.1

Content under title 1.1.

# Title 2

Content under title 2.
"#;

        let lines: Vec<&str> = content.lines().collect();
        let mut nodes = vec![
            MdRawNode {
                title: "Title 1".to_string(),
                level: 1,
                line_num: 2,
                text: String::new(),
                text_token_count: None,
            },
            MdRawNode {
                title: "Title 1.1".to_string(),
                level: 2,
                line_num: 6,
                text: String::new(),
                text_token_count: None,
            },
            MdRawNode {
                title: "Title 2".to_string(),
                level: 1,
                line_num: 10,
                text: String::new(),
                text_token_count: None,
            },
        ];

        extract_node_text(&mut nodes, &lines);

        assert!(nodes[0].text.contains("Content under title 1."));
        assert!(nodes[0].text.contains("# Title 1"));
        // nodes[0].text contains content up to (but not including) the next header at line 6
        assert!(!nodes[0].text.contains("## Title 1.1"));
        assert!(!nodes[0].text.contains("Content under title 2."));
    }

    #[test]
    fn test_find_all_children() {
        let nodes = vec![
            MdRawNode {
                title: "Level 1".to_string(),
                level: 1,
                line_num: 1,
                text: String::new(),
                text_token_count: None,
            },
            MdRawNode {
                title: "Level 2".to_string(),
                level: 2,
                line_num: 2,
                text: String::new(),
                text_token_count: None,
            },
            MdRawNode {
                title: "Level 3".to_string(),
                level: 3,
                line_num: 3,
                text: String::new(),
                text_token_count: None,
            },
            MdRawNode {
                title: "Level 2 again".to_string(),
                level: 2,
                line_num: 4,
                text: String::new(),
                text_token_count: None,
            },
            MdRawNode {
                title: "Level 1 again".to_string(),
                level: 1,
                line_num: 5,
                text: String::new(),
                text_token_count: None,
            },
        ];

        // Children of first node (index 0)
        let children = find_all_children(0, &nodes);
        assert_eq!(children, vec![1, 2, 3]);

        // Children of second node (index 1)
        let children = find_all_children(1, &nodes);
        assert_eq!(children, vec![2]);
    }

    #[test]
    fn test_count_tokens() {
        let llm = MockLlm;
        let text = "This is a test string with some words.";
        let tokens = count_tokens(text, &llm);
        assert!(tokens > 0);
    }

    #[test]
    fn test_no_headers_error() {
        let content = "This is just plain text\nwith no headers.";
        let lines: Vec<&str> = content.lines().collect();
        let result = extract_header_nodes(&lines);
        assert!(matches!(result, Err(Error::NoHeaders)));
    }

    #[tokio::test]
    async fn test_parse_markdown_basic() {
        let content = r#"
# Main Title

Some content here.

## Section 1

Content for section 1.

### Subsection 1.1

Subsection content.

## Section 2

Content for section 2.
"#;

        let llm = MockLlm;
        let result = parse_markdown(&llm, content).await.unwrap();

        assert_eq!(result.node_count, 4); // Section 1, Subsection 1.1, Section 2
        assert!(result.line_count > 0);
    }
}
