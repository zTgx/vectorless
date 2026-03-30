// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! API key authentication middleware.

use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

/// API key authentication state.
#[derive(Clone)]
pub struct ApiKeyAuth {
    /// Valid API keys (empty means no authentication required).
    pub valid_keys: Arc<Vec<String>>,
}

impl ApiKeyAuth {
    /// Create a new API key authenticator.
    ///
    /// If `valid_keys` is empty, authentication is disabled (all requests allowed).
    pub fn new(valid_keys: Vec<String>) -> Self {
        Self {
            valid_keys: Arc::new(valid_keys),
        }
    }

    /// Check if authentication is enabled.
    pub fn is_enabled(&self) -> bool {
        !self.valid_keys.is_empty()
    }

    /// Validate an API key.
    pub fn validate_key(&self, key: &str) -> bool {
        if !self.is_enabled() {
            return true;
        }
        self.valid_keys.iter().any(|k| k == key)
    }
}

impl Default for ApiKeyAuth {
    fn default() -> Self {
        Self::new(vec![])
    }
}

/// Extract API key from request headers.
fn extract_api_key(headers: &HeaderMap) -> Option<String> {
    // Try X-API-Key header first
    if let Some(key) = headers.get("x-api-key") {
        return key.to_str().ok().map(|s| s.to_string());
    }

    // Try Authorization header with Bearer token
    if let Some(auth) = headers.get("authorization") {
        if let Ok(auth_str) = auth.to_str() {
            if auth_str.starts_with("Bearer ") {
                return Some(auth_str[7..].to_string());
            }
        }
    }

    None
}

/// Middleware to require API key authentication.
///
/// This middleware checks for an API key in the request headers.
/// Valid sources (in order of priority):
/// - `X-API-Key` header
/// - `Authorization: Bearer <key>` header
///
/// Returns 401 Unauthorized if no valid key is provided.
pub async fn require_api_key(
    auth: axum::extract::State<ApiKeyAuth>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip authentication if not enabled
    if !auth.is_enabled() {
        return Ok(next.run(request).await);
    }

    // Extract API key from headers
    let key = extract_api_key(request.headers())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate key
    if auth.validate_key(&key) {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_api_key_from_x_api_key() {
        let mut headers = HeaderMap::new();
        headers.insert("x-api-key", "test-key-123".parse().unwrap());

        let key = extract_api_key(&headers);
        assert_eq!(key, Some("test-key-123".to_string()));
    }

    #[test]
    fn test_extract_api_key_from_authorization() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", "Bearer test-key-456".parse().unwrap());

        let key = extract_api_key(&headers);
        assert_eq!(key, Some("test-key-456".to_string()));
    }

    #[test]
    fn test_extract_api_key_none() {
        let headers = HeaderMap::new();
        let key = extract_api_key(&headers);
        assert_eq!(key, None);
    }

    #[test]
    fn test_api_key_auth_disabled() {
        let auth = ApiKeyAuth::default();
        assert!(!auth.is_enabled());
        assert!(auth.validate_key(""));
    }

    #[test]
    fn test_api_key_auth_enabled() {
        let auth = ApiKeyAuth::new(vec!["key1".to_string(), "key2".to_string()]);
        assert!(auth.is_enabled());
        assert!(auth.validate_key("key1"));
        assert!(auth.validate_key("key2"));
        assert!(!auth.validate_key("invalid"));
    }
}
