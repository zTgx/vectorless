# vectorless

[![ crates.io ]](https://img.shields.io/crates/v/vectorless)](https://crates.io/crates/vectorless)
[![ documentation ]](https://img.shields.io/docsrs/vectorless)](https://docs.rs/vectorless)
[![ license ]](https://img.shields.io/crates/l/vectorless)](#license)

A lightweight document indexing engine without vectorization.

## Overview

`vectorless` provides efficient document indexing and search capabilities without relying on vector embeddings or external machine learning models. It uses traditional indexing techniques such as inverted indices, tokenization, and BM25 ranking for fast and accurate full-text search.

## Features

- **Zero-dependency core** - No vector database or ML runtime required
- **Fast indexing** - Optimized for quick document ingestion
- **BM25 ranking** - Industry-standard relevance scoring algorithm
- **Memory efficient** - Designed for low-memory environments
- **Flexible tokenization** - Pluggable tokenizers for different languages
