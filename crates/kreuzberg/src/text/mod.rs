pub mod utf8_validation;

#[cfg(feature = "quality")]
pub mod quality;

#[cfg(feature = "quality")]
pub mod string_utils;

#[cfg(feature = "quality")]
pub mod token_reduction;

#[cfg(feature = "quality")]
pub mod quality_processor;

#[cfg(feature = "quality")]
pub use quality_processor::QualityProcessor;

#[cfg(feature = "quality")]
pub use token_reduction::{ReductionLevel, TokenReductionConfig};

// OSS v5 follow-up text-analysis modules. Each subsystem is feature-gated so the
// non-OSS targets (no-ort-target, wasm-target, android-target) compile out cleanly.
#[cfg(feature = "ner")]
pub mod ner;
#[cfg(feature = "redaction")]
pub mod redaction;
#[cfg(feature = "summarization")]
pub mod summarization;
#[cfg(all(feature = "translation", not(target_os = "windows")))]
pub mod translation;
#[cfg(all(feature = "classification", not(target_os = "windows")))]
pub mod classification;
