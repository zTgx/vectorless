// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Document parsing for building the index tree.

use crate::config::IndexerConfig;
use crate::node::{PageNode, PageNodeRef};
use serde::{Deserialize, Serialize};
use vectorless_llm::chat::{ChatModel, Message, Role, ChatOptions};

/// A section returned by the LLM splitter.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct Section {
    title: String,
    content: String,
}

/// Response format for the segmentation prompt.
#[derive(Debug, Clone, Deserialize, Serialize)]
struct SegmentResponse {
    sections: Vec<Section>,
}

/// Extract JSON from a response that may contain extra text.
fn extract_json(text: &str) -> Result<String, String> {
    let text = text.trim();

    // If the entire text is valid JSON, return it
    if let Ok(_) = serde_json::from_str::<serde_json::Value>(text) {
        return Ok(text.to_string());
    }

    // Try to find JSON object boundaries
    let start = text.find('{').ok_or_else(|| "No JSON object found".to_string())?;
    let end = text.rfind('}').ok_or_else(|| "No JSON object found".to_string())?;

    if start >= end {
        return Err("Invalid JSON object bounds".to_string());
    }

    Ok(text[start..=end].to_string())
}

/// Split text into logical sections using the LLM.
async fn segment<M>(llm: &M, text: &str, max_tokens: u32) -> Result<Vec<Section>, Error>
where
    M: ChatModel,
{
    // Truncate text to avoid context limits
    let truncated = if text.len() > 8000 {
        &text[..8000]
    } else {
        text
    };

    let prompt = format!(
        r#"You are a document parser. Split the following text into logical sections.

Return ONLY a valid JSON object (no additional text). Format:
{{"sections":[{{"title":"short title","content":"text belonging to this section"}},...]}}

Text to parse:
{}"#,
        truncated
    );

    let response = llm
        .chat(
            &[Message {
                role: Role::User,
                content: prompt,
            }],
            &ChatOptions {
                temperature: Some(0.0),
                max_tokens: Some(max_tokens),
            },
        )
        .await
        .map_err(|e| Error::Llm(e.to_string()))?;

    // Extract JSON from response (in case LLM added extra text)
    let json_str = extract_json(&response.content)
        .map_err(|e| Error::InvalidJson(format!("Failed to extract JSON: {}. Response was: {}", e, response.content)))?;

    // Parse JSON response
    let parsed: SegmentResponse = serde_json::from_str(&json_str)
        .map_err(|e| Error::InvalidJson(format!("JSON parse error: {}. Content was: {}", e, json_str)))?;

    Ok(parsed.sections)
}

/// Parse a document into a tree structure using the LLM with custom config.
pub async fn parse_document_with_config<M>(
    llm: &M,
    text: &str,
    config: &IndexerConfig,
) -> Result<PageNodeRef, Error>
where
    M: ChatModel,
{
    let root = PageNode::new("root", "");
    root.borrow_mut().depth = 0;

    // First pass: split into top-level sections
    let sections = segment(llm, text, config.max_segment_tokens as u32).await?;

    for item in sections {
        let title = item.title;
        let content = item.content;

        let node = PageNode::new(&title, "");
        node.borrow_mut().depth = 1;
        node.borrow_mut().parent = Some(PageNodeRef::clone(&root));

        let word_count = content.split_whitespace().count();

        if word_count > config.subsection_threshold {
            // Second pass: split long sections into subsections
            let subsections = segment(llm, &content, config.max_segment_tokens as u32).await?;

            if subsections.len() > 1 {
                for sub in subsections {
                    let child = PageNode::new(&sub.title, &sub.content);
                    child.borrow_mut().depth = 2;
                    child.borrow_mut().parent = Some(PageNodeRef::clone(&node));

                    node.borrow_mut().children.push(PageNodeRef::clone(&child));
                }
            } else {
                // Splitting gave nothing useful, keep as leaf
                node.borrow_mut().content = content;
            }
        } else {
            // Short enough to stay as a leaf
            node.borrow_mut().content = content;
        }

        root.borrow_mut().children.push(PageNodeRef::clone(&node));
    }

    Ok(root)
}

/// Parse a document into a tree structure using the LLM with default config.
pub async fn parse_document<M>(llm: &M, text: &str) -> Result<PageNodeRef, Error>
where
    M: ChatModel,
{
    parse_document_with_config(llm, text, &IndexerConfig::default()).await
}

/// Parse error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Invalid JSON response: {0}")]
    InvalidJson(String),

    #[error("Parsing failed: {0}")]
    ParsingFailed(String),
}
