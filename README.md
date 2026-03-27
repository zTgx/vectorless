# vectorless

> RAG without vector embeddings — tree-based retrieval powered by LLM navigation

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)

vectorless is a Rust-based RAG (Retrieval-Augmented Generation) system that uses **tree-based indexing** and **LLM navigation** instead of traditional vector embeddings. No vector database required.

## 🌟 Features

- **Tree-Based Indexing** — Documents are organized as hierarchical trees with summaries at each node
- **LLM Navigation** — Intelligent traversal using LLM to find relevant content
- **No Vector Database** — Eliminates infrastructure complexity and costs
- **Built in Rust** — Blazing fast performance with memory safety
- **HTTP API** — Simple RESTful API for easy integration
- **Multiple LLM Support** — Pluggable LLM providers (ZAI, OpenAI, etc.)

## 🚀 Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/zTgx/vectorless
cd vectorless

# Build the project
cargo build --release
```

### Run the RAG Service

```bash
# Set your LLM API credentials
export ZAI_API_KEY="your-api-key"
export ZAI_ENDPOINT="https://api.z.ai/api/coding/paas/v4"

# Start the HTTP server
cargo run -p vectorless-rag
```

The server will start on `http://localhost:8080`

### Basic Usage

```rust
use vectorless_core::*;
use vectorless_llm::zai::ZaiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize LLM client
    let llm = ZaiClient::new("your-api-key");

    // Configure indexer
    let config = IndexerConfig::builder()
        .subsection_threshold(200)
        .max_segment_tokens(4000)
        .summary_model("glm-5")
        .max_summary_tokens(200)
        .build();

    // Parse document
    let root = parse_document_with_config(&llm, document_text, &config).await?;

    // Build summaries
    build_summaries_with_config(&llm, &root, &config).await?;

    // Save index
    save(&root, "index.json")?;

    // Query
    let answer = retrieve(&llm, "What is the main topic?", &root).await?;
    println!("Answer: {}", answer);

    Ok(())
}
```

## 📚 Architecture

```
vectorless/
├── core/      # Core indexing and retrieval logic
├── llm/       # LLM abstraction layer
├── rag/       # HTTP RAG service
├── agent/     # Agent framework
├── cli/       # Command-line interface
└── sdk/       # Client SDK
```

### How It Works

1. **Parse** — Documents are segmented into sections based on structure
2. **Index** — A hierarchical tree is built with summaries for each node
3. **Retrieve** — The LLM navigates the tree to find relevant content
4. **Generate** — Results are used for RAG generation

## 🔌 HTTP API

### Documents

```bash
# Create a document
POST /documents
{"title": "My Document"}

# Upload content
POST /documents/{id}/content
{"content": "Document content here..."}

# List documents
GET /documents

# Get document
GET /documents/{id}

# Delete document
DELETE /documents/{id}
```

### Query

```bash
# Query the knowledge base
POST /query
{
  "query": "What is the main point?",
  "max_results": 3
}
```

### Health

```bash
# Check service health
GET /health
```

## ⚙️ Configuration

Configuration is done via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `ZAI_API_KEY` | LLM API key | - |
| `ZAI_ENDPOINT` | LLM endpoint | `https://api.z.ai/api/paas/v4` |
| `ZAI_MODEL` | Model name | `glm-5` |
| `RAG_HOST` | Server host | `0.0.0.0` |
| `RAG_PORT` | Server port | `8080` |
| `RAG_DATA_DIR` | Data directory | `./data` |
| `RAG_INDEX_DIR` | Index directory | `./indices` |
| `RAG_SUBSECTION_THRESHOLD` | Subsection token threshold | `200` |
| `RAG_MAX_SEGMENT_TOKENS` | Max segment tokens | `4000` |
| `RAG_MAX_SUMMARY_TOKENS` | Max summary tokens | `200` |

## 🆚 Comparison

| Vector RAG | Vectorless | Keyword Search |
|------------|------------|----------------|
| Requires embedding model | ✅ No embeddings | No semantic understanding |
| Vector database costs | ✅ Zero extra costs | Keyword matching only |
| Approximate results | ✅ Precise retrieval | Limited relevance |
| Complex infrastructure | ✅ Simple deployment | No context awareness |

## 📖 Example

```bash
cargo run --package basic --bin basic
```

Output:
```
> Hello! How can I help you today?

Building index...
Parsing document...
Building summaries...
Saving index to index.json
Index built successfully!

Query: What is this document about?
Answer: This document is an introductory book about the Rust programming language.
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

at your option.
