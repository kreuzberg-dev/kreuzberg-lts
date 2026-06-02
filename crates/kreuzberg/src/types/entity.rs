//! Named-entity recognition output types.
//!
//! Produced by the NER post-processor (`crates/kreuzberg/src/text/ner/`) and
//! attached to [`ExtractionResult::entities`](super::extraction::ExtractionResult::entities).
//! Backends (gline-rs ONNX, LLM-driven) share the [`NerBackend`](crate::text::ner::backend::NerBackend)
//! trait so the redaction post-processor can consume the same entity stream.

use serde::{Deserialize, Serialize};

/// A single named entity detected in the extracted text.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct Entity {
    /// Canonical category the entity belongs to (PERSON, ORG, LOCATION, etc.).
    pub category: EntityCategory,
    /// Raw mention text exactly as it appeared in the source.
    pub text: String,
    /// Byte-offset span in `ExtractionResult::content` where the mention starts.
    pub start: u32,
    /// Byte-offset span in `ExtractionResult::content` where the mention ends (exclusive).
    pub end: u32,
    /// Backend-reported confidence in `[0.0, 1.0]`. `None` when the backend does not
    /// expose confidence scores.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f32>,
}

/// Standard entity categories produced by built-in NER backends.
///
/// The `Custom(String)` variant lets caller-supplied categories (e.g. LLM
/// schemas) flow through without losing fidelity to the consumer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum EntityCategory {
    Person,
    Organization,
    Location,
    Date,
    Time,
    Money,
    Percent,
    Email,
    Phone,
    Url,
    Custom(String),
}
