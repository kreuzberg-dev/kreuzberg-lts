//! Build script for the wasm bindings.
//!
//! Link arguments live here rather than in `.cargo/config.toml` because CI sets
//! `RUSTFLAGS=-D warnings` as an environment variable, and cargo treats the
//! `RUSTFLAGS` env var as a single mutually-exclusive source that fully replaces
//! any `target.*.rustflags` from config files. Config-based link args are
//! therefore silently dropped in CI (observable as the wasm module falling back
//! to the default 1 MiB stack). Build-script link args are applied regardless of
//! which `RUSTFLAGS` source cargo chooses, so they survive.

/// wasm-ld stack size for the cdylib (16 MiB). Tesseract/pdfium wasm code paths
/// overflow the 1 MiB default.
const WASM_STACK_SIZE_BYTES: u32 = 16 * 1024 * 1024;

fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch != "wasm32" {
        return;
    }

    // kreuzberg-tesseract (Tesseract/Leptonica) and the transitive tree-sitter
    // crate each statically embed their own copy of the WASI SDK libc objects
    // (e.g. assert.o -> __assert_fail) into their rlib via cc-rs. Both copies come
    // from the same WASI sysroot and are functionally identical, so the wasm-ld
    // duplicate-symbol error is a visibility artifact, not an ABI conflict.
    // Neither source is ours to patch, so keep the first definition wasm-ld sees.
    println!("cargo::rustc-link-arg-cdylib=--allow-multiple-definition");
    println!("cargo::rustc-link-arg-cdylib=-zstack-size={WASM_STACK_SIZE_BYTES}");
}
