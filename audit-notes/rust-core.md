# Rust Core + FFI + CLI Bug Audit

Scope: `crates/kreuzberg/`, `crates/kreuzberg-ffi/`, `crates/kreuzberg-cli/`, `e2e/rust/`

---

## ROOT_CAUSE findings (fixed in this session)

### F-1: FFI integration tests could not link — missing `rlib` crate type

**File:** `crates/kreuzberg-ffi/Cargo.toml`

**Severity:** High — all FFI integration tests were silently unlinkable

**Root cause:** `crate-type = ["cdylib", "staticlib"]` — without `"rlib"`, integration tests cannot
`use kreuzberg_ffi::...` because there is no Rust metadata artifact for the linker to resolve.
Tests in `crates/kreuzberg-ffi/tests/` that used `kreuzberg_ffi::kreuzberg_*` symbols all failed
with `error[E0432]: unresolved import kreuzberg_ffi`.

**Fix:** Added `"rlib"` to the `crate-type` list.

**Status:** Fixed. Both `vtable_bytes_len` and `email_attachment_data_len` integration test suites
now compile and link.

---

### F-2: `email_attachment_data_len.rs` tests called obsolete 1-parameter API

**File:** `crates/kreuzberg-ffi/tests/email_attachment_data_len.rs`

**Severity:** High — active regression tests would have caught the wrong function signature

**Root cause:** The alef-generated `kreuzberg_email_attachment_data` was already updated to the
2-parameter form (`ptr, out_len: *mut usize`), but the test file had two active test bodies still
calling the old 1-parameter form. Two previously-`#[ignore]`-annotated tests (the core correctness
checks for the out_len contract) were also never un-ignored after the alef fix.

**Fix:** Rewrote the test file entirely:
- Un-ignored `email_attachment_data_accessor_must_provide_out_len_in_header` (header audit)
- Un-ignored `email_attachment_data_with_out_len_returns_full_buffer_including_embedded_nuls`
- Updated `email_attachment_data_none_returns_null_pointer` to the 2-param call form
- Added new `email_attachment_data_null_out_len_is_safe` to verify null out_len pointer is safe

**Status:** Fixed. All four tests pass.

---

### F-3: PDF extractor injected image placeholders without `inject_placeholders` guard

**File:** `crates/kreuzberg/src/extractors/pdf/mod.rs` (lines 508-522 before fix)

**Severity:** Medium — callers setting only `pdf_options.extract_images = true` received
unexpected `![](image_N.jpeg)` Markdown placeholders

**Root cause:** Commit `b0cf59341f feat: mixed-mode OCR extraction (run_ocr_on_images)` introduced
a new `ocr_page_rasters` field and reorganised image injection. The OCR path's dedicated injection
block (guarded by `config.images.inject_placeholders`) was added correctly. However, the earlier
native-path injection block at lines 508-522 was left ungated. When `force_ocr: true` and
`pdf_options.extract_images = true` (but `config.images` is `None`), the code path:
1. `used_ocr = true` → `use_structured_doc = false` (line 443)
2. `extracted_images` populated from native PDF parse (respecting `pdf_options.extract_images`)
3. Native block (lines 508-522): `if let Some(imgs) = images { if !use_structured_doc { push elements } }`
   fired unconditionally — no `inject_placeholders` check
4. OCR block (lines 562-593): also guarded but now redundant since native block already pushed

**Fix:** Added `inject_placeholders` gate to the native block:
```rust
let inject_placeholders = config.images.as_ref().is_some_and(|c| c.inject_placeholders);
if !use_structured_doc && inject_placeholders {
    // push image elements
}
```

`doc.images = imgs` (storing images in the result) is unchanged — callers still receive extracted
image data, just without Markdown placeholders unless they opt in.

**Status:** Fixed. Both inject_placeholder tests pass:
- `test_inject_placeholders_present_on_force_ocr_path` — still passes
- `test_inject_placeholders_absent_when_only_pdf_options_set` — now passes

---

## SAFE — verified not bugs

### S-1: `semaphore.acquire().await.unwrap()` in batch.rs (line 86)

