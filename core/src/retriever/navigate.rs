// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Retriever: navigate the tree to find relevant content.

use crate::node::PageNodeRef;
use vectorless_llm::chat::{ChatModel, Message, Role, ChatOptions};

/// Retrieve the most relevant content for a query by navigating the tree.
///
/// Starting at root, the LLM reads the children's summaries and picks the best branch.
/// Repeats until reaching a leaf node, then returns its content.
pub async fn retrieve<M>(llm: &M, query: &str, root: &PageNodeRef) -> Result<String, Error>
where
    M: ChatModel,
{
    let mut node = PageNodeRef::clone(root);

    loop {
        let borrowed = node.borrow();
        let is_leaf = borrowed.children.is_empty();
        let has_children = !borrowed.children.is_empty();
        drop(borrowed);

        // Stop if we're at a leaf or there are no children
        if is_leaf || !has_children {
            let borrowed = node.borrow();
            return Ok(borrowed.content.clone());
        }

        // Pick the best child for this query
        node = pick_child(llm, query, &node).await?;
    }
}

/// Ask the LLM to pick the most relevant child node.
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

/// Retriever error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Invalid LLM response: {0}")]
    InvalidResponse(String),

    #[error("Node has no children")]
    NoChildren,
}
