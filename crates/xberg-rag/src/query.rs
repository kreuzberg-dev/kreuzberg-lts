//! Retrieval query types.

use crate::error::{RagError, RagResult};
use crate::filter::Filter;
use crate::types::{CollectionSpec, MultiVector, RetrievedChunk, SparseVector};
use serde::{Deserialize, Serialize};

/// Maximum number of results a single query may request.
pub const MAX_TOP_K: u32 = 200;
/// Maximum candidate-pool multiplier.
pub const MAX_CANDIDATE_MULTIPLIER: u32 = 20;

/// Retrieval mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RetrieveMode {
    /// Vector similarity search only (default).
    #[default]
    Vector,
    /// Full-text search only.
    FullText,
    /// Hybrid (vector + full-text [+ sparse] fused).
    Hybrid,
    /// Sparse (SPLADE-style) retrieval only.
    Sparse,
    /// Late-interaction (ColBERT-style MaxSim) reranking of a candidate set.
    ///
    /// Backends implement this differently despite advertising the same
    /// [`Capabilities::late_interaction`](crate::Capabilities::late_interaction)
    /// flag: the in-memory backend runs an exhaustive MaxSim scan over every
    /// stored `multi_vector` and ignores `query_vector`/`query_text` entirely
    /// (true global ranking, no `query_vector` required). The sqlite backend
    /// seeds a candidate set via dense KNN over `query_vector` and reranks only
    /// those candidates with MaxSim (recall bounded by `candidate_k`), so it
    /// requires `query_vector` and errors without one even though `query_text`
    /// alone satisfies [`RetrieveQuery::validate`]. This is an intentional
    /// two-stage tradeoff (as in ColBERT PLAID), not a bug — check the backend
    /// docs before assuming exhaustive recall.
    LateInteraction,
}

impl RetrieveMode {
    /// Lowercase wire token, used in error messages.
    pub fn as_str(self) -> &'static str {
        match self {
            RetrieveMode::Vector => "vector",
            RetrieveMode::FullText => "full_text",
            RetrieveMode::Hybrid => "hybrid",
            RetrieveMode::Sparse => "sparse",
            RetrieveMode::LateInteraction => "late_interaction",
        }
    }
}

/// A retrieval query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveQuery {
    /// Retrieval mode.
    #[serde(default)]
    pub mode: RetrieveMode,
    /// Text query (required for full-text/hybrid; optional for vector when a
    /// vector is supplied).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_text: Option<String>,
    /// Pre-computed query vector.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_vector: Option<Vec<f32>>,
    /// Pre-computed sparse (SPLADE-style) query vector.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_sparse: Option<SparseVector>,
    /// Pre-computed late-interaction (ColBERT-style) query multi-vector.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query_multi_vector: Option<MultiVector>,
    /// Number of results to return.
    pub top_k: u32,
    /// Optional filter constraint.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Filter>,
    /// Candidate-pool multiplier (pull `top_k * multiplier` before rerank/fusion).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub candidate_multiplier: Option<u32>,
    /// Collapse to the single best chunk per document.
    #[serde(default)]
    pub group_by_document: bool,
    /// Include chunk content in results.
    #[serde(default)]
    pub include_content: bool,
    /// Include the parent document summary in results.
    #[serde(default)]
    pub include_document: bool,
}

impl RetrieveQuery {
    /// A minimal vector query for `top_k` results.
    pub fn vector(top_k: u32) -> Self {
        Self {
            mode: RetrieveMode::Vector,
            query_text: None,
            query_vector: None,
            query_sparse: None,
            query_multi_vector: None,
            top_k,
            filter: None,
            candidate_multiplier: None,
            group_by_document: false,
            include_content: true,
            include_document: false,
        }
    }

    /// Validate the query against the target collection.
    ///
    /// Checks `top_k`/`candidate_multiplier` ranges, query-vector dimension,
    /// mode/input consistency, and the filter (fields + complexity). Capability
    /// gating (whether the backend supports the mode) is enforced by the store,
    /// which returns [`RagError::UnsupportedMode`].
    ///
    /// # Errors
    ///
    /// [`RagError::InvalidQuery`], [`RagError::EmbeddingDimMismatch`], or any
    /// filter error.
    pub fn validate(&self, collection: &CollectionSpec) -> RagResult<()> {
        if self.top_k < 1 || self.top_k > MAX_TOP_K {
            return Err(RagError::InvalidQuery(format!(
                "top_k must be between 1 and {MAX_TOP_K}"
            )));
        }
        if let Some(mult) = self.candidate_multiplier
            && !(1..=MAX_CANDIDATE_MULTIPLIER).contains(&mult)
        {
            return Err(RagError::InvalidQuery(format!(
                "candidate_multiplier must be between 1 and {MAX_CANDIDATE_MULTIPLIER}"
            )));
        }
        if let Some(vec) = &self.query_vector
            && vec.len() as u32 != collection.embedding_dim
        {
            return Err(RagError::EmbeddingDimMismatch {
                expected: collection.embedding_dim,
                got: vec.len() as u32,
            });
        }
        match self.mode {
            RetrieveMode::Vector => {
                if self.query_text.is_none() && self.query_vector.is_none() {
                    return Err(RagError::InvalidQuery(
                        "vector mode requires query_text or query_vector".to_string(),
                    ));
                }
            }
            RetrieveMode::FullText => {
                if self.query_text.is_none() {
                    return Err(RagError::InvalidQuery("full_text mode requires query_text".to_string()));
                }
                if self.query_vector.is_some() {
                    return Err(RagError::InvalidQuery(
                        "full_text mode does not accept query_vector".to_string(),
                    ));
                }
            }
            RetrieveMode::Hybrid => {
                if self.query_text.is_none() {
                    return Err(RagError::InvalidQuery("hybrid mode requires query_text".to_string()));
                }
            }
            RetrieveMode::Sparse => {
                let Some(sparse) = &self.query_sparse else {
                    return Err(RagError::InvalidQuery("sparse mode requires query_sparse".to_string()));
                };
                if !sparse.is_well_formed() {
                    return Err(RagError::InvalidQuery(
                        "query_sparse is malformed: indices and values must be equal length and indices strictly ascending"
                            .to_string(),
                    ));
                }
            }
            RetrieveMode::LateInteraction => {
                let Some(multi_vector) = &self.query_multi_vector else {
                    return Err(RagError::InvalidQuery(
                        "late_interaction mode requires query_multi_vector".to_string(),
                    ));
                };
                if !multi_vector.is_well_formed() {
                    return Err(RagError::InvalidQuery(
                        "query_multi_vector is malformed: data length must equal num_tokens * dim".to_string(),
                    ));
                }
                if self.query_text.is_none() && self.query_vector.is_none() {
                    return Err(RagError::InvalidQuery(
                        "late_interaction mode requires query_text or query_vector to seed candidates".to_string(),
                    ));
                }
            }
        }
        if let Some(filter) = &self.filter {
            filter.validate()?;
        }
        Ok(())
    }
}