`Semaphore::new(n)` never produces a closed semaphore; `acquire()` only returns `Err` when the
semaphore is explicitly closed (via `close()`). This semaphore is never closed. The `unwrap()` is
safe. The `#[allow(clippy::unwrap_used)]` annotation is appropriate.

### S-2: `slots[index].lock().take().expect(...)` in batch.rs (line 320)

Each slot is filled exactly once and consumed exactly once in the join loop. The `expect()` is an
invariant assertion, not a fallible path.

### S-3: `try_into().unwrap()` in cache/core.rs (lines 172-174, 205)

All conversions are on fixed-size byte slices where the source length is verified before conversion.
Safe.

### S-4: `expect("checked non-empty")` in llm/vlm_embeddings.rs (line 49)

Guarded by `if input_strings.len() == 1` immediately above. Safe.

### S-5: FFI `block_on` pattern

`get_ffi_runtime()` uses `OnceLock<Runtime>` creating a dedicated Tokio runtime (separate from any
caller runtime). `block_on` is called from C-exported `extern "C"` functions — never from within
an existing Tokio context. No re-entrant deadlock risk.

### S-6: `InternalElementId::as_str()` `unwrap()` (types/internal.rs line 102-104)

The buffer is always populated with lowercase ASCII hex characters by `InternalElementId::new()`.
The `unwrap()` on `from_utf8` is invariantly safe. Could be replaced with
`unsafe { std::str::from_utf8_unchecked(&self.0) }` + SAFETY comment for clarity, but is not a
production risk.

### S-7: `unsafe` blocks in embeddings/mod.rs (flock), core/io.rs (mmap)

All have SAFETY comments explaining the invariants. Correct.

---

## LATENT — pre-existing, not yet fixed

### L-1: `PaddleOcrBackend::default()` panics in production

**File:** `crates/kreuzberg/src/paddle_ocr/backend.rs` (~line 607)

```rust
impl Default for PaddleOcrBackend {
    fn default() -> Self {
        Self::with_config(PaddleOcrConfig::default())
            .unwrap_or_else(|e| panic!("Failed to create default PaddleOcrBackend: {}", e))
    }
}
```

`Default::default()` for a type that requires resource initialisation should not exist (or should
return `Result`). The production registry (`plugins/registry/ocr.rs:103`) uses `PaddleOcrBackend::new()`
with proper error handling, not `Default`. However, the `Default` impl remains as a footgun for any
future caller (including test helpers) and violates "never panic in library code".

**Recommended fix:** Remove the `Default` impl and replace test usages with explicit `::new()` calls.
If `Default` is truly needed for trait bounds, implement it to return a sentinel/no-op and document
the degraded behaviour.

---

## OUT OF SCOPE — tracked for other engineers

### O-1: WASM `FormatMetadata::Code` non-exhaustive match

**File:** `crates/kreuzberg-wasm/src/lib.rs` (alef-generated, ~line 19672)

`From<kreuzberg::FormatMetadata>` does not handle `Code(tree_sitter_language_pack::ProcessResult)`.
When `task rust:test` runs with `--all-features` on `kreuzberg-wasm`, workspace feature unification
enables `tree-sitter` in that crate, exposing the `Code` variant and causing a non-exhaustive match
compile error.

Root fix: update the alef template for `kreuzberg-wasm`'s `FormatMetadata` conversion to add a
`#[cfg(feature = "tree-sitter")]` arm or a wildcard with a comment. Out of scope for this audit
(wasm-specialist).

---

## Summary

| ID  | Status | File                                      | Severity |
|-----|--------|-------------------------------------------|----------|
| F-1 | Fixed  | `crates/kreuzberg-ffi/Cargo.toml`         | High     |
| F-2 | Fixed  | `crates/kreuzberg-ffi/tests/email_attachment_data_len.rs` | High |
| F-3 | Fixed  | `crates/kreuzberg/src/extractors/pdf/mod.rs` | Medium |
| L-1 | Latent | `crates/kreuzberg/src/paddle_ocr/backend.rs` | Low-medium |
| O-1 | Out of scope | `crates/kreuzberg-wasm/src/lib.rs`  | — |
