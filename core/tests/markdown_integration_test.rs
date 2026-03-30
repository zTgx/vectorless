// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Integration tests for Markdown parsing.

use vectorless_core::markdown::{parse_markdown_with_config, MdConfig};

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
            content: "Mock summary for testing purposes.".to_string(),
            finish_reason: Some("stop".to_string()),
        })
    }
}

#[tokio::test]
async fn test_simple_markdown_parsing() {
    let content = r#"# Introduction

This is the introduction section with some text.

## Background

More background information here.

### History

Historical context.

## Methodology

The methodology section.

# Conclusion

Final thoughts."#;

    let llm = MockLlm;
    let result = parse_markdown_with_config(&llm, content, &MdConfig::default())
        .await
        .unwrap();

    // Should have 5 sections: Introduction, Background, History, Methodology, Conclusion
    assert_eq!(result.node_count, 5);
    assert!(result.line_count > 10);
}

#[tokio::test]
async fn test_markdown_with_code_blocks() {
    let content = r#"# Code Example

Here's some code:

```rust
fn main() {
    println!("Hello");
}
```

## Analysis

The code above prints Hello."#;

    let llm = MockLlm;
    let result = parse_markdown_with_config(&llm, content, &MdConfig::default())
        .await
        .unwrap();

    // Should only find 2 headers: Code Example and Analysis
    assert_eq!(result.node_count, 2);
}

#[tokio::test]
async fn test_deeply_nested_headers() {
    let content = r#"
# Level 1

## Level 2

### Level 3

#### Level 4

##### Level 5

###### Level 6
"#;

    let llm = MockLlm;
    let result = parse_markdown_with_config(&llm, content, &MdConfig::default())
        .await
        .unwrap();

    // All 6 levels should be parsed
    assert_eq!(result.node_count, 6);
}

#[tokio::test]
async fn test_markdown_thinning() {
    let content = r#"
# Main Section

## Small Section 1

Brief content.

## Small Section 2

More brief content.

## Large Section

This section has a lot more content that should exceed the thinning threshold when combined with other sections.
"#;

    let llm = MockLlm;
    let config = MdConfig {
        thinning: true,
        thinning_threshold: 100,
        generate_summary: false,
        summary_threshold: 0,
    };

    let result = parse_markdown_with_config(&llm, content, &config)
        .await
        .unwrap();

    // With thinning, small nodes should be merged
    // The exact count depends on content size
    assert!(result.node_count > 0);
}

#[tokio::test]
async fn test_markdown_without_summaries() {
    let content = r#"
# Title

Content here.

## Subtitle

More content.
"#;

    let llm = MockLlm;
    let config = MdConfig {
        thinning: false,
        generate_summary: false,
        ..Default::default()
    };

    let result = parse_markdown_with_config(&llm, content, &config)
        .await
        .unwrap();

    // Verify summaries are empty when generation is disabled
    let root = &result.root;
    let borrowed = root.borrow();

    if let Some(first_child) = borrowed.children.first() {
        let child_borrowed = first_child.borrow();
        assert!(child_borrowed.summary.is_empty() || child_borrowed.summary == child_borrowed.content);
    }
}

#[tokio::test]
async fn test_empty_markdown() {
    let content = "Just some text with no headers.";

    let llm = MockLlm;
    let result = parse_markdown_with_config(&llm, content, &MdConfig::default()).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_tree_structure_hierarchy() {
    let content = r#"
# Chapter 1

Content of chapter 1.

## Section 1.1

Content of section 1.1.

### Subsection 1.1.1

Content of subsection.

## Section 1.2

Content of section 1.2.

# Chapter 2

Content of chapter 2.
"#;

    let llm = MockLlm;
    let result = parse_markdown_with_config(&llm, content, &MdConfig::default())
        .await
        .unwrap();

    let root = &result.root;
    let borrowed = root.borrow();

    // Should have 2 chapters at top level
    assert_eq!(borrowed.children.len(), 2);

    // First chapter should have 2 sections
    let first_chapter = borrowed.children[0].borrow();
    assert_eq!(first_chapter.children.len(), 2);

    // First section should have 1 subsection
    let first_section = first_chapter.children[0].borrow();
    assert_eq!(first_section.children.len(), 1);
}
