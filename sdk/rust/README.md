# Vectorless Rust SDK

Rust SDK for the vectorless service - HTTP client for document indexing and RAG queries.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
vectorless-sdk-rs = "0.1.0"
```

Or use:

```bash
cargo add vectorless-sdk-rs
```

## Quick Start

```rust
use vectorless_sdk_rs::{Client, ClientConfig};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client
    let config = ClientConfig::builder()
        .base_url("http://localhost:8080")
        .api_key("your-api-key")
        .build();

    let client = Client::new(config)?;

    // Check service health
    let health = client.health().await?;
    println!("Service status: {}", health.status);

    // Create a document
    let doc = client.create_document("My Document").await?;
    println!("Created document: {}", doc.id);

    // Query the RAG system
    let response = client.query("What is this about?").await?;
    println!("Answer: {}", response.answer);

    Ok(())
}
```

## Features

- Type-safe Rust API
- Async/await support with tokio
- Builder pattern for configuration
- Comprehensive error handling
- Full API coverage

## API Methods

| Method | Description |
|--------|-------------|
| `health()` | Check service health |
| `list_documents()` | List all documents |
| `create_document(title)` | Create new document |
| `get_document(id)` | Get document by ID |
| `delete_document(id)` | Delete document |
| `upload_content(id, content)` | Upload document content |
| `query(query)` | Query RAG system |

## Configuration

```rust
let config = ClientConfig::builder()
    .base_url("http://localhost:8080")
    .api_key("your-api-key")
    .timeout(60)
    .build();
```

## License

MIT OR Apache-2.0
