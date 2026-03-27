// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Index builder: traverse tree and build summaries.

use crate::config::IndexerConfig;
use crate::node::PageNodeRef;
use vectorless_llm::chat::{ChatModel, Message, Role, ChatOptions};

/// Build summaries for all nodes in the tree with custom config.
///
/// Traverses the tree post-order (children before parent).
/// - Leaves summarize their own content
/// - Inner nodes summarize their children's summaries
pub async fn build_summaries_with_config<M>(
    llm: &M,
    root: &PageNodeRef,
    config: &IndexerConfig,
) -> Result<(), Error>
where
    M: ChatModel,
{
    // Post-order traversal using explicit stack to avoid async recursion
    // Stack contains (node, visited_children flag)
    let mut stack: Vec<(PageNodeRef, bool)> = vec![(PageNodeRef::clone(root), false)];

    while let Some((node, children_visited)) = stack.pop() {
        if children_visited {
            // Children have been processed, now build summary for this node
            let borrowed = node.borrow();
            let is_leaf = borrowed.is_leaf();
            let title = borrowed.title.clone();
            let content = borrowed.content.clone();
            drop(borrowed);

            let summary = if is_leaf {
                if content.trim().is_empty() {
                    "(empty section)".to_string()
                } else {
                    summarize(llm, &content, &title, config).await?
                }
            } else {
                // Build parent summary from children's summaries
                let children_summaries: Vec<String> = {
                    let borrowed = node.borrow();
                    borrowed
                        .children
                        .iter()
                        .map(|c| {
                            let child_borrowed = c.borrow();
                            format!("[{}]: {}", child_borrowed.title, child_borrowed.summary)
                        })
                        .collect()
                };
                let children_text = children_summaries.join("\n\n");
                summarize(llm, &children_text, &title, config).await?
            };

            node.borrow_mut().summary = summary;
        } else {
            // First time seeing this node: push it back with visited=true,
            // then push all children
            stack.push((PageNodeRef::clone(&node), true));

            // Push children in reverse order to maintain correct processing order
            let children: Vec<PageNodeRef> = {
                let borrowed = node.borrow();
                borrowed.children.iter().map(|c| PageNodeRef::clone(c)).collect()
            };

            for child in children.into_iter().rev() {
                stack.push((child, false));
            }
        }
    }

    Ok(())
}

/// Build summaries for all nodes in the tree with default config.
pub async fn build_summaries<M>(llm: &M, root: &PageNodeRef) -> Result<(), Error>
where
    M: ChatModel,
{
    build_summaries_with_config(llm, root, &IndexerConfig::default()).await
}

/// Summarize text using the LLM.
async fn summarize<M>(
    llm: &M,
    text: &str,
    section_name: &str,
    config: &IndexerConfig,
) -> Result<String, Error>
where
    M: ChatModel,
{
    // Truncate text to avoid context limits
    let truncated = if text.len() > 3000 {
        &text[..3000]
    } else {
        text
    };

    let hint = if section_name.is_empty() {
        String::new()
    } else {
        format!("This is the section titled: {}.\n", section_name)
    };

    let prompt = format!(
        "{}Summarize the following in 2-3 sentences. Be specific and factual. Do not add anything not in the text.\n\n{}",
        hint, truncated
    );

    let response = llm
        .chat(
            &[Message {
                role: Role::User,
                content: prompt,
            }],
            &ChatOptions {
                temperature: Some(0.0),
                max_tokens: Some(config.max_summary_tokens),
            },
        )
        .await
        .map_err(|e| Error::Llm(e.to_string()))?;

    Ok(response.content.trim().to_string())
}

/// Indexer error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Summary generation failed: {0}")]
    SummaryFailed(String),
}
