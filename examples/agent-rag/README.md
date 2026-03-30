# Agent RAG Example

Demonstrates using vectorless with AI agents for enhanced RAG capabilities.

## Purpose

This example shows how to build an AI agent that uses vectorless for document retrieval, enabling:
- Tool-based document querying
- Multi-step reasoning
- Context-aware responses
- Chained retrieval operations

## Features

- **Document Retrieval Tool**: Agent can query indexed documents
- **Multi-step Workflow**: Chain multiple queries together
- **Context Management**: Maintain conversation state
- **LLM Integration**: Use LLM for reasoning and synthesis

## Usage

### 1. Set up environment

```bash
export ZAI_API_KEY="your-api-key"
```

### 2. Index some documents first

```bash
# Run the local-rag example to create indexed documents
ZAI_API_KEY=your-key cargo run --example local-rag
```

### 3. Run the agent example

```bash
ZAI_API_KEY=your-key cargo run --example agent-rag
```

## Architecture

```
┌─────────────┐
│   Agent     │
│             │
│  ┌───────┐  │       ┌──────────────────┐
│  │ Tools │──┼──────>│ DocumentCollection│
│  └───────┘  │       │  (Indexed Docs)  │
└─────────────┘       └──────────────────┘
       │
       ▼
┌─────────────┐
│  LLM Client │
│             │
└─────────────┘
```

## Implementation Status

**🚧 Work in Progress**

This example demonstrates the agent pattern but the full agent implementation is evolving:
- ✅ Basic document retrieval
- ✅ LLM integration
- ✅ Multi-step workflow
- 🚧 Full agent tool system (in vectorless-agent crate)
- 🚧 Conversation memory
- 🚧 Advanced reasoning chains

## Related Examples

- [local-rag](../local-rag/) - Basic document indexing and retrieval
- [service-client](../service-client/) - HTTP API client example
