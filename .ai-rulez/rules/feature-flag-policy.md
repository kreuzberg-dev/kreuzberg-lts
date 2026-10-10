---
type: Decision
title: Feature Flag Policy
x-ai-rulez:
  kind: rule
  id: feature-flag-policy
  metadata:
    priority: high
---

# Feature Flag Policy

All features in `crates/kreuzberg/Cargo.toml`.

## WASM-Incompatible Features

- `ocr` — uses `tiff`, `fast_image_resize`: not WASM-safe; use `ocr-wasm` instead
- `paddle-ocr` — ONNX Runtime + native C++ deps: not WASM-safe; no WASM equivalent
- `excel` — requires `tokio-runtime`; use `excel-wasm` on WASM
- `tree-sitter` — uses dynamic loading; use `tree-sitter-wasm` on WASM
- `wasm-target` feature composes the complete safe WASM-compatible set

## ORT Variants (Mutually Exclusive)

- `ort-bundled` — downloads official Microsoft ORT binaries; default when OCR/ML features active
- `ort-dynamic` — load ORT from system; only when system ORT is guaranteed present

## Platform-Conditional

- `kreuzberg-paddle-ocr`, `hf-hub`, `pprof` — excluded on `wasm32`
- `ureq`: `rustls` on non-Windows; `native-tls` on Windows

## Aggregate Sets

| Feature | Description |
|---------|-------------|
| `formats` | All document formats + api/mcp/otel/chunking; no OCR, no ML |
| `full` | `formats` + ocr + paddle-ocr + layout + embeddings + tree-sitter + liter-llm |
| `wasm-target` | Full safe WASM-compatible set |

## Build Profiles

- `release` — LTO thin, codegen-units=1, strip
- `profiling` — inherits release, retains debug info
- `kreuzberg-wasm` override: `opt-level="z"` (size-optimized)
- `sevenz-rust2`, `zip` override: `opt-level=2` (prevents SIGBUS on macOS ARM64)
