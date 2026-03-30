// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! CORS (Cross-Origin Resource Sharing) middleware.

use tower_http::cors::{Any, CorsLayer};

/// CORS configuration.
#[derive(Debug, Clone)]
pub struct CorsConfig {
    /// Allowed origins (e.g., "http://localhost:3000").
    /// Use "*" to allow any origin.
    pub allowed_origins: Vec<String>,

    /// Allowed methods (e.g., "GET", "POST", "PUT", "DELETE").
    pub allowed_methods: Vec<String>,

    /// Allowed headers.
    pub allowed_headers: Vec<String>,

    /// Allow credentials (cookies, authorization headers).
    pub allow_credentials: bool,

    /// Max age for preflight requests (seconds).
    pub max_age: Option<u64>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            // By default, allow common origins for development
            allowed_origins: vec![
                "http://localhost:3000".to_string(),
                "http://localhost:8080".to_string(),
            ],
            // Common HTTP methods
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            // Common headers
            allowed_headers: vec![
                "content-type".to_string(),
                "authorization".to_string(),
                "x-api-key".to_string(),
            ],
            allow_credentials: false,
            max_age: Some(86400), // 24 hours
        }
    }
}

impl CorsConfig {
    /// Create a new CORS configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a permissive CORS configuration (allows any origin).
    ///
    /// # Warning
    /// This should not be used in production without careful consideration.
    pub fn permissive() -> Self {
        Self {
            allowed_origins: vec!["*".to_string()],
            allowed_methods: vec!["*".to_string()],
            allowed_headers: vec!["*".to_string()],
            allow_credentials: false,
            max_age: None,
        }
    }

    /// Add an allowed origin.
    pub fn add_origin(mut self, origin: impl Into<String>) -> Self {
        self.allowed_origins.push(origin.into());
        self
    }

    /// Add an allowed method.
    pub fn add_method(mut self, method: impl Into<String>) -> Self {
        self.allowed_methods.push(method.into());
        self
    }

    /// Add an allowed header.
    pub fn add_header(mut self, header: impl Into<String>) -> Self {
        self.allowed_headers.push(header.into());
        self
    }

    /// Set whether credentials are allowed.
    pub fn allow_credentials(mut self, allow: bool) -> Self {
        self.allow_credentials = allow;
        self
    }

    /// Set the max age for preflight requests.
    pub fn max_age(mut self, seconds: u64) -> Self {
        self.max_age = Some(seconds);
        self
    }

    /// Build the CORS layer from this configuration.
    pub fn build(&self) -> CorsLayer {
        let mut cors = CorsLayer::new();

        // Set allowed origins
        if self.allowed_origins.contains(&"*".to_string()) {
            cors = cors.allow_origin(Any);
        } else {
            let origins: Vec<_> = self.allowed_origins
                .iter()
                .map(|s| s.parse().unwrap())
                .collect();
            if !origins.is_empty() {
                cors = cors.allow_origin(origins);
            }
        }

        // Set allowed methods
        if self.allowed_methods.contains(&"*".to_string()) {
            cors = cors.allow_methods(Any);
        } else {
            let methods: Vec<_> = self.allowed_methods
                .iter()
                .map(|s| s.parse().unwrap())
                .collect();
            if !methods.is_empty() {
                cors = cors.allow_methods(methods);
            }
        }

        // Set allowed headers
        if self.allowed_headers.contains(&"*".to_string()) {
            cors = cors.allow_headers(Any);
        } else {
            let headers: Vec<_> = self.allowed_headers
                .iter()
                .map(|s| s.parse().unwrap())
                .collect();
            if !headers.is_empty() {
                cors = cors.allow_headers(headers);
            }
        }

        // Set credentials
        if self.allow_credentials {
            cors = cors.allow_credentials(true);
        }

        // Set max age
        if let Some(max_age) = self.max_age {
            cors = cors.max_age(std::time::Duration::from_secs(max_age));
        }

        cors
    }
}

/// Create a CORS layer with default configuration.
pub fn cors_layer() -> CorsLayer {
    CorsConfig::default().build()
}

/// Create a CORS layer with custom configuration.
pub fn cors_layer_with_config(config: &CorsConfig) -> CorsLayer {
    config.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_config_default() {
        let config = CorsConfig::default();
        assert!(!config.allowed_origins.is_empty());
        assert!(!config.allowed_methods.is_empty());
        assert!(!config.allowed_headers.is_empty());
        assert!(!config.allow_credentials);
    }

    #[test]
    fn test_cors_config_permissive() {
        let config = CorsConfig::permissive();
        assert_eq!(config.allowed_origins, vec!["*"]);
        assert_eq!(config.allowed_methods, vec!["*"]);
        assert_eq!(config.allowed_headers, vec!["*"]);
    }

    #[test]
    fn test_cors_config_builder() {
        let config = CorsConfig::new()
            .add_origin("http://example.com")
            .add_method("PATCH")
            .add_header("x-custom-header")
            .allow_credentials(true)
            .max_age(3600);

        assert!(config.allowed_origins.contains(&"http://example.com".to_string()));
        assert!(config.allowed_methods.contains(&"PATCH".to_string()));
        assert!(config.allowed_headers.contains(&"x-custom-header".to_string()));
        assert!(config.allow_credentials);
        assert_eq!(config.max_age, Some(3600));
    }
}
