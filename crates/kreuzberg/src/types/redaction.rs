//! Redaction & anonymisation output types.
//!
//! Produced by the redaction post-processor
//! (`crates/kreuzberg/src/text/redaction/`) and attached to
//! [`ExtractionResult::redaction_report`](super::extraction::ExtractionResult::redaction_report).
//!
//! Redaction is a **Late-stage** post-processor: it always runs after NER,
//! summarisation, translation, page classification, and captioning have populated
//! their own fields. The processor rewrites `result.content`, `result.formatted_content`,
//! every `result.chunks[i].content`, and the textual fields of `result.entities`,
//! `summary`, `translation`, `page_classifications`. The original text never appears in
//! the returned `ExtractionResult` — this struct is the audit trail of what was found.

use serde::{Deserialize, Serialize};

/// Audit report describing what the redaction processor found and how it replaced it.
///
/// The redactor returns this alongside the rewritten content so compliance, replay, and
/// audit-log consumers can see exactly what fired. Offsets are relative to the *original*
/// pre-redaction `content` and are intended for audit reconstruction only — the original
/// bytes are dropped at the end of the pipeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct RedactionReport {
    /// Individual redaction findings in original-source byte order.
    pub findings: Vec<RedactionFinding>,
    /// Total number of redactions applied across the document.
    pub total_redacted: u32,
}

/// One redaction event: which span was rewritten, why, and with what.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
pub struct RedactionFinding {
    /// Byte-offset start in the original (pre-redaction) `ExtractionResult::content`.
    pub start: u32,
    /// Byte-offset end (exclusive) in the original `ExtractionResult::content`.
    pub end: u32,
    /// PII category that fired this redaction.
    pub category: PiiCategory,
    /// Strategy applied to this finding (mask, hash, token-replace, drop).
    pub strategy: RedactionStrategy,
    /// String that replaced the original mention. Always present; for `Drop` the
    /// replacement is the empty string.
    pub replacement_token: String,
}

/// Strategy applied when a PII match is rewritten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum RedactionStrategy {
    /// Replace the matched span with a fixed mask token (default `"[REDACTED]"`).
    Mask,
    /// Replace with a SHA-256 hash of the original value (truncated to 16 hex chars).
    /// Lets downstream consumers do equality joins without recovering the source.
    Hash,
    /// Replace with a per-category running token (`"[PERSON_1]"`, `"[PERSON_2]"`, …)
    /// so the same person referenced twice gets the same token within the document.
    TokenReplace,
    /// Delete the matched span entirely.
    Drop,
}

/// PII categories the pattern engine recognises.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "api", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum PiiCategory {
    Email,
    Phone,
    Ssn,
    CreditCard,
    PostalCode,
    IpAddress,
    Iban,
    SwiftBic,
    DateOfBirth,
    /// Person name, surfaced by the optional NER backend.
    Person,
    /// Organization name, surfaced by the optional NER backend.
    Organization,
    /// Location, surfaced by the optional NER backend.
    Location,
    /// Caller-supplied custom category (e.g. internal employee IDs).
    ///
    /// Surfaced by the redaction engine when a hit comes from
    /// [`RedactionConfig::custom_terms`](crate::core::config::redaction::RedactionConfig::custom_terms)
    /// or [`RedactionConfig::custom_patterns`](crate::core::config::redaction::RedactionConfig::custom_patterns).
    /// The string is the label passed alongside the term/pattern. Use those
    /// fields rather than constructing `Custom` directly via the
    /// `categories` filter — the pattern engine cannot detect arbitrary text
    /// from a category name alone.
    Custom(String),
}
