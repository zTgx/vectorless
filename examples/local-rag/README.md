# Local RAG Example

A demonstration of using vectorless for local document indexing and retrieval.

## Purpose

This example shows how to build a local RAG (Retrieval-Augmented Generation) system using the vectorless DocumentCollection API. It demonstrates:

- Creating a document collection with workspace persistence
- Indexing PDF and Markdown documents
- Querying documents with various retrieval modes
- Managing document metadata

## Structure

```
local-rag/
├── Cargo.toml       # Project dependencies
├── README.md        # This file
└── src/
    └── main.rs      # Entry point (structure only, implementation pending)
```

## Usage

1. Set the ZAI API key:
   ```bash
   export ZAI_API_KEY="your-api-key"
   ```

2. Add sample documents:
   ```bash
   mkdir -p documents
   # Add your PDF and Markdown files here
   ```

3. Run the example:
   ```bash
   cargo run --package local-rag
   ```

## Implementation Status

**🚧 Work in Progress - Structure Only**

This example is currently a placeholder. The implementation will include:

- [ ] Document ingestion workflow
- [ ] Query and retrieval examples
- [ ] Document management operations
- [ ] Batch indexing support
- [ ] Configuration options

## Related Examples

- [basic](../basic/) - Basic vectorless usage examples
