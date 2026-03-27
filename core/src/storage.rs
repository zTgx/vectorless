// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Storage: save and load the index tree.

use crate::node::{PageNode, PageNodeRef};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

/// Serializable representation of a node.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeDto {
    title: String,
    content: String,
    summary: String,
    depth: usize,
    children: Vec<NodeDto>,
}

impl From<&PageNode> for NodeDto {
    fn from(node: &PageNode) -> Self {
        Self {
            title: node.title.clone(),
            content: node.content.clone(),
            summary: node.summary.clone(),
            depth: node.depth,
            children: node.children.iter().map(|c| {
                let borrowed = c.borrow();
                NodeDto::from(&*borrowed)
            }).collect(),
        }
    }
}

impl NodeDto {
    /// Convert DTO to PageNodeRef.
    fn to_node(self) -> PageNodeRef {
        let node = PageNode::new(&self.title, &self.content);
        node.borrow_mut().summary = self.summary;
        node.borrow_mut().depth = self.depth;

        for child_dto in self.children {
            let child = child_dto.to_node();
            child.borrow_mut().parent = Some(PageNodeRef::clone(&node));
            node.borrow_mut().children.push(PageNodeRef::clone(&child));
        }

        node
    }
}

/// Save the index tree to a JSON file.
pub fn save<P>(root: &PageNodeRef, path: P) -> Result<(), Error>
where
    P: AsRef<Path>,
{
    let borrowed = root.borrow();
    let dto = NodeDto::from(&*borrowed);
    drop(borrowed);

    let json = serde_json::to_string_pretty(&dto)
        .map_err(|e| Error::SerializationFailed(e.to_string()))?;

    fs::write(path, json).map_err(|e| Error::Io(e))?;
    Ok(())
}

/// Load the index tree from a JSON file.
pub fn load<P>(path: P) -> Result<PageNodeRef, Error>
where
    P: AsRef<Path>,
{
    let json = fs::read_to_string(path).map_err(|e| Error::Io(e))?;
    let dto: NodeDto = serde_json::from_str(&json)
        .map_err(|e| Error::DeserializationFailed(e.to_string()))?;

    Ok(dto.to_node())
}

/// Storage error types.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),
}
