// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! # vectorless-llm
//!
//! LLM integration for vectorless - unified interface for chat and embedding models.
//!
//! ## Features
//!
//! - `chat` (default) - Chat completion models
//! - `embedding` - Text embedding models
//!
//! ## Example
//!
//! ```rust,no_run
//! use vectorless_llm::chat::{ChatModel, Message, Role, ChatOptions};
//!
//! # struct MyModel;
//! # #[async_trait::async_trait]
//! # impl vectorless_llm::chat::ChatModel for MyModel {
//! #     async fn chat(&self, _: &[Message], _: &ChatOptions) -> Result<vectorless_llm::chat::ChatCompletion, vectorless_llm::chat::Error> {
//! #         unimplemented!()
//! #     }
//! # }
//! async fn example() {
//!     let model = MyModel;
//!     let messages = vec![
//!         Message { role: Role::User, content: "Hello".into() }
//!     ];
//!     let response = model.chat(&messages, &ChatOptions::default()).await.unwrap();
//!     println!("{}", response.content);
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]

#[cfg(feature = "chat")]
pub mod chat;

#[cfg(feature = "embedding")]
pub mod embedding;

pub mod zai;

pub use zai::{ZaiClient, ZAI_API_BASE, ZAI_CODING_BASE};
