// Copyright (c) 2026 vectorless developers
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Repository layer for data persistence.

pub mod metadata;
pub mod index;

pub use metadata::MetadataRepository;
pub use index::IndexRepository;
