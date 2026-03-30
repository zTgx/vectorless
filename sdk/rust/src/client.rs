// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! HTTP client for the vectorless service.

use crate::{error::{Error, Result}, types::*};
use reqwest::{header, Client as HttpClient, StatusCode};
use uuid::Uuid;

/// Client configuration builder.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Base URL of the vectorless service
    pub base_url: String,

    /// API key for authentication (optional)
    pub api_key: Option<String>,

    /// Request timeout in seconds
    pub timeout_secs: u64,

    /// User agent header
    pub user_agent: String,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            api_key: None,
            timeout_secs: 30,
            user_agent: format!("vectorless-sdk-rs/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

impl ClientConfig {
    /// Create a new configuration builder.
    pub fn builder() -> ClientConfigBuilder {
        ClientConfigBuilder::new()
    }

    /// Convert to request client.
    fn to_http_client(&self) -> Result<HttpClient> {
        HttpClient::builder()
            .timeout(std::time::Duration::from_secs(self.timeout_secs))
            .user_agent(&self.user_agent)
            .build()
            .map_err(Error::from)
    }
}

/// Builder for `ClientConfig`.
#[derive(Debug, Clone, Default)]
pub struct ClientConfigBuilder {
    config: ClientConfig,
}

impl ClientConfigBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            config: ClientConfig::default(),
        }
    }

    /// Set the base URL.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = url.into();
        self
    }

    /// Set the API key.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.config.api_key = Some(key.into());
        self
    }

    /// Set the request timeout.
    pub fn timeout(mut self, secs: u64) -> Self {
        self.config.timeout_secs = secs;
        self
    }

    /// Build the configuration.
    pub fn build(self) -> ClientConfig {
        self.config
    }
}

/// HTTP client for the vectorless service.
///
/// # Example
///
/// ```no_run
/// use vectorless_sdk_rs::{Client, ClientConfig};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = ClientConfig::builder()
///     .base_url("http://localhost:8080")
///     .build();
///
/// let client = Client::new(config)?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    /// HTTP client
    http: HttpClient,

    /// Base URL
    base_url: String,

    /// API key for authentication
    api_key: Option<String>,
}

impl Client {
    /// Create a new client with the given configuration.
    pub fn new(config: ClientConfig) -> Result<Self> {
        let http = config.to_http_client()?;
        Ok(Self {
            http,
            base_url: config.base_url,
            api_key: config.api_key,
        })
    }

    /// Create a client with default configuration.
    ///
    /// Uses `http://localhost:8080` as the base URL and no authentication.
    pub fn default() -> Result<Self> {
        Self::new(ClientConfig::default())
    }

    /// Get request headers including authentication.
    fn get_headers(&self) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        if let Some(api_key) = &self.api_key {
            if let Ok(value) = header::HeaderValue::from_str(api_key) {
                headers.insert("x-api-key", value);
            }
        }

        headers
    }

    /// Check service health.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use vectorless_sdk_rs::Client;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # let client = Client::default()?;
    /// let health = client.health().await?;
    /// println!("Service status: {}", health.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn health(&self) -> Result<HealthResponse> {
        let response = self
            .http
            .get(format!("{}/health", self.base_url))
            .headers(self.get_headers())
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    /// List all documents.
    pub async fn list_documents(&self) -> Result<Vec<Document>> {
        let response = self
            .http
            .get(format!("{}/documents", self.base_url))
            .headers(self.get_headers())
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    /// Create a new document.
    pub async fn create_document(&self, title: &str) -> Result<CreateDocumentResponse> {
        let request = CreateDocumentRequest {
            title: title.to_string(),
        };

        let response = self
            .http
            .post(format!("{}/documents", self.base_url))
            .headers(self.get_headers())
            .json(&request)
            .send()
            .await?;

        if response.status() == StatusCode::UNAUTHORIZED {
            return Err(Error::AuthenticationFailed);
        }

        let response = response.error_for_status()?;

        Ok(response.json().await?)
    }

    /// Get document by ID.
    pub async fn get_document(&self, id: Uuid) -> Result<Document> {
        let response = self
            .http
            .get(format!("{}/documents/{}", self.base_url, id))
            .headers(self.get_headers())
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    /// Delete a document.
    pub async fn delete_document(&self, id: Uuid) -> Result<DeleteDocumentResponse> {
        let response = self
            .http
            .delete(format!("{}/documents/{}", self.base_url, id))
            .headers(self.get_headers())
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    /// Upload document content.
    pub async fn upload_content(&self, id: Uuid, content: &str) -> Result<UploadContentResponse> {
        let request = UploadContentRequest {
            content: content.to_string(),
        };

        let response = self
            .http
            .post(format!("{}/documents/{}/content", self.base_url, id))
            .headers(self.get_headers())
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }

    /// Query the RAG system.
    pub async fn query(&self, query: &str) -> Result<QueryResponse> {
        let request = QueryRequest {
            query: query.to_string(),
        };

        let response = self
            .http
            .post(format!("{}/query", self.base_url))
            .headers(self.get_headers())
            .json(&request)
            .send()
            .await?
            .error_for_status()?;

        Ok(response.json().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_builder() {
        let config = ClientConfig::builder()
            .base_url("http://example.com")
            .api_key("test-key")
            .timeout(60)
            .build();

        assert_eq!(config.base_url, "http://example.com");
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.timeout_secs, 60);
    }

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.base_url, "http://localhost:8080");
        assert!(config.api_key.is_none());
    }
}
