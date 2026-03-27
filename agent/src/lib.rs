// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # vectorless-agent
//!
//! Agent framework for vectorless - LLM-powered autonomous agents.
//!
//! ## Example
//!
//! ```rust,no_run
//! use vectorless_agent::{Agent, AgentConfig};
//!
//! # struct MyAgent;
//! # #[async_trait::async_trait]
//! # impl vectorless_agent::Agent for MyAgent {
//! #     async fn run(&self, task: &str) -> Result<Vec<vectorless_agent::AgentAction>, vectorless_agent::agent::Error> {
//! #         unimplemented!()
//! #     }
//! #     fn add_tool(&mut self, tool: Box<dyn vectorless_agent::tool::Tool>) {}
//! #     fn tools(&self) -> &std::collections::HashMap<String, Box<dyn vectorless_agent::tool::Tool>> {
//! #         unimplemented!()
//! #     }
//! # }
//! async fn example() {
//!     let agent = MyAgent;
//!     let actions = agent.run("Solve this problem").await.unwrap();
//!     for action in actions {
//!         println!("{:?}", action);
//!     }
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]

pub mod agent;
pub mod tool;

pub use agent::{Agent, AgentAction, AgentConfig, Error as AgentError};
pub use tool::{Tool, ToolDefinition, ToolInput, ToolOutput, Error as ToolError};
