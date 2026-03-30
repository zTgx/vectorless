// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! HTTP middleware for authentication, logging, and CORS.

pub mod auth;
pub mod logging;
pub mod cors;

pub use auth::{require_api_key, ApiKeyAuth};
pub use logging::request_logging;
pub use cors::{cors_layer, CorsConfig};
