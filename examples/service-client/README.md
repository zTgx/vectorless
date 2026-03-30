# Service Client Example

Demonstrates using the vectorless HTTP API from a Rust client.

## Purpose

This example shows how to interact with the vectorless service using its REST API:
- Health checks
- Document management (create, list, get, delete)
- Content upload
- RAG queries

## Prerequisites

The vectorless service must be running:

```bash
# Option 1: Using Docker
docker compose -f docker/docker-compose.yml up -d

# Option 2: Using cargo
ZAI_API_KEY=your-key cargo run -p vectorless-service
```

## Usage

### 1. Set environment variables

```bash
# Service URL (default: http://localhost:8080)
export SERVICE_URL="http://localhost:8080"

# API key (if authentication is enabled)
export SERVICE_API_KEY="your-api-key"
```

### 2. Run the example

```bash
cargo run --example service-client
```

## API Endpoints

The example demonstrates the following endpoints:

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |
| GET | `/documents` | List all documents |
| POST | `/documents` | Create new document |
| GET | `/documents/:id` | Get document by ID |
| DELETE | `/documents/:id` | Delete document |
| POST | `/documents/:id/content` | Upload document content |
| POST | `/query` | Query the RAG system |

## Example Request/Response

### Create Document

**Request:**
```bash
curl -X POST http://localhost:8080/documents \
  -H "Content-Type: application/json" \
  -d '{"title": "My Document"}'
```

**Response:**
```json
{
  "id": "uuid-here",
  "status": "pending"
}
```

### Query

**Request:**
```bash
curl -X POST http://localhost:8080/query \
  -H "Content-Type: application/json" \
  -d '{"query": "What is this about?"}'
```

**Response:**
```json
{
  "answer": "This document is about...",
  "sources": [
    {
      "document_id": "uuid-here",
      "section": "Introduction",
      "content": "..."
    }
  ]
}
```

## Authentication

If the service is configured with API keys, include the key in your requests:

```bash
# Using X-API-Key header
curl -H "X-API-Key: your-key" http://localhost:8080/health

# Using Authorization header
curl -H "Authorization: Bearer your-key" http://localhost:8080/health
```

## Code Structure

```
src/
└── main.rs           # Example client implementation
```

The `VectorlessClient` struct provides methods for all API operations.

## Related Examples

- [local-rag](../local-rag/) - Direct library usage without HTTP
- [agent-rag](../agent-rag/) - Agent-based RAG system