/// The output of a retrieval query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveOutput {
    /// Mode that was executed.
    pub mode: RetrieveMode,
    /// Retrieved chunks, in descending relevance order.
    pub chunks: Vec<RetrievedChunk>,
    /// Primary-retrieval latency in milliseconds (before any rerank).
    #[serde(default)]
    pub primary_latency_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DistanceMetric;

    fn collection(dim: u32) -> CollectionSpec {
        CollectionSpec {
            name: "c".to_string(),
            embedding_dim: dim,
            distance_metric: DistanceMetric::Cosine,
            index_method: crate::types::IndexMethod::Flat,
        }
    }

    #[test]
    fn valid_vector_query() {
        let q = RetrieveQuery {
            query_text: Some("hi".to_string()),
            ..RetrieveQuery::vector(10)
        };
        assert!(q.validate(&collection(768)).is_ok());
    }

    #[test]
    fn rejects_zero_top_k() {
        assert!(RetrieveQuery::vector(0).validate(&collection(768)).is_err());
    }

    #[test]
    fn rejects_top_k_over_cap() {
        assert!(RetrieveQuery::vector(201).validate(&collection(768)).is_err());
    }

    #[test]
    fn rejects_vector_dim_mismatch() {
        let q = RetrieveQuery {
            query_vector: Some(vec![0.0; 512]),
            ..RetrieveQuery::vector(10)
        };
        assert!(matches!(
            q.validate(&collection(768)),
            Err(RagError::EmbeddingDimMismatch {
                expected: 768,
                got: 512
            })
        ));
    }

    #[test]
    fn rejects_vector_mode_without_inputs() {
        assert!(RetrieveQuery::vector(10).validate(&collection(768)).is_err());
    }

    #[test]
    fn rejects_fulltext_with_vector() {
        let q = RetrieveQuery {
            mode: RetrieveMode::FullText,
            query_text: Some("q".to_string()),
            query_vector: Some(vec![0.0; 768]),
            ..RetrieveQuery::vector(10)
        };
        assert!(q.validate(&collection(768)).is_err());
    }

    #[test]
    fn rejects_sparse_mode_without_query_sparse() {
        let q = RetrieveQuery {
            mode: RetrieveMode::Sparse,
            ..RetrieveQuery::vector(10)
        };
        assert!(matches!(q.validate(&collection(768)), Err(RagError::InvalidQuery(_))));
    }

    #[test]
    fn accepts_sparse_mode_with_query_sparse() {
        let q = RetrieveQuery {
            mode: RetrieveMode::Sparse,
            query_sparse: Some(crate::types::SparseVector {
                indices: vec![1, 2],
                values: vec![0.5, 0.5],
            }),
            ..RetrieveQuery::vector(10)
        };
        assert!(q.validate(&collection(768)).is_ok());
    }

    #[test]
    fn rejects_late_interaction_without_multi_vector() {
        let q = RetrieveQuery {
            mode: RetrieveMode::LateInteraction,
            query_text: Some("hi".to_string()),
            ..RetrieveQuery::vector(10)
        };
        assert!(matches!(q.validate(&collection(768)), Err(RagError::InvalidQuery(_))));
    }

    #[test]
    fn rejects_late_interaction_without_seed_input() {
        let q = RetrieveQuery {
            mode: RetrieveMode::LateInteraction,
            query_multi_vector: Some(crate::types::MultiVector {
                num_tokens: 1,
                dim: 2,
                data: vec![0.1, 0.2],
            }),
            ..RetrieveQuery::vector(10)
        };
        assert!(matches!(q.validate(&collection(768)), Err(RagError::InvalidQuery(_))));
    }

    #[test]
    fn accepts_late_interaction_with_multi_vector_and_seed() {
        let q = RetrieveQuery {
            mode: RetrieveMode::LateInteraction,
            query_text: Some("hi".to_string()),
            query_multi_vector: Some(crate::types::MultiVector {
                num_tokens: 1,
                dim: 2,
                data: vec![0.1, 0.2],
            }),
            ..RetrieveQuery::vector(10)
        };
        assert!(q.validate(&collection(768)).is_ok());
    }
}
