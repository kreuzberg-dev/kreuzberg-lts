//! Configuration types for text chunking.

use serde::{Deserialize, Serialize};

pub use crate::core::config::processing::{ChunkSizing, ChunkerType, ChunkingConfig};

/// Result of a text chunking operation.
///
/// Contains the generated chunks and metadata about the chunking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingResult {
    /// List of text chunks
    pub chunks: Vec<crate::types::Chunk>,
    /// Total number of chunks generated
    pub chunk_count: usize,
}
