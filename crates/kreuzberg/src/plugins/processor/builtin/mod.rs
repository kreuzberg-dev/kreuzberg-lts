//! Built-in post-processors that ship with kreuzberg.
//!
//! Each submodule registers a single [`PostProcessor`](crate::plugins::PostProcessor)
//! implementation behind its own feature gate so non-OSS targets (no-ort-target,
//! wasm-target, android-target) compile out cleanly. Modules added by parallel
//! work streams plug in here without touching one another's files.

#[cfg(all(feature = "classification", not(target_os = "windows")))]
pub mod classification;

#[cfg(feature = "summarization")]
pub mod summarization;

#[cfg(all(feature = "translation", not(target_os = "windows")))]
pub mod translation;

#[cfg(all(feature = "captioning", not(target_os = "windows")))]
pub mod captioning;

#[cfg(feature = "qr-codes")]
pub mod qr;

#[cfg(feature = "ner")]
pub mod ner;

#[cfg(feature = "redaction")]
pub mod redaction;

/// Register every built-in post-processor enabled by the active feature set.
///
/// This is the single entry point that callers (including
/// `register_default_post_processors`) use to populate the global
/// post-processor registry with the in-tree built-ins. Each submodule's own
/// `register` function is gated by its feature flag so this aggregate stays
/// safe to call on any target.
pub fn register_builtin() -> crate::Result<()> {
    #[cfg(all(feature = "classification", not(target_os = "windows")))]
    classification::register()?;

    #[cfg(feature = "summarization")]
    summarization::register()?;

    #[cfg(all(feature = "translation", not(target_os = "windows")))]
    translation::register()?;

    #[cfg(all(feature = "captioning", not(target_os = "windows")))]
    captioning::register()?;

    #[cfg(feature = "qr-codes")]
    qr::register()?;

    #[cfg(feature = "ner")]
    ner::register()?;

    #[cfg(feature = "redaction")]
    redaction::register()?;

    Ok(())
}
