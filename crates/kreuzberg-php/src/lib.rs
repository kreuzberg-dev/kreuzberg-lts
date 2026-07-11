//! Kreuzberg PHP Bindings
//!
//! This module exposes the Rust core extraction API to PHP using ext-php-rs.
//!
//! # Architecture
//!
//! - All extraction logic is in the Rust core (crates/kreuzberg)
//! - PHP is a thin wrapper that adds language-specific features
//! - Zero duplication of core functionality
//! - Modern ext-php-rs patterns throughout

#![cfg_attr(windows, feature(abi_vectorcall))]
#![allow(dead_code, unused_imports)]

use ext_php_rs::builders::FunctionBuilder;
use ext_php_rs::prelude::*;
use ext_php_rs::types::Zval;
use once_cell::sync::Lazy;

pub mod async_extraction;
pub mod config;
pub mod deferred;
pub mod embeddings;
pub mod error;
pub mod extraction;
pub mod plugins;
pub mod types;
pub mod validation;

/// Global Tokio runtime for async worker threads.
///
/// Initializes once per PHP process (persists across requests in PHP-FPM).
/// Thread count is configurable via `KREUZBERG_PHP_WORKER_THREADS` env var.
///
/// Stored as `Result` so that construction errors can be surfaced at call time
/// as PHP exceptions rather than causing a process-level panic.
pub(crate) static WORKER_RUNTIME: Lazy<Result<tokio::runtime::Runtime, String>> = Lazy::new(|| {
    let worker_threads = std::env::var("KREUZBERG_PHP_WORKER_THREADS")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or_else(|| std::cmp::max(2, num_cpus::get() / 2));

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(worker_threads)
        .enable_all()
        .thread_name("kreuzberg-php-worker")
        .build()
        .map_err(|e| format!("Failed to create Tokio runtime for PHP async workers: {e}"))
});

/// Returns a reference to the global Tokio runtime, or a PHP exception if it
/// failed to initialise.
pub(crate) fn worker_runtime() -> PhpResult<&'static tokio::runtime::Runtime> {
    WORKER_RUNTIME.as_ref().map_err(|e| PhpException::from(e.clone()))
}

#[ctor::ctor(unsafe)]
fn setup_onnx_runtime_path() {
    kreuzberg::ort_discovery::ensure_ort_available();
}

/// Get the Kreuzberg library version.
///
/// # Returns
///
/// Version string in semver format (e.g., "4.10.0")
///
/// # Example
///
/// ```php
/// $version = kreuzberg_version();
/// echo "Kreuzberg version: $version\n";
/// ```
#[php_function]
pub fn kreuzberg_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Kreuzberg PHP extension module.
///
/// Exports all extraction functions, configuration types, error handling, and plugin management.
#[php_module]
pub fn get_module(module: ModuleBuilder) -> ModuleBuilder {
    // via the #[ctor::ctor] attribute before this function is called

    let mut module = module.function(wrap_function!(kreuzberg_version));

    for builder in config::get_function_builders() {
        module = module.function(builder);
    }
    for builder in embeddings::get_function_builders() {
        module = module.function(builder);
    }
    for builder in error::get_function_builders() {
        module = module.function(builder);
    }
    for builder in extraction::get_function_builders() {
        module = module.function(builder);
    }
    for builder in plugins::get_function_builders() {
        module = module.function(builder);
    }
    for builder in validation::get_function_builders() {
        module = module.function(builder);
    }
    for builder in async_extraction::get_function_builders() {
        module = module.function(builder);
    }

    module = module
        .enumeration::<types::ContentLayer>()
        .enumeration::<types::ElementType>()
        .enumeration::<types::KeywordAlgorithm>()
        .enumeration::<types::OcrElementLevel>()
        .enumeration::<types::OutputFormat>()
        .enumeration::<types::PageUnitType>()
        .enumeration::<types::RelationshipKind>()
        .enumeration::<types::ResultFormat>()
        .enumeration::<types::UriKind>()
        .enumeration::<types::PdfAnnotationType>();

    module = module
        .class::<types::Metadata>()
        .class::<types::ExtractedImage>()
        .class::<types::ExtractedTable>()
        .class::<types::ChunkMetadata>()
        .class::<types::TextChunk>()
        .class::<types::PageResult>()
        .class::<types::Keyword>()
        .class::<types::PdfAnnotation>()
        .class::<types::ProcessingWarning>()
        .class::<types::LlmUsage>()
        .class::<types::BoundingBoxType>()
        .class::<types::UriType>()
        .class::<types::ExtractionResult>()
        .class::<types::ArchiveEntry>()
        .class::<types::ExtractionConfigType>()
        .class::<types::TableType>();

    module = module.class::<deferred::DeferredResult>();

    module = module.class::<embeddings::EmbeddingPreset>();

    module = module.class::<error::ErrorClassification>();

    module
}
