// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Agent interface.

use crate::tool::Tool;
use async_trait::async_trait;
use std::collections::HashMap;

/// Agent configuration.
#[derive(Debug, Clone)]
pub struct AgentConfig {
    pub max_iterations: usize,
    pub verbose: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self { max_iterations: 10, verbose: false }
    }
}

/// Agent action result.
#[derive(Debug, Clone)]
pub enum AgentAction {
    /// Agent made an observation.
    Thought(String),
    /// Agent used a tool.
    ToolUse { tool: String, input: String, output: String },
    /// Agent produced final answer.
    Answer(String),
}

/// Async agent interface.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Run the agent with a task.
    async fn run(&self, task: &str) -> Result<Vec<AgentAction>, Error>;

    /// Add a tool to the agent.
    fn add_tool(&mut self, tool: Box<dyn Tool>);

    /// Get available tools.
    fn tools(&self) -> &HashMap<String, Box<dyn Tool>>;
}

/// Agent error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Max iterations reached")]
    MaxIterationsReached,

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}
