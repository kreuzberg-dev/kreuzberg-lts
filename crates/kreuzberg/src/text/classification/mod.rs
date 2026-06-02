//! Per-page LLM classification.
//!
//! Walks the rendered `content`, slices it on the page boundary metadata produced
//! during extraction, and asks the configured LLM to assign one or more labels
//! from a fixed vocabulary to each page. Results land on
//! [`ExtractionResult::page_classifications`](crate::types::ExtractionResult::page_classifications).
//!
//! Triggered by [`ExtractionConfig::page_classification`](crate::core::config::ExtractionConfig::page_classification);
//! invoked by the Middle-stage post-processor in
//! [`crate::plugins::processor::builtin::classification`].

pub mod page_classifier;

pub use page_classifier::classify_pages;
