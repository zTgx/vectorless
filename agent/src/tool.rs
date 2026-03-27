// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Tool interface for agents.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Input arguments for a tool.
pub type ToolInput = serde_json::Value;

/// Output result from a tool.
pub type ToolOutput = String;

/// Tool definition describing its capabilities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Async tool interface.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool definition.
    fn definition(&self) -> &ToolDefinition;

    /// Execute the tool with given input.
    async fn execute(&self, input: &ToolInput) -> Result<ToolOutput, Error>;
}

/// Tool error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Tool not found: {0}")]
    NotFound(String),
}
