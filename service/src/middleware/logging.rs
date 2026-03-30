// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Request logging middleware.

use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use tracing::{info, warn};

/// Request logging middleware.
///
/// Logs information about incoming requests including:
/// - HTTP method and path
/// - Response status code
/// - Request duration
pub async fn request_logging(request: Request, next: Next) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let path = uri.path().to_string();

    let response = next.run(request).await;

    let duration = start.elapsed();
    let status = response.status();
    let status_code = status.as_u16();

    // Log request with appropriate level based on status code
    let duration_ms = duration.as_millis();
    if status_code >= 500 {
        warn!(
            method = %method,
            path = %path,
            status = status_code,
            duration_ms = duration_ms,
            "Request failed with server error"
        );
    } else if status_code >= 400 {
        warn!(
            method = %method,
            path = %path,
            status = status_code,
            duration_ms = duration_ms,
            "Request failed with client error"
        );
    } else {
        info!(
            method = %method,
            path = %path,
            status = status_code,
            duration_ms = duration_ms,
            "Request completed"
        );
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_logging_exists() {
        // Just verify the function compiles
        assert!(true);
    }
}
