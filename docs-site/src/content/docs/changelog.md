---
title: Changelog
description: "Release history for Kreuzberg v4 LTS."
---

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [4.10.0] - 2026-07-11

First release of the standalone **Kreuzberg v4 LTS** line. This is the long-term-support home for
Kreuzberg v4; active development continues as [Xberg](https://github.com/xberg-io/xberg) (v5+). No
behavioural changes to extraction — this release is a security, licensing, and packaging refresh.

### Security

- Upgraded core dependencies to their latest patched releases (`hf-hub`, `rmcp`, `lopdf`, `liter-llm`, `pyo3`, `comrak`, `calamine`, `quick-xml`, `text-splitter`, `tower-http`, and others).

### Changed

- **License is now MIT** (earlier v4 releases shipped under the Elastic License 2.0).
- Repository moved to standalone **[`kreuzberg-dev/kreuzberg-lts`](https://github.com/kreuzberg-dev/kreuzberg-lts)**. New v4 Go releases publish under `github.com/kreuzberg-dev/kreuzberg-lts/v4`; existing `github.com/kreuzberg-dev/kreuzberg` pins keep resolving via the Go module proxy cache.

### Added

- LTS support policy and migration guide at [docs.kreuzberg.dev/lts](https://docs.kreuzberg.dev/lts/): v4 receives critical bug and security fixes until the end of 2026 on a best-effort basis. The **R binding remains exclusive to the v4 LTS line** (not part of Xberg v5).

## [4.9.9] - 2026-06-05

LTS patch release with PDF/OCR robustness fixes and selected stability
backports from main.

### Fixed

- **OCR on wide, vector-heavy single-page PDFs**: uses a bounded render profile and retries at a lower cap instead of failing extraction with `PdfiumLibraryInternalError(Unknown)`.
- **Embedded PDF image OCR**: unsupported image streams are re-extracted through Pdfium or skipped with a specific warning instead of repeated `image dimension probe failed` noise.
- **RTF/MSG decompression**: cap the initial allocation hint so a crafted stream cannot request a multi-gigabyte allocation from an untrusted size header.
- **Table row sorting** is now NaN-safe (no longer panics).
- **FFI embedding preset symbols** are exported even when embeddings are disabled, preventing Java/native binding startup from failing with missing symbols.
- **Chunking** now uses formatted content for non-plain output formats, preserves page metadata, and normalizes trailing-space page artifacts before boundary matching.
- **UTF-16 email transcoding**: short binary inputs are no longer misclassified as UTF-16.
- Returns a validation error when `extraction_timeout_secs` is set without `tokio-runtime`.

## [4.9.8] - 2026-05-17

LTS patch release. Four targeted bug fixes plus dependency pinning so the
branch builds against current crates.io releases.

### Fixed

- **RTF hex escapes** now honor `\ansicpgNNNN`, so CP1251 Cyrillic decodes as readable text instead of Windows-1252 mojibake.
- **Python `ExtractionConfig(cancel_token=…)`** no longer raises `TypeError`; the kwarg is accepted and threaded through to the underlying cancellation token.
- **C# `OcrConfig`** gains the missing `VlmConfig` property and the previously-undefined `LlmConfig` type (`Model`, `ApiKey`, `BaseUrl`, `TimeoutSecs`, `MaxRetries`, `Temperature`, `MaxTokens`).
- **musl CLI tarball** now bundles all transitive ONNX Runtime deps, fixing startup failure on any host.
- **Build compatibility**: pinned `tokenizers` and `v_htmlescape` to versions compatible with the current API.

## [4.9.7] - 2026-05-08

LTS patch release. Publish-pipeline fixes only — no library code changes.

- Maintenance release (internal changes only).

## [4.9.6] - 2026-05-07

LTS patch release. Bug fixes backported from `main` (v5 development). The
`chore/v4.9-lts` branch is now the long-lived line for the 4.9.x series.

### Fixed

- **`max_images_per_page` cap** is now enforced in image extraction itself — previously PDFs with thousands of image objects per page hung indefinitely. Image decoding also moved off the async executor so `extraction_timeout_secs` can fire while images are processing.
- **OCR elements** are now propagated through the extraction pipeline.
- **`extraction_timeout_secs`** is now enforced in single-file extraction paths (previously only multi-file batch flows).
- **PDF image data** no longer leaks into structured output when image extraction is disabled.
- **Preset-only chunking config** no longer auto-injects an unwanted `EmbeddingConfig` on every chunk.
- **PDF heading and image-placeholder classification** corrected.
- On the **WASM target**, OCR PSM now defaults to `SINGLE_BLOCK` (native default unchanged).
- **`HwpExtractor`** no longer claims the `application/haansofthwpx` MIME type (that format is ZIP-based XML, not CFB-based HWP).
- **Email HTML body fallback**: HTML-only emails that the mail parser failed to surface now return content instead of empty output.
- **Image decode pixel cap**: attacker-controlled image bytes above 64 MP are rejected with a clean error instead of triggering multi-GB allocations.
- **CLI `--log-level`** no longer panics on malformed input (falls back to `info`).
- **Markdown rendering**: collapse runs of 3+ consecutive newlines to exactly 2.
- **MCP `file_configs` schema**: emits a schema form accepted by Moonshot AI / Kimi.

### API surface

- `extract_images_from_pdf`, `extract_images_from_pdf_with_password`, and `PdfImageExtractor::{extract_images, get_image_count, extract_images_from_page}` gain a required `max_images_per_page: Option<u32>` parameter. Pass `None` to preserve previous unbounded behaviour.

### Not backported from main

The following fixes on `main` could not be applied to v4.9.x because they build on v5 architecture:

- #834 (DOCX `inject_placeholders`/OCR pipeline integration).
- #799 (Form XObject image extraction).
- #824 (image extraction across XObject references).

---

## [4.9.5] - 2026-04-23

### Fixed

- **GPU acceleration**: kreuzberg now bundles CPU-only ONNX Runtime by default. When a GPU execution provider (`cuda`, `tensorrt`, `coreml`) is explicitly requested but unavailable, it returns an error with setup instructions instead of silently using CPU; `Auto` mode falls back to CPU with an info log. For GPU support, set `ORT_DYLIB_PATH` to a GPU-enabled ONNX Runtime.
- **DOCX OCR extraction**: OCR now runs on embedded images and its text is injected into rendered output, instead of being discarded and replaced with placeholder text.
- **PaddleOCR GPU (CUDA)**: `AccelerationConfig` is now propagated to all PaddleOCR ONNX sessions instead of silently falling back to CPU.
- **`PaddleOcrConfig`** is now exposed in Python bindings (with `OcrConfig` backward compatibility).
- **Ruby gem packaging**: excludes staged `libpdfium.dylib` from gem artifacts.

---

## [4.9.2] - 2026-04-19

### Fixed

- Cancellation token is now checked in the WASM (non-tokio) path for Excel, DOC, PPT, Pages, Keynote, and Numbers extractors — cancellation was previously silently ignored in WASM builds.
- Propagate the `Cancelled` error code (9) to all bindings (Go, C FFI, Python, TypeScript).

---

## [4.9.1] - 2026-04-19

### Fixed

- Preserve the `_internal_bindings.pyi` type stub during wheel cleanup — published wheels now include inline type information for the core binding module.

---

## [4.9.0] - 2026-04-18

### Fixed

- Suppress C23 glibc symbols in manylinux wheels to prevent incompatible symbols on glibc < 2.38 (Debian 12, Ubuntu 22.04).
- Remove `kreuzberg-cli` from the Python wheel to fix `libonnxruntime.so.1` loading failure — the CLI is available as a standalone release.
- **Cancellation token support**: cancelled extractions no longer block subsequent calls; wired across Python, Node.js, Ruby, WASM, and C FFI.
- Fix `kreuzberg[easyocr]` extra silently installing nothing on Python 3.14+.
- Fix ~1000x slowdown on Ghostscript-produced PDFs with structured output.
- Fix `llm_usage` returning `None` when using VLM-based OCR.

### Added

- Cancellation token API available in all language bindings (`CancellationToken` in Python/Node/Ruby/WASM/FFI).

### Changed

- **Breaking**: `kreuzberg-cli` binary is no longer bundled in the Python wheel — install the standalone CLI from GitHub releases.

---

## [4.9.4] - 2026-04-22

### Fixed

- **Ruby gem build failure** — missing `max_images_per_page` field in `ImageExtractionConfig` caused a compilation error on all platforms.
- **PaddleOCR GPU (CUDA)**: now correctly used when `AccelerationConfig(provider="cuda")` is set — previously the execution provider was never applied and it silently fell back to CPU.

---

## [4.9.3] - 2026-04-22

### Added

- **Layout detection regions on `PageContent`** — new `layout_regions` field exposes detected layout regions (class, confidence, bounding box, area fraction) from the RT-DETR model, enabling programmatic detection of diagrams, figures, tables, and other content types per page. Available across all 10 bindings.

### Fixed

- **`PaddleOcrConfig` in Python API**: exposed as a first-class class; `OcrConfig` accepts both objects and raw dictionaries.
- **DOCX page extraction (`extract_pages=True`)**: `result.pages` and `get_page_count()` now work correctly instead of always returning `None`/`0`. (Tables spanning multiple pages remain a known limitation.)
- **`serve`/`mcp` CLI subcommands** now apply `KREUZBERG_*` environment overrides (previously only `extract` honoured them). `MISTRAL_API_KEY` is now picked up for bare `mistral-*` model names.
- **Tagged-PDF structure tree** no longer drops paragraph body text when a block has both text and children, and no longer emits malformed markdown for numbered section headings.
- **Semantic chunker fallback** now respects `max_characters` instead of a hardcoded 4000-char ceiling; warns when `chunker_type='semantic'` is used without an `EmbeddingConfig`.
- **OCR backend dispatch**: a non-default backend that errors no longer silently falls back to paddleocr; auto-fallback is limited to the default tesseract backend.
- **EasyOCR on PDFs**: Rust's renderer now handles page rendering, removing the implicit `pdf2image`/`pymupdf` requirement that was never declared in the `[easyocr]` extra.
- **`OcrConfig.vlm_prompt`** is now honored in VLM OCR requests (previously documented but never forwarded).
- **PDF image links** are no longer silently dropped from markdown output; respects `inject_placeholders`.
- **PDF with large numbers of image fragments** no longer hangs — added `ImageExtractionConfig.max_images_per_page` (default `None`) and honored `extraction_timeout_secs` at inter-page checkpoints.
- **PST extractor** now populates email attachments (name, filename, MIME type, size, data); entry IDs formatted as proper MAPI hex strings.

### Added

- `ImageExtractionConfig.max_images_per_page` — optional cap on images decoded per page; prevents hangs on PDFs with thousands of inline image fragments.

---

## [4.8.6] - 2026-04-17

### Added

- **PST message EntryID in metadata** — the `entry_id` field from Outlook PST message entries is now included in the `metadata` of `EmailExtractionResult`, letting callers link extracted data back to its source message.
- **AccelerationConfig wired through all ORT model loading** — CUDA/CoreML/TensorRT/Auto acceleration is now propagated to all ONNX Runtime sessions (layout detection, embeddings, document orientation, PaddleOCR); previously GPU acceleration was silently ignored. The `acceleration` field is added to `LayoutDetectionConfig` and `EmbeddingConfig` across all 11 bindings.
- **Semantic chunker** (`ChunkerType::Semantic`) for topic-aware document splitting, with a `topic_threshold` config field.

### Fixed

- **Batch extraction crash on ARM64 Linux** ("Lazy instance has previously been poisoned") — OCR backend init failures no longer poison a shared static and cascade to all concurrent batch tasks.
- **PaddleOCR `model_tier` from TOML ignored by API server** — per-request `model_tier` is now honored.
- **VLM OCR backend ignored when paddle-ocr feature enabled** — `vlm_config` is now propagated so configured VLM OCR is actually used.
- **Doubled OCR content and corrupted page text in image extraction** — OCR content is no longer duplicated word-by-word into `content`/`pages`.
- **Image OCR `pages[]` empty** — element output is now forced on for image extraction.
- **`LlmConfig` missing `Default`** — the documented `..Default::default()` pattern now compiles.
- **LLM embedding provider panics in server mode** — embedding no longer panics when called inside an existing tokio runtime (HTTP/MCP server).
- **OCR table metadata serialized as strings instead of numbers** (`table_count`, `tables_detected`, `table_rows`, `table_cols`), which broke numeric comparisons in all bindings; also fixed a duplicate `output_format` metadata key.
- **Ruby `structured_output`** was missing from the `Result` class and not serialized.
- **DOCX tables assigned wrong page numbers** — now numbered by actual document position.
- **`ocr.enabled=false` ignored** — OCR no longer runs when explicitly disabled; also fixed a dropped trailing newline in `--format text` output.

### Changed

- Updated dependencies including html-to-markdown-rs 3.1→3.2 and tokio 1.51→1.52.

---

## [4.8.5] - 2026-04-14

### Added

- **LLM usage tracking** — new `llm_usage` field on `ExtractionResult` captures token counts, estimated cost (USD), model identifier, and finish reason for every LLM call (VLM OCR, structured extraction, LLM embeddings). Exposed across all bindings.

### Fixed

- **Markdown chunker** no longer duplicates a heading when `prepend_heading_context` is enabled and a chunk boundary aligns with a heading.
- **Python wheel requires glibc ≥ 2.38 (breaks Debian 12, Ubuntu 22.04)** — downgraded to `manylinux_2_28` and suppressed C23 glibc symbol emission so wheels install on systems with glibc < 2.38.
- **FFI memory leak** — `kreuzberg_free_result` now frees `djot_content_json`, `structured_output_json`, and `llm_usage_json`.

---

## [4.8.4] - 2026-04-13

### Added

- **Helm chart for Kubernetes deployment** — minimal, security-hardened chart (Deployment, Service, Ingress, PVC, HPA, PDB, ServiceAccount), published to GHCR as an OCI artifact.

### Fixed

- **Comrak bridge panic on multi-byte UTF-8 boundaries** — annotation offsets landing inside multi-byte characters (e.g. Cyrillic) no longer cause panics.

---

## [4.8.3] - 2026-04-12

### Fixed

- **ONNX session creation fails on Linux x86-64** ("graph_optimization_level is not valid") — the Linux wheel bundled an incompatible ORT version. Switched to a portable optimization level and aligned all ORT versions to 1.24.2.

### Documentation

- **Documented AVX/AVX2 CPU requirement for ONNX Runtime features** — CPUs without AVX (e.g. Intel Atom, Celeron N5105) cannot use PaddleOCR, layout detection, or embeddings.

---

## [4.8.2] - 2026-04-10

### Added

- **`HtmlOutputConfig` typed in all bindings** — the `html_output` config field (themes, CSS classes, embed CSS, custom CSS, class prefix) is now fully typed in Python, TypeScript/Node, Go, Ruby, Elixir, PHP, Java, C#, R, and FFI (previously Rust-only).

### Fixed

- **PDF: legitimate repeated content stripped regardless of `strip_repeating_text`** — page-merge deduplication ran unconditionally, removing legitimately repeated brand names and other content even when content filtering was disabled. Both dedup passes are now gated behind the flag.

---

## [4.8.1] - 2026-04-09

### Added

- **Styled HTML output** — new `HtmlOutputConfig` on `ExtractionConfig` with 5 built-in themes (`default`, `github`, `dark`, `light`, `unstyled`), semantic `kb-*` CSS class hooks, CSS custom properties, custom CSS injection, and configurable class prefix. The existing `Html` output format is upgraded in-place when `html_output` is set.
- 5 new CLI flags: `--html-theme`, `--html-css`, `--html-css-file`, `--html-class-prefix`, `--html-no-embed-css` — any implicitly sets `--content-format html`.

### Changed

- **Vendored yake-rust 1.0.3** into core — fixes a `BacktrackLimitExceeded` panic on large files (10+ MB) and expands YAKE stopwords from 34 to 64 languages.

### Fixed

- **PPTX**: panic on non-char-boundary during page boundary recomputation (multi-byte UTF-8 characters).
- **PDF**: `include_headers`/`include_footers` flags were ignored by layout-model furniture stripping; they now correctly preserve those regions.
- **PDF**: heuristic table detector no longer misclassifies body text as tables on slide-like PDFs (rejects ≤3-row tables spanning >50% of page height).
- **PPTX/DOCX/HTML/DocBook/LaTeX/RST**: `ImageExtractionConfig.inject_placeholders=false` now correctly suppresses image references.

---

## [4.8.0] - 2026-04-08

### Added

- **Cross-extractor content filtering** — new `ContentFilterConfig` on `ExtractionConfig` with `include_headers`, `include_footers`, `strip_repeating_text`, and `include_watermarks` flags across PDF, DOCX, RTF, ODT, HTML, EPUB, and PPT extractors. Typed in all bindings.
- **Local LLM support** via liter-llm 1.2 — use Ollama, LM Studio, vLLM, llama.cpp, LocalAI, or llamafile as VLM OCR, embedding, or structured-extraction backends with zero API key configuration.
- **LLM-powered document intelligence** — integrates with 146 LLM providers for three capabilities: **VLM OCR** (vision models as OCR backend for low-quality scans, handwriting, Arabic/Farsi, complex layouts, via `ocr.backend = "vlm"`), **structured extraction** (JSON-schema-constrained extraction), and **VLM embeddings** (provider-hosted embedding models).
- **New CLI command** `kreuzberg extract-structured`, **API endpoint** `POST /extract-structured`, and **MCP tool** `extract_structured` for schema-guided LLM extraction.
- **Minijinja template engine** for customizable LLM prompts.
- **5 new environment variables**: `KREUZBERG_LLM_MODEL`, `KREUZBERG_LLM_API_KEY`, `KREUZBERG_LLM_BASE_URL`, `KREUZBERG_VLM_OCR_MODEL`, `KREUZBERG_VLM_EMBEDDING_MODEL`.
- `LlmConfig` and `StructuredExtractionConfig` types, `structured_output` field on `ExtractionResult`, and `EmbeddingModelType::Llm` variant across bindings.
- **Standalone text embedding API** with `/embed` endpoint, `embed_text` MCP tool, and `embed` CLI command.

### Changed

- **License changed from MIT to Elastic License 2.0 (ELv2)** — copyright holder changed to Kreuzberg, Inc. Forked crates retain their original MIT licenses.
- API returns 501 Not Implemented (instead of 500) when the liter-llm feature is disabled.
- JSON schema `additionalProperties` automatically stripped for non-OpenAI providers.

### Fixed

- **PDF: brand names stripped by repeating-text detection** — `strip_repeating_text = false` now disables the removal that incorrectly stripped brand names from PowerPoint-exported decks.
- **PPTX: slide order scrambled for decks with 10+ slides** — fixed lexicographic sort (`slide10.xml` before `slide2.xml`) to use numeric ordering.
- **UTF-8 panic in arXiv watermark stripping** when a multi-byte character spans the search limit.
- **DOC: garbled text from old Word files** — CP1252 text misread as UTF-16LE; added heuristic to detect and re-decode.
- **WASM: table extraction returns empty array** when `pageNumber` is null (now defaults to page 0).

---

## [4.7.4] - 2026-04-06

### Added

- Re-added the `--layout` boolean CLI flag for easy layout detection enablement (`--layout` to enable with model defaults, `--layout false` to disable).
- arXiv watermark/sidebar noise filtering for academic PDFs.
- Second-tier cross-page repeating-text detection for conference headers and journal running titles outside the margin zone.
- Figure/picture text suppression — text inside layout-detected Picture regions is excluded from body output.

### Fixed

- **Figure-internal text leaking into body output** — diagram labels and axis text are no longer included (sometimes promoted to headings) in extracted body content.
- **Empty image references in PDF markdown/HTML output** — PDFs with embedded images no longer produce empty `![]()` / `<img src="">`; actual image pixel data is now extracted, producing proper `![](image_N.png)` references.
- **WASM build failure with `extern "C-unwind"`** — added a macro that uses `extern "C-unwind"` natively and `extern "C"` on WASM.
- **Go module tag format** — tags now use the correct `packages/go/v4/vX.Y.Z` format matching the module path.

### Changed

- CLI documentation updated with missing extraction override flags (`--layout-table-model`, `--disable-ocr`, `--cache-namespace`, `--cache-ttl-secs`).

---

## [4.7.3] - 2026-04-05

### Fixed

- **Archive extraction SIGBUS crash on macOS ARM64** — ZIP, 7Z, TAR, and GZIP extraction crashed in release builds due to miscompilation under `opt-level=3`; reduced to level 2 for the affected crates.
- **Native-text PDF extraction fails when OCR backend unavailable** — PDFs with extractable native text no longer hard-fail with `All OCR pipeline backends failed`; the OCR quality-enhancement pass now falls back to the native result with a warning.
- **Rust cannot catch foreign exceptions crash** — C++ exceptions from Tesseract or Leptonica (e.g. on corrupted images) no longer abort the process; they now unwind safely and convert to recoverable errors.

---

## [4.7.2] - 2026-04-04

### Changed

- **Global model cache** — models now download to a platform-appropriate global cache (`~/.cache/kreuzberg/` on Linux, `~/Library/Caches/kreuzberg/` on macOS, `%LOCALAPPDATA%/kreuzberg/` on Windows) instead of per-directory `.kreuzberg/` folders. Override with `KREUZBERG_CACHE_DIR`.

### Fixed

- **Embedded HTML in PDF text layers** — raw HTML in a PDF text layer (`<p>`, `<br />`, `<a href>`) that previously produced escaped garbage is now converted to clean markdown.
- **Code classification false positives** — regular prose is no longer misclassified as Code blocks.
- **PageBreak rendering as `-----` separators** — PageBreak elements no longer pollute output with `-----` / `<hr>`; treated as structural metadata.
- **Leptonica DPI crash** — images with 0 DPI no longer trigger an uncatchable C++ exception; DPI is validated and fixed to 72 before preprocessing.
- **Node.js `ExtractionResult.children` missing at runtime** in the published v4.7.1 binary.
- **Node.js `disable_ocr` config not respected** — `disableOcr: true` no longer produces OCR content for images.
- **PaddleOCR angle classification crash** — fixed input dimensions to match the v2 angle classifier model.
- **Chunk page numbers missing** — chunks produced with `first_page`/`last_page` null when chunking was configured without explicit `pages` config; page tracking is now auto-enabled.

---

## [4.7.1] - 2026-04-03

### Added

- **Tree-sitter grammar management CLI** — new `kreuzberg tree-sitter` subcommand with `download`, `list`, `cache-dir`, and `clean` for managing grammar parsers (by language, group, or all); reads `[tree_sitter]` config from `kreuzberg.toml`.
- **Tree-sitter grammar management API** — new REST endpoints `POST /grammars/download`, `GET /grammars/list`, `GET /grammars/cache`, `DELETE /grammars/cache`.
- **Tree-sitter grammar management MCP tools** — `download_grammars`, `list_grammars`, `grammar_cache_info`, `clean_grammar_cache`.
- **Tree-sitter config startup initialization** — API and MCP servers auto-download grammars on startup when `[tree_sitter]` specifies `languages` or `groups`.

### Changed

- **Normalized OCR+layout pipeline** — the Tesseract+layout path now follows the same architecture as pdfium+layout, fixing destroyed paragraph structure and reading order.
- **Elixir NIF crash protection** — extraction and batch NIFs are wrapped so native-library panics (pdfium, tesseract) return `{:error, reason}` instead of crashing the BEAM VM.

### Fixed

- **hOCR parser depth tracking** — content after inner word spans is no longer silently dropped due to premature paragraph termination.
- **hOCR multi-page content loss** — content on pages 2+ is no longer dropped due to a per-page filter on per-page hOCR documents.
- **OCR batch parallelization** — page processing now scales with available CPUs (capped at 8) instead of a hardcoded 4, speeding up multi-page documents.
- **Chunking page boundary regression** — page boundaries are recomputed from rendered per-page content, fixing null `first_page`/`last_page` and a validation warning.
- **HF Hub environment variables** — respects `HF_HOME`/`HF_ENDPOINT`, fixing permission errors on Kubernetes when running as non-root.
- **PDF bridge tracing panic on multibyte characters** — no longer panics on multibyte UTF-8 (e.g. `•`).
- **Go FFI struct layout** — vendored C header was missing `children_json`, shifting all subsequent fields and reading wrong memory.
- **Java FFI struct layout** — `CExtractionResult` was missing `code_intelligence_json`, causing all Java extractions to return `success=false`.
- **PHP `__get` magic method bypass** — `elements`, `djotContent`, `document`, `ocrElements`, `children`, and `uris` returned raw JSON strings instead of deserialized arrays.
- **Ruby `disable_ocr` config** — the keyword was not parsed, so OCR ran even when explicitly disabled.
- **Node.js `ExtractionResult` parity** — `document`, `djotContent`, and `ocrElements` were omitted from JS objects when `None`; now default to `null`.
- **Node.js `convertChunk` missing `chunkType`** — the converter did not forward `chunk_type`.
- **ODT caption text extraction** — text inside `draw:text-box` (e.g. image captions) was not extracted.
- **Italian/European PDF ligature corruption** — repairs `tt`, `ti`, `tti` ligatures (e.g. `Dire*ore` → `Direttore`).
- **WASM Rayon thread pool panic** — parallel iteration now falls back to sequential on `wasm32` instead of panicking.

---

## [4.7.0] - 2026-03-30

### Added

- **Semantic chunk labeling**: chunks now include a `chunk_type` field identifying the semantic nature of the content (e.g. `paragraph`, `heading`, `list_item`, `table_cell`, `code_block`), across all 11 language bindings.
- **Image extraction across 8 formats**: embedded images extracted as `ExtractedImage` (binary data, format, dimensions, alt text) for DOCX, PPTX, PDF, EPUB, ODT, HTML, RTF, and Markdown/MDX/Jupyter. Markdown output renders as `![alt](image_N.ext)` with binary data in `ExtractionResult.images`.
- **Recursive OCR on embedded images**: extracted images from EPUB, ODT, HTML, and RTF are OCR'd, producing nested `ExtractionResult` in `ExtractedImage.ocr_result`.
- **PDF watermark artifact filtering** using pdfium's `/Artifact` content marks.
- **Vertical table header reconstruction**: fixes rotated PDF table column headers extracted as reversed spaced characters.
- **Pages API for PDF extraction**: per-page content is now available via `result.pages` for PDF documents.
- **TOON wire format**: token-efficient (~30-50% fewer tokens) JSON alternative across CLI (`--format toon`), API (`Accept: application/toon`), MCP (`response_format: "toon"`), and all 11 language bindings; losslessly convertible to/from JSON.
- **Renderer registry**: trait-based `Renderer`/`RendererRegistry` lets external crates register custom output-format plugins via `register_renderer()`.
- **URI extraction**: new `Uri` type with `UriKind` classification (Hyperlink, Image, Anchor, Citation, Reference, Email) extracted from 20+ formats, deduplicated, in `ExtractionResult.uris`.
- **Recursive email attachment extraction**: EML/MSG/PST attachments and nested `message/rfc822` parts extracted as `ArchiveEntry` children, respecting `max_archive_depth`.
- **PDF embedded file extraction**: PDF file attachments (portfolios) extracted as children (with filename sanitization, size limits, depth guards).
- **PDF bookmark/outline extraction**: document outlines extracted as URIs (page destinations as anchors, external links as hyperlinks).
- **DOCX/PPTX embedded object extraction**: OLE objects and embedded files extracted as children.
- **PPTX hyperlink extraction** from slide XML, resolved via relationship files.
- **Image path resolution for markup formats**: relative image paths in Markdown, MDX, LaTeX, RST, OrgMode, Typst, Djot, and DocBook are resolved from the filesystem (with path-traversal prevention) and extracted.
- **FictionBook image and link extraction** (base64 `<binary>` images and `<a>` hyperlinks).
- **Apple iWork improvements**: Numbers outputs tables, Keynote has improved slide structure, Pages has heading detection; all three extract metadata.
- **`code_intelligence` field on `ExtractionResult`**: top-level access to tree-sitter results (structure, imports, exports, chunks, symbols, diagnostics, docstrings).
- **`CodeContentMode` config**: control code extraction mode — `chunks` (default), `raw`, or `structure`.
- **TSLP semantic chunking for code**: code files use function/class-aware chunks with semantic types and heading context.

### Code Intelligence

- **Tree-sitter integration** for 248 programming languages: extract functions, classes, imports, exports, symbols, docstrings, diagnostics; syntax-aware chunking; language detection from extension/shebang; dynamic grammar download (native) / 30-language static subset (WASM); new `tree-sitter` and `tree-sitter-wasm` feature flags (in `full` and `wasm-target`); `TreeSitterConfig`/`TreeSitterProcessConfig` in `ExtractionConfig`.

### Typed Metadata

- New `FormatMetadata` variants: `Code`, `Csv`, `Bibtex`, `Citation`, `FictionBook`, `Dbf`, `Jats`, `Epub`, `Pst`; `PptxMetadata` extended with `image_count`/`table_count`; typed fields across all 11 bindings.

### Breaking Changes

- **Layout detection preset removed**: the `preset` field on `LayoutDetectionConfig` and the `--layout-preset` CLI flag are removed; layout detection now uses the RT-DETR v2 model unconditionally. Old configs with `"preset"` are silently ignored.
- **Table model config typed**: `table_model` on `LayoutDetectionConfig` changed from `Option<String>` to a `TableModel` enum (`tatr`, `slanet_wired`, `slanet_wireless`, `slanet_plus`, `slanet_auto`, `disabled`), defaulting to `tatr`. String values still accepted in JSON/TOML.

### Fixed

- **PDF table rendering**: populate `Table.cells` from the TATR/SLANeXT grid so tables render as proper Table nodes.
- **LaTeX extraction**: convert `\href`, `\emph`, `\textbf`, `\verb`, `\sout`, blockquotes, lists, special characters, and ligatures to markdown.
- **XLSX/XLS**: emit a `## SheetName` heading before each sheet's table.
- **OPML/IPYNB/JATS/RST/ORG headings** improved (outline headings, notebook ATX headings and cell outputs, JATS abstract/references, RST title levels and code-block hints, ORG source/example blocks).
- **ODT formula extraction**: embedded MathML formulas extracted as formula content instead of empty image placeholders.
- **PPTX slide titles** detected via OOXML placeholder type and emitted as H2 headings; bulleted/numbered lists extracted properly.
- **RTF formatting**: bold/italic/strikethrough use exact byte offsets (fixes formatting bleeding across paragraphs); hidden text suppressed; hyperlink parsing, strikethrough, and multi-row table rendering fixed.
- **HTML preprocessing**: navigation, forms, and sidebars are now stripped by default, removing page chrome from output.
- **PDF table detection**: rejects false tables where >70% of cells are single-word fragments (justified prose).
- **DocBook root element handling**: XML fragments without a root element are wrapped automatically.
- **PDF image FlateDecode fallback**: images that fail `decode_flate_to_png()` (FlateDecode, CCITT, JBIG2) are re-extracted via pdfium, producing valid PNG instead of unusable bytes.
- **Metadata standardization**: PPTX, Excel, ODT, RST, OrgMode, Typst, RTF, JATS, DOC, PPT, HTML, Email, BibTeX, and Citation metadata now map to standard `Metadata` fields (title, authors, dates, keywords, language) instead of only the `additional` map.
- **MDX/RST/LaTeX/OrgMode/BibTeX/JATS link, URL, and classification fixes** (heading and list-item links, RST hyperlinks, `\url{}` extraction, more image extensions, correct BibTeX URL classification, JATS title field).
- **Tesseract C++ exception crash**: fixed a fatal runtime error where Tesseract C++ exceptions unwound through Rust FFI; Tesseract is now compiled with `-fno-exceptions`.
- **`ExtractionConfig` rejects unknown fields**: `deny_unknown_fields` added, so typos/invalid fields (e.g. `layout_analysis`) are no longer silently ignored.
- **PPTX markdown mode** derived from `output_format` instead of hardcoded plain text, so tables/lists render correctly.
- **DOCX merged cells** (gridSpan/vMerge) repeat content across spans; added `source_path` to `ExtractedImage` for DOCX image paths.
- **Python wheel `__isoc23_strtoll` error on older Linux** — downgraded manylinux target to `manylinux_2_28` for glibc < 2.39 compatibility (Ubuntu 20.04/22.04, Debian 11/12).
- **Go macOS link failure** — added missing `-framework Foundation` to CGO LDFLAGS for ORT's CoreML provider.
- **Windows GNU ORT linking** — uses dynamic linking with pre-downloaded Microsoft ORT; documented the ONNX Runtime DLL requirement for Go, Elixir, and C/C++ on Windows.

### Changed

- **PDF text extraction**: full rewrite to `page.text().all()` + char-indexed font metadata, producing cleaner text with correct word spacing.
- **CLI format flags**: `--format` (`-f`) now supports `text`, `json`, and `toon`; `--output-format` renamed to `--content-format` (deprecated alias kept with warning).

### Removed

- **`max_upload_mb` server config field** and `KREUZBERG_MAX_UPLOAD_SIZE_MB` env var — use `max_multipart_field_bytes` / `KREUZBERG_MAX_MULTIPART_FIELD_BYTES` instead.
- **`metadata.additional` legacy insertions**: pipeline features no longer insert error/status keys — errors are in `processing_warnings`, keywords in `extracted_keywords`.

---

## [4.6.3] - 2026-03-27

### Added

- **Tower service layer** (`service` module): Composable `ExtractionService` with configurable middleware (tracing, metrics, timeout, concurrency limit) behind a new `tower-service` feature, auto-enabled by `api`/`mcp`.
- **Semantic OpenTelemetry conventions**: Formal `kreuzberg.*` attribute namespace with span attributes, metric names, and operation/stage constants for extraction, pipeline, OCR, and model inference telemetry.
- **Extraction metrics**: 11 OTel instruments covering extraction totals, durations, cache hits/misses, pipeline stages, OCR, and concurrency (feature-gated behind `otel`).

### Improved

- **Deeper instrumentation**: Pipeline post-processing stages, individual processors, OCR, and layout model inference now emit semantic spans and duration metrics.
- **API and MCP servers route extractions through the Tower service stack**, gaining unified tracing, metrics, and middleware.
- **API server hardening**: Added response compression (gzip/brotli/zstd), panic recovery, request-ID correlation, and sensitive-header redaction.

### Changed

- **Span attribute names migrated to `kreuzberg.*` namespace** (e.g. `extraction.filename` -> `kreuzberg.document.filename`).

### Fixed

- **EPUB spine handling**: Preserves manifest fallback chains, guide references, and non-linear spine items; strips navigation chrome; malformed guide references now warn instead of hard-failing.
- **DOCX images with high-quality settings not extracted**: `<a:blip>` elements with child elements are now handled.
- **OCR tables discarded on default output format**: Layout detection was skipped for the `Plain` format, dropping tables from the OCR pipeline; both paths now propagate tables.
- **Missing binding fields**: Exposed `chunker_type`, `sizing_cache_dir`, and `prepend_heading_context` across Python, TypeScript/WASM, Go, C#, and PHP.
- **Full API parity across all 10 bindings**: Added `max_archive_depth` everywhere, `acceleration`/`email` to Ruby/R, `layout` to PHP, and 7 missing fields to WASM.
- **Windows install failure for the Node package**: Replaced bash-specific prepare-script fallback with a cross-platform equivalent.

---

## [4.6.2] - 2026-03-26

### Added

- **PDF page rendering API**: New `render_pdf_page` function and `PdfPageIterator` render individual PDF pages as PNG images across all 11 bindings, with idiomatic patterns per language. Default 150 DPI, configurable per call.

### Fixed

- **Zero tables found on scanned PDFs**: Layout bboxes (640x640 model space) are now scaled to OCR render resolution before table recognition.
- **OCR elements reported `page_number: 1` for all pages**: Page numbers are now correctly stamped after OCR in the batch loop.
- **Ruby gem missing ONNX Runtime**: Added bundled ORT so OCR/layout/embeddings work out of the box.

### Improved

- **`version:sync` also syncs Go C header, version constant, and Docker tags**, preventing version drift across bindings.

---

## [4.6.1] - 2026-03-25

### Added

- **Per-file batch extraction timeouts**: New `extraction_timeout_secs` (batch default) and `timeout_secs` (per-file override), plus a `KreuzbergError::Timeout` variant with elapsed/limit fields. All bindings updated.
- **Page-level OCR overrides**: New `force_ocr_pages` (1-indexed) enables selective OCR on specific pages of mixed-quality PDFs while preserving native text on others.
- **PST extraction support**: Extract emails from Microsoft Outlook PST archives, feature-gated under `email`.
- **JSONL/NDJSON extraction**: Native `.jsonl`/`.ndjson` extraction registered as `application/x-ndjson`.

### Fixed

- **OCR elements now propagated to `ExtractionResult`** with geometry data and coordinate-bearing narrative blocks.
- **OOM crash on multi-page scanned PDFs**: PDF pages are now rendered and OCR'd in bounded batches instead of all at once.
- **OCR memory usage reduced 60-78%**: One page is rendered and encoded at a time; a 98-page scanned PDF dropped from 4.6GB to 1.9GB peak RSS. Batch size adapts to available memory on Linux/macOS.
- **PDF control-character artifacts**: Broken ToUnicode mappings that produced control characters where hyphens belong are fixed (e.g. `re\x02labelling` -> `re-labelling`).
- **Missing headings for PDFs**: Heading nodes are now inserted for structured PDFs, and markdown heading markers in fallback paragraphs create heading groups.
- **Empty tables on scanned PDFs**: Three bugs that always returned `[]` tables for scanned/image PDFs are fixed so all paths propagate tables.
- **Table recognition coordinate mismatch on scanned PDFs**: Bounding boxes are now scaled from layout-model resolution to actual OCR render resolution, fixing zero recognized tables.
- **OCR elements reported `page_number: 1` for all pages**: Correct 1-indexed page numbers are now stamped per batch page.
- **PDF layout engine crash on malformed input**: Layout-engine init failure now returns a descriptive error instead of crashing the host process via FFI.

---

## [4.6.0] - 2026-03-24

### Added

- **Recursive archive extraction**: ZIP/TAR/7Z/GZIP archives now recursively extract every processable file, each with its own `ExtractionResult`. New `ArchiveEntry` type; configurable `max_archive_depth` (default 3, 0 for legacy single-text behavior).
- **YAML/JSON section chunker**: New `ChunkerType::Yaml` splits structured files by keys with full hierarchy paths, auto-inferred for YAML/JSON.
- **Unified `DocumentStructure` DTO**: 7 new node types, 4 new annotation kinds, and a format-specific `attributes` bag on every node.
- **Unified rendering module**: `render_to_markdown()` and `render_to_plain()` walk a `DocumentStructure` tree for consistent output with inline annotations, table escaping, and nested lists.
- **DocumentStructure support for all 35 formats**: Every extractor natively produces a `DocumentStructure` when `include_document_structure` is enabled, including Office, markup, books, scientific, data, email, and image formats.
- **DocBook/JATS inline annotations**: Semantic emphasis, bold, code, links, and sub/superscript for academic documents.
- **Document-level OCR**: `OcrBackend` supports whole-file `process_document()`, up to 30% faster on multi-page documents.

### Changed

- **CSV extraction for embedding quality**: Emits `Row N: Header: Value` format when a header row is detected; the `tables` field is unchanged.
- **XML extraction for embedding quality**: Indented hierarchical output preserving the element tree with inline attributes and `xmlns:*` filtering.

### Improved

- **Zero-copy file I/O**: Automatic memory-mapping for files >1MB with SIMD-accelerated UTF-8 validation; measurable speedup for large PDFs and archives (WASM falls back to heap allocation).
- **Unified concurrency management**: Centralized thread budget for Rayon, ONNX, and PaddleOCR via configurable `ConcurrencyConfig`; PDF OCR batched to reduce memory on large documents.

### Fixed

- **Incorrect page numbers in element-based output**: Element-based output now auto-enables page extraction, fixing all elements reporting `page_number=1`.
- **MSG extraction misses compressed RTF bodies**: Added compressed-RTF fallback so `.msg` files that store the body only in compressed RTF are extracted.
- **Indexed-colour PDF images returned as raw bytes**: Palette-based PDF images now decode to valid PNG output.
- **ODT extraction robustness**: Replaced unwraps with safe fallbacks in ODT parsing.

---

## [4.5.4] - 2026-03-23

### Added

- **Document-level OCR optimization**: `OcrBackend` supports native whole-file `process_document()`, avoiding per-page PDF rasterization when the backend supports it.
- **PST (Outlook Personal Folders) extraction**: New `PstExtractor` extracts subject, sender, recipients, body, and date from every message, enabled via the `email` feature (MIME `application/vnd.ms-outlook-pst`).
- **`prepend_heading_context` chunking option**: When set with the Markdown chunker, prepends the heading hierarchy path to each chunk for self-contained RAG context. Available across all 10 bindings, CLI, and WASM.

### Fixed

- **PDF image extraction panic on mismatched buffer lengths**: Malformed PDF images are now skipped instead of panicking (regression from 4.5.0).
- **`pdf` feature failed to compile without `layout-detection`**: Feature-gated the offending `config.layout` reference.
- **Ruby binding missing `table_model` field**: Added `table_model` parsing to the layout config.
- **WASM fails to load in Supabase/Deno edge functions**: Added explicit package exports and Deno detection with clear error messaging for restricted edge runtimes.
- **Build failures on some toolchains and WASM/CI**: Pinned `zip` below 7.4 and vendored HWP text extraction to drop a transitive `zip 2.x` dependency.

---

## [4.5.3] - 2026-03-22

### Added

- **Apple iWork format support**: Native parsing of modern `.pages`, `.numbers`, and `.key` files via a new `iwork` feature.
- **SLANeXT table structure recognition models**: New `table_model` field on `LayoutDetectionConfig` selects the backend (`tatr` default, `slanet_wired`, `slanet_wireless`, `slanet_plus`, `slanet_auto`). Available across all 12 bindings and CLI (`--layout-table-model`).
- **PP-LCNet table classifier**: Automatic wired/wireless table detection for SLANeXT auto mode.
- **CLI `cache warm --all-table-models`**: Opt-in download of SLANeXT model variants (~730MB); default warm downloads only RT-DETR + TATR.

---

## [4.5.2] - 2026-03-21

### Fixed

- **PDF word splitting in extracted text**: Spurious mid-word spaces (e.g. `"s hall a b e active"`) are fixed via selective page-level respacing using character-level gap analysis.
- **Markdown underscore escaping removed**: Identifiers like `CTC_ARP_01` are no longer mangled into `CTC\_ARP\_01`.
- **Page header/footer leakage**: Running headers and copyright footers are now detected via fuzzy alphanumeric matching and stripped from the body.
- **R batch functions failed with a spurious NULL argument**: Removed the extra positional `NULL` that broke all batch operations.
- **Elixir OCR/layout/embeddings failed on Windows**: ONNX Runtime DLL is now staged where the BEAM VM loads NIFs.

### Added

- **General extraction result caching**: All file types (not just OCR) are now cached; repeat extractions with the same config return instantly.
- **Cache namespace isolation**: New `cache_namespace` (and `--cache-namespace`) enables multi-tenant cache isolation on shared filesystems, with per-namespace deletion and stats.
- **Per-request cache TTL**: New `cache_ttl_secs` overrides the global TTL per extraction (0 skips cache).
- **Bundled `eng.traineddata`**: English OCR works out of the box with zero runtime configuration (~4MB bundled).
- **Tessdata in `cache warm`/`cache manifest`**: `cache warm` downloads all ~120 tessdata_fast language files, giving full Tesseract language support without system packages; `KREUZBERG_CACHE_DIR/tessdata` is now resolved.
- **CLI `embed` command**: Generate vector embeddings from text (feature-gated on `embeddings`).
- **CLI `chunk` command**: Split text into chunks with configurable size, overlap, chunker type, and tokenizer.
- **CLI `completions` command**: Generate shell completions for bash, zsh, fish, and powershell.
- **CLI `--log-level` global flag**: Override `RUST_LOG` per invocation.
- **CLI extraction overrides**: 27 new flags including `--layout-preset`, `--acceleration`, `--extract-images`, `--target-dpi`, `--token-reduction`, `--max-concurrent`, and more.
- **CLI colored output**: Colored headers and labels, respecting `NO_COLOR`.
- **API `POST /detect`, `GET /version`, `GET /cache/manifest`, `POST /cache/warm`**: New MIME detection, version, model manifest, and eager-download endpoints.
- **MCP `get_version`, `cache_manifest`, `cache_warm`, `embed_text`, `chunk_text` tools**.
- **TATR model availability check**: Layout detection now errors when table regions are detected but the TATR model is unavailable, instead of silently degrading.

### Changed

- **CLI batch flags**: The batch command now supports all extraction override flags, matching the extract command.

### Improved

- **CLI/API/MCP input validation**: OCR backend names, chunk sizes/overlap, DPI range, layout confidence, embedding preset names, and chunk bounds are now validated.
- **Chunk overlap auto-clamping**: Overlap is clamped to `size/4` when `--chunk-size` is smaller than the default overlap, instead of erroring.

---

## [4.5.1] - 2026-03-20

- **Java batch extraction failed with memory access errors**: Fixed swapped `count`/`results` fields in the `CBatchResult` Panama FFM layout.
- **Go bindings read fields at wrong offsets**: Synced the Go C header struct field order with the Rust `#[repr(C)]` layout, fixing corrupted `pages_json`.
- **FFI failed to compile without `layout-detection`**: Feature-gated `LayoutDetectionConfig` and the layout setter.
- **Python wheel builds failed on Linux aarch64**: The OpenSSL path is now detected via `uname -m` instead of being hardcoded to x86_64.
- **R batch functions errored**: Added the missing `file_configs` parameter that broke all batch operations.
- **R package failed to load with ONNX Runtime**: Now links against ORT when `ORT_LIB_LOCATION` is set, fixing `undefined symbol: OrtGetApiBase`.

---

## [4.5.0] - 2026-03-20

### Added

- **ONNX-based document layout detection**: New `layout` config field runs RT-DETR v2 (17 element classes) with `"fast"`/`"accurate"` presets and auto-downloaded models, across all bindings.
- **SLANet table structure recognition**: Detected table regions produce markdown tables with colspan/rowspan support, now on all pages.
- **Layout-enhanced heading detection**: Layout SectionHeader/Title regions guide heading detection and can override font-size classification.
- **Multi-backend OCR pipeline**: New `OcrPipelineConfig` enables quality-based fallback across OCR backends (e.g. Tesseract then PaddleOCR).
- **OCR quality thresholds**: New `OcrQualityThresholds` with 16 tunable parameters for output quality assessment and fallback.
- **OCR auto-rotate**: New `OcrConfig.auto_rotate` (default false) detects 0/90/180/270-degree page rotations.
- **PaddleOCR v2 model tier system**: New `model_tier` field with `"mobile"` (default, fast) and `"server"` (highest accuracy), unified multilingual models, across all bindings.
- **`AccelerationConfig` for GPU/execution-provider control**: Fine-grained control over ONNX providers (CPU, CoreML, CUDA, TensorRT), typed across all bindings.
- **`ConcurrencyConfig` for thread limiting**: New `max_threads` caps Rayon, ONNX intra-op threads, and batch concurrency, typed across all bindings.
- **`EmailConfig` for MSG fallback codepage**: Configurable fallback codepage for MSG files lacking one (default windows-1252), typed across all bindings.
- **Per-file extraction configuration (`FileExtractionConfig`)**: Each file in a batch can specify its own OCR, chunking, and output settings, via CLI `--file-configs` and MCP `file_configs`.
- **Opt-in single-column pseudo tables**: New `allow_single_column_tables` on `PdfConfig` (default false) emits glossaries and itemized lists as tables.
- **CLI `cache warm` and `cache manifest` commands**: Eagerly download all OCR/layout models, or output a JSON manifest with SHA256 checksums and source URLs.
- **ChunkSizing configuration**: `sizing_type`, `sizing_model`, and `sizing_cache_dir` exposed across all bindings.
- **Chunk heading context**: New `HeadingContext` in `ChunkMetadata` with heading level and text.
- **`ModelManifestEntry` type with `manifest()` / `ensure_all_models()`**: Public API for querying and eagerly downloading model caches.

### Changed

- **Layout preset default changed from `"fast"` to `"accurate"`**: The `Fast` variant is removed; the `"fast"` string is still accepted.
- **PaddleOCR default model tier changed from `"server"` to `"mobile"`**: Mobile is 3-5x faster with equivalent quality on standard documents; server remains available.
- **PaddleOCR v2 models**: All models updated to v2 generation with unified multilingual recognition; V1 models remain for older versions.
- **Batch API unification**: `_with_configs` batch functions removed; per-file `FileExtractionConfig` is now an optional parameter on the unified batch functions.
- **Embedding `embed()` now takes `&self`**: Enables parallel embedding generation.
- **`padding` field in `PaddleOcrConfig`**: Now exposed across Python, TypeScript, Ruby, and Go.
- **Language-agnostic section pattern recognition**: Headings ending with a period are allowed when matching structural patterns, improving heading detection for legal/academic/multilingual documents.
- **Strong typing across bindings**: Replaced weak `Dictionary`/`Map`/`array` config types with strongly typed classes in C#, Java, and PHP, and added missing config types to other bindings.

### Removed

- **`fastembed` dependency**: Replaced by a vendored embedding engine using ONNX Runtime directly.
- **`EmbeddingModelType::FastEmbed` variant**: Use `Preset` or `Custom` instead.

### Fixed

- **C# library completely broken in 4.4.6**: Fixed the `CExtractionResult` struct layout mismatch that caused deserialization failures and overflow exceptions.
- **PDF `force_ocr` without explicit OCR config silently ignored**: Now unconditionally triggers OCR with default settings.
- **PDF image extraction returned raw compressed data**: Extracted images are now decoded and re-encoded as PNG/JPEG.
- **Node.js `extractFileInWorker` dropped mime_type**: MIME type is now forwarded to extraction instead of injected into the PDF password config.
- **DOCX/XML parser compilation failure**: Resolved type ambiguity introduced by `zip` 8.2.0.
- **Python type stubs missing from sdist**: `py.typed` and `.pyi` files are now included in wheel and sdist.
- **PDF broken CMap word spacing**: Geometric validation vetoes false word boundaries (e.g. `"co mputer"` -> `"computer"`).
- **PDF structure-tree headings rejected**: H1-H6 tags are now trusted as author intent instead of being rejected by font-size validation.
- **Slow PDF structure-tree extraction**: Text and style maps are built in a single pass, eliminating multi-second extraction on complex pages.
- **OCR Picture regions suppressed text**: Embedded text in Picture regions is preserved as paragraphs instead of dropped.
- **Unstable reading order**: Spatial sorts use discrete row buckets, ensuring correct and stable ordering.
- **Page furniture over-stripping**: Added guards to prevent removal of legitimate content.
- **`KREUZBERG_CACHE_DIR` not respected by all caches**: Embeddings, OCR, and extraction caches now honor the variable.
- **MSG ANSI string decoding**: MSG files now decode ANSI properties using the declared code page instead of lossy UTF-8.
- **SLANet-Plus inference failures on macOS CoreML**: Re-exported the ONNX model with a shape fix, resolving silent table-extraction failures.
- **TATR model crash in batch processing**: Model unavailability in parallel closures now falls back gracefully instead of crashing FFI callers.
- **Docker musl builds failed**: Alpine/musl images now link the system ONNX Runtime; all features work in musl CLI images.
- **C#/Java FFI batch functions rejected NULL**: They now accept NULL for per-file config JSON.

### Known Issues

- **PHP PIE Windows package temporarily unavailable**: Disabled due to a transitive dependency conflict on the Windows target; Linux and macOS are unaffected.
- **WASM: no layout detection, acceleration, or email config**: ONNX Runtime is unavailable on WASM, so RT-DETR layout detection, hardware acceleration, and concurrency config are unavailable; Tesseract WASM OCR and embeddings are supported.

---

## [4.4.6]

### Added

- **dBASE (.dbf) format support**: Extract table data from dBASE files as markdown tables with field type support.
- **Hangul Word Processor (.hwp/.hwpx) support**: Extract text from HWP 5.0 documents.
- **Office template/macro format variants**: Added `.docm`, `.dotx`, `.dotm`, `.dot`, `.potx`, `.potm`, `.pot`, `.xltx`, and `.xlt` support.

### Fixed

- **DOCX image placeholders missing with `extract_images=True`**: The default plain-text path stripped image references; image extraction now forces markdown output so placeholders appear.

### Changed

- **Format count updated to 91+** (from 75+) across docs and manifests.

## [4.4.5]

### Fixed

- **PDF markdown garbled positioned text**: Line breaks in positioned/tabular text (CVs, addresses, tables) are preserved by splitting short lines into separate paragraphs when few lines reach the right margin.
- **Node worker pool password bug**: `extractFileInWorker` passed `password` as `mime_type`, so passwords were never applied; it now injects into `config.pdf_options.passwords`.
- **WASM document structure returned empty results**: JS camelCase config keys are now transformed to snake_case for deserialization.
- **PHP 8.5 array coercion on macOS**: A wrapper transparently converts coerced array return values back to objects.
- **PHP 8.5 support**: Upgraded ext-php-rs to 0.15.6.

### Added

- **CLI `--pdf-password` flag**: New option on `extract`/`batch` for encrypted PDFs (repeatable).
- **MCP `pdf_password` parameter**: Added to `extract_file`, `extract_bytes`, and `batch_extract_files`.
- **API `pdf_password` multipart field**: The extract endpoint now accepts encrypted-PDF passwords.

## [4.4.4]

- Maintenance release (internal changes only).

## [4.4.3]

### Added

- **PDF image placeholder toggle**: New `inject_placeholders` on `ImageExtractionConfig` (default true); set false to extract image data without injecting `![image](...)` references.

### Fixed

- **Token reduction not applied**: Config was accepted but never executed; the pipeline now applies `reduce_tokens()` when configured.
- **Nested HTML table extraction**: Nested tables now extract correctly with proper cell data and markdown rendering.
- **hOCR plain text output**: hOCR now produces plain text when requested instead of falling back to Markdown.
- **PDF garbled positioned/tabular text**: Spaces are now inserted when the X-position gap between characters exceeds `0.8 × avg_font_size`.
- **Chunk page metadata drift with overlap**: Byte offsets are computed via pointer arithmetic, fixing incorrect page numbers when overlap is enabled.
- **Node.js metadata casing**: Standardized `Metadata`/`EmailMetadata` fields to camelCase (e.g. `pageCount`, `creationDate`) and corrected `authors`/`keywords` pluralization.

## [4.4.2]

### Fixed

- **WASM OCR blocked the event loop**: `ocrRecognize()` ran synchronously on the main thread; it now runs in a worker thread, keeping the runtime responsive and letting timeouts fire.
- **JPEG 2000 / JBIG2 OCR decode failure**: JP2/JPX/JPM/MJ2 and JBIG2 images failed with "format could not be determined"; a shared loader now detects them by magic bytes and uses the correct decoders across all OCR backends, including `ocr-wasm`.
- **WASM PDF returned empty content**: `initWasm()` now awaits PDFium initialization, fixing a race where extraction started before PDFium was ready.

### Added

- **OMML-to-LaTeX math conversion for DOCX**: DOCX equations are now converted to LaTeX (fractions, radicals, n-ary operators, matrices, etc.) with `$$...$$`/`$...$` in markdown and raw LaTeX in plain text, instead of concatenated Unicode.
- **Plain text output paths for all extractors**: DOCX, PPTX, ODT, FB2, DocBook, RTF, and Jupyter now produce clean plain text (no `#`, `**`, `|`, image placeholders) when plain/structured output is requested, instead of always emitting markdown.

### Changed

- **CLI now built with the `full` feature set** (the `cli` feature group is removed), ensuring the CLI supports all formats including archives (7z, tar, gz, zip).

### Fixed

- **Alpine/musl CLI Docker image "Dynamic loading not supported"**: The CLI binary is now dynamically linked against musl libc, enabling runtime loading for PDF processing.
- **R package Windows installation**: Improved Python detection (`py` launcher, `RETICULATE_PYTHON`) and graceful handling of symlink extraction errors.
- **PHP 8.5 precompiled binaries**: Added PHP 8.5 support alongside PHP 8.4.
- **OCR DPI normalization**: Images are normalized to the target DPI before Tesseract, eliminating the resolution-estimate warning and improving accuracy on non-standard DPI.
- **HTML metadata missing with Plain output**: HTML headers, links, images, and structured data are now collected even for the default plain output.
- **PPTX text run spacing**: Adjacent runs are joined with smart spacing ("HelloWorld" -> "Hello World").
- **CSV Shift-JIS/cp932 detection**: Shift-JIS CSVs are decoded correctly instead of producing mojibake, with fallback encoding detection.
- **EML multipart body extraction**: All text/html parts and nested `message/rfc822` parts are now extracted, not just index 0.
- **EPUB media tag leakage**: `<video>`, `<audio>`, `<iframe>`, etc. no longer leak into text; `<br>`/`<hr>` become newlines.
- **FB2 poem extraction**: `<poem>`, `<stanza>`, and `<v>` verse elements are now extracted instead of dropped; added sub/superscript, strikethrough, and footnotes.
- **ODT StarMath-to-Unicode conversion**: ODT formulas are converted to Unicode instead of raw StarMath syntax.
- **BibTeX output format**: Now uses standard `@type{key, field = {value}}` conventions.
- **LaTeX display math**: `\[...\]` environments are converted to `$...$`.
- **RST directive preservation**: Field lists, directive markers, and `.. code-block::` directives are preserved.
- **Typst extraction improvements**: Layout directives stripped, headings as plain text, column-aware tables, links as display text.
- **DOCX field codes refined**: Field results (visible text like "Figure 1:", page numbers) are preserved while field instructions are skipped.
- **DOCX drawing alt text in plain text**: Image alt text from `wp:docPr` is now emitted instead of skipped.
- **DOCX XML entity decoding**: Attribute values like `&#xA;` are now correctly unescaped.

---

## [4.4.1]

### Added

- **OCR table inlining into markdown content**: With Markdown output, OCR-detected pipe tables are inlined into `result.content` at their correct vertical positions instead of only appearing in `result.tables`.
- **OCR table bounding boxes**: OCR-detected tables now include pixel-level bounding boxes, propagated through all bindings as `Table.bounding_box`.

### Fixed

- **MSG recipients missing email addresses**: The extractor now reads recipient substorages for full `"Name" <email>` output with correct To/CC/BCC separation.
- **MSG date missing or incorrect**: Date is now read from `PR_CLIENT_SUBMIT_TIME`, with fallback to transport headers.
- **EML date mangled for ISO 8601 formats**: The raw `Date:` header text is preserved instead of being garbled.
- **EML/MSG attachments line polluted text output**: The `Attachments: ...` line is removed from text output; names remain in metadata.
- **HTML script/style tags leaked in email fallback**: Multiline `<script>`/`<style>` content is now stripped (dotall matching).
- **SVG CDATA leaked JavaScript/CSS**: `<script>`/`<style>` CDATA no longer appears in SVG text output.
- **RTF metadata noise in text**: Destination groups (font tables, color tables, stylesheets, info) are now skipped, removing ~17KB of internal noise.
- **RTF `\u` control words mishandled**: Formatting commands like `\ul` are no longer misread as Unicode escapes.
- **RTF paragraph breaks collapsed to spaces**: `\par` now emits newlines, and whitespace normalization preserves paragraph structure.

---

## [4.4.0]

### Added

- **R language bindings**: New kreuzberg R package via extendr with the full extraction API (sync/async, batch, bytes), typed errors, S3 result class, config discovery, and OCR/chunking configuration.
- **PHP async extraction**: Non-blocking `extractFileAsync()`, `extractBytesAsync()`, and batch variants via a Tokio thread pool, with Amp and ReactPHP bridges.
- **WASM native OCR** (`ocr-wasm`): Tesseract compiled into the WASM binary, enabling OCR in Browser/Node/Deno/Bun without browser-specific APIs; 43 languages via CDN tessdata.
- **WASM Node.js/Deno PDFium support**: PDFium now loads from the filesystem, configurable via `KREUZBERG_PDFIUM_PATH`.
- **WASM full-feature build**: OCR, Excel, and archive extraction are enabled by default in the WASM package.
- **WASM Excel extraction** (`excel-wasm`): Calamine-based spreadsheet extraction without a Tokio runtime.
- **WASM archive extraction**: ZIP, TAR, 7z, and GZIP extraction now available in WASM.
- **WASM PDF annotations**: PDF annotations are exposed via the `annotations` field on `ExtractionResult`.
- **C FFI distribution**: Official `libkreuzberg` shared library with cbindgen header, cmake/pkg-config packaging, and prebuilt binaries for Linux x86_64/aarch64, macOS arm64, and Windows x86_64.
- **Go FFI bindings**: New Go package (`packages/go/v4`) consuming the C FFI library, with prebuilt binaries for all four platforms.
- **R distribution via r-universe**: Switched from CRAN to r-universe for faster releases and easier native compilation.

### Fixed

- **DOCX equations not extracted**: OMML math (`<m:oMath>` etc.) was silently dropped; math runs are now extracted as text.
- **DOCX line breaks ignored**: `<w:br/>` now inserts whitespace instead of merging adjacent text.
- **PPTX/PPSX table content lost**: Tables now render as markdown pipe tables with proper cell separation instead of an unreadable blob.
- **PPTX/PPSX/PPTM and DOCX image markers polluted text**: Image references now use a clean `![image]()` / `![alt](image)` form instead of injecting numeric tokens.
- **EPUB double-lossy conversion**: XHTML is now traversed directly instead of through an XHTML->markdown->plain-text pipeline that lost content.
- **Excel float formatting dropped numeric precision**: Whole-number floats format as `"1"` instead of `"1.0"`.
- **HTML metadata polluted content**: YAML frontmatter is no longer prepended to content when metadata is already returned as a struct.
- **Markdown extractor lost tokens through AST reconstruction**: Now returns raw text content directly while still parsing the AST for tables and images.
- **SVG text extraction included element prefixes**: SVG extraction now targets only text-bearing elements without `element_name:` prefixes.
- **WASM OCR not working** (`enableOcr()` regression): `enableOcr()` now registers the backend in the Rust-side registry so OCR works via `extractBytes`/`extractFile`.
- **WASM tessdata CDN URL returned 404**: Updated to the official `tesseract-ocr/tessdata_fast` repository.
- **XML UTF-16 parsing failed on odd byte counts**: The decoder now truncates to the nearest even byte boundary instead of rejecting the file.
- **R bindings crashed on strings with embedded NUL bytes**: NUL bytes are now stripped before passing strings to R.
- **R bindings `%||%` operator incompatible with R < 4.4**: Added a package-local polyfill.
- **API returned HTTP 500 for unsupported formats**: The `/extract` endpoint now falls back to extension-based MIME detection and maps `UnsupportedFormat` to HTTP 400.
- **PDF markdown missing headings/bold for flat structure trees**: PDFs tagging everything as `<P>` (e.g. InDesign) now get headings via font-size clustering, and bold detection recognizes "Bold" in font names.
- **PaddleOCR backend not found via `backend="paddleocr"`**: The registry now resolves the `"paddleocr"` alias to the canonical `"paddle-ocr"` name.
- **WASM metadata serialization dropped fields**: Switched to `serde_json` + `JSON.parse()` so `format_type` and format-specific metadata are preserved.
- **WASM config deserialization**: camelCase config keys are converted to snake_case before crossing the WASM boundary.
- **WASM PDFium module loading**: The build now copies the real PDFium ESM module instead of a stub, with a Deno compatibility fix.
- **Email header extraction lost display names**: From/To/CC/BCC now use `"Display Name" <email>` format when available.
- **Email date header normalized to RFC 3339**: The raw `Date` header value is now preserved, falling back to RFC 3339 only when unavailable.

### Removed

- **`polars` dependency**: Removed unused crate and dead code; Excel extraction uses `calamine` directly.

---

## [4.3.8]

### Added

- **MDX format support** (`mdx` feature): extract text from `.mdx` files, stripping JSX/import/export syntax while preserving markdown, frontmatter, tables, and code fences.
- **List supported formats API**: query all supported extensions and MIME types via `list_supported_formats()`, the `GET /formats` REST endpoint, the `list_formats` MCP tool, or `kreuzberg formats`.

### Fixed

- **PDF ligature corruption in CM/Type1 fonts**: repairs broken-CMap text like `di!erent` → `different`, `#nancial` → `financial` in LaTeX-generated PDFs.
- **PDF dehyphenation across line boundaries**: rejoins words broken across line breaks (e.g. `soft ware` → `software`), including implicit breaks where the hyphen was stripped.
- **PDF page markers missing in Markdown and OCR output**: `insert_page_markers` / `marker_format` were dropped for Markdown and OCR output since 4.3.5; markers now appear (Djot inherits them).
- **PDF Djot/HTML output quality parity**: Djot and HTML now use the same structural pipeline as Markdown (headings, tables, bold/italic, dehyphenation) instead of falling back to plain paragraphs.
- **PDF sidebar text pollution**: widened the margin band so rotated sidebar text (e.g. arXiv identifiers) no longer leaks into extracted content.
- **Node.js PDF config dropped**: `extractAnnotations`, `hierarchy`, `topMarginFraction`, and `bottomMarginFraction` were silently discarded, making PDF annotation extraction always return `undefined`.

---

## [4.3.7]

### Added

- NFC unicode normalization applied to all extraction outputs for consistent composed characters across backends (gated behind `quality` feature).
- Configurable PDF page margin fractions (`top_margin_fraction`, `bottom_margin_fraction`) in `PdfConfig`.
- PDF annotation extraction via `extract_annotations` in `PdfConfig` and a new `annotations` field on `ExtractionResult` (all bindings), with a `PdfAnnotation` type covering Text, Highlight, Link, Stamp, Underline, StrikeOut, and Other.

### Fixed

- **PDF markdown extraction quality at parity with docling** (91.0% vs 91.4% avg F1, 10-50x faster): switched to per-character extraction that correctly handles font matrices, CMap lookups, and positioning, with adaptive line-break detection.
- **PDF markdown no longer drops all content on broken font metrics**: falls back to unfiltered segments when the min-font-size filter would remove everything.
- **PDF markdown no longer drops all content on edge-case margins**: skips margin filtering for a page when it would remove all text.
- **PDF multi-column extraction** improved (69.9% → 90.7% F1 on Federal Register-style layouts) via correct reading order.
- PDF table detection now requires ≥3 aligned columns and rejects low-quality tables, eliminating false positives from two-column text layouts.
- PDF markdown no longer drops titles/authors when pdfium returns zero-value baseline coordinates (some LaTeX PDFs).
- PaddleOCR backend validation now checks the plugin registry, preventing false "backend not registered" errors when the plugin is available.
- WASM bindings now export `detectMimeFromBytes` and `getExtensionsForMime`.
- XLSX extraction with `output_format="markdown"` now produces markdown tables instead of plain text.
- MCP no-parameter tools (`cache_stats`, `cache_clear`) now emit a valid object `inputSchema`, fixing MCP clients that validate schema type.
- Python `get_valid_ocr_backends()` now includes `paddleocr`, matching other bindings.
- PHP `PdfConfig` now includes `extractAnnotations`, `topMarginFraction`, and `bottomMarginFraction`, restoring parity with the Rust core.

---

## [4.3.6]

### Added

- **Automatic OCR fallback on garbled fonts**: pages where >30% of characters have broken unicode mappings (tofu) now trigger OCR automatically.
- **Extended list detection**: recognizes en/em dashes, single-letter prefixes (`a.`, `B)`), and roman numerals.
- Soft-hyphen and stray control-character normalization in PDF text for cleaner word rejoining.

### Fixed

- **PDF list detection panic on multi-byte UTF-8 with CRLF line endings**: fixed crash from byte-vs-char newline handling.
- **PaddleOCR backend not respected in Python bindings**: `paddleocr`/`paddle-ocr` backends now correctly hand off to the Rust core instead of being silently skipped.
- **Ruby gem missing `sorbet-runtime` at runtime**: promoted from a dev to a runtime dependency, fixing load failures.
- **DOCX extractor panic on multi-byte UTF-8 page boundaries**: fixed crash from byte-index slicing during page-break insertion.
- **Node.js `djot_content` always undefined**: the field is now mapped from Rust results.
- **Tesseract OCR word-level elements dropped**: fixed off-by-one TSV level mapping and a hardcoded `None` in the image OCR path so word-level elements are returned.
- **OCR cache deserialization failure**: older cached OCR data missing the newer `detection` field now deserializes again.
- **PDF table detection false positives**: precision improved from 50% to 100% by validating tables on both the pdfium and OCR paths.
- PDF line grouping and paragraph-break detection made robust to subscripts/superscripts, fixing mis-grouped lines and spurious paragraph breaks.

---

## [4.3.5]

### Added

- **PDF markdown output format**: native PDF extraction now supports `output_format: Markdown` with headings, paragraphs, inline bold/italic, and list detection instead of flat text.
- **Multi-column PDF layout detection**: 2+ column layouts (academic papers, magazines) are processed per column, preventing text interleaving.
- **musl/Alpine Linux prebuilt native libraries for Elixir, Java, and C#**: instant install on Alpine and musl distros without compiling from source.
- **Pre-compiled platform-specific Ruby gems** (`x86_64-linux`, `aarch64-linux`, `arm64-darwin`, `x64-mingw-ucrt`): eliminates the 30+ minute compile on `gem install kreuzberg` (source gem still available as fallback).
- **`bounding_box` field on `Table` and `ExtractedImage`** (all bindings): spatial positioning for tables and images, computed from character positions.
- **Inline tables and image placeholders in PDF markdown**: tables and image references (with OCR text as blockquotes) are now placed at their correct vertical position instead of appended at the end.
- **Typed `ExtractionResult` fields**: `extracted_keywords` (with algorithm/score/position) and `quality_score` are now first-class typed fields instead of untyped `metadata.additional` entries (all bindings).
- **`ProcessingWarning` on `ExtractionResult.processing_warnings`**: surfaces non-fatal warnings during extraction.
- **Typed `Metadata` fields**: `category`, `tags`, `document_version`, `abstract_text`, and `output_format`; `output_format` is now populated for every output format, not just structured.

### Fixed

- **PaddleOCR wrong-character recognition**: fixed an off-by-one dictionary offset that mismapped recognized characters.
- **PaddleOCR angle classifier misfiring on short text**: `use_angle_cls` now defaults to `false` (re-enable via `PaddleOcrConfig::with_angle_cls(true)` for rotated docs).
- **PaddleOCR padding included table gridlines**: default detection padding reduced 50px → 10px and made configurable via `with_padding()`.
- **PaddleOCR recognition height mismatch**: fixed `[batch, 3, 48, width]` input shape that caused ONNX Runtime dimension errors on all platforms.
- **Faster PDF markdown**: eliminated a redundant document parse, saving 25-40ms per PDF.
- **PDF crashes on corrupt coordinates**: replaced panics in text clustering/merging with graceful handling of NaN/Inf coordinates.
- **PDF heading and list false positives**: decorative extreme font sizes no longer become headings; long paragraphs starting with "1." or "-" are no longer misclassified as lists.
- **PHP binding: `bounding_box` always null** on `Table` and `ExtractedImage`.
- **Go binding: embedding config dropped**: added the missing `Embedding` field to `ChunkingConfig`, so embedding-enabled extractions actually run embeddings.
- **Go binding: `extracted_keywords`, `quality_score`, `processing_warnings` always nil**: the FFI header and decoder now include these fields.
- **`extraction_duration_ms` missing from Go, Java, PHP, C# bindings**: added.
- Deprecated `Metadata.additional` map now marked obsolete/deprecated in C#, Java, and Go.

### Changed

- **PaddleOCR recognition models upgraded to PP-OCRv5** for all 11 script families (arabic, devanagari, tamil, telugu were on v3), improving accuracy.
- **PDFium upgraded to chromium/7678** (from 7578); C API remains backward-compatible with existing bindings.

---

## [4.3.4] - 2026-02-16

### Fixed

- **Node.js keyword extraction fields missing**: `extractedKeywords`, `qualityScore`, and `processingWarnings` were silently dropped from results; also renamed the mismatched `keywords` property to `extractedKeywords`.
- **CLI installer resolving benchmark tags as latest release**: `install.sh` could install a benchmark run instead of a real version; it now filters for `v`-prefixed release tags.

---

## [4.3.3] - 2026-02-14

### Added

- **Jupyter notebook image extraction**: embedded base64 images (PNG, JPEG, GIF, WebP) are decoded into `ExtractedImage` structs and OCR'd when configured; SVGs are kept as text.
- **Markdown data-URI image extraction**: `data:image/...;base64,...` images are decoded and OCR'd when configured; HTTP(S) image URLs are preserved as text markers (no network access).
- **PaddleOCR 80+ language support via 11 script families** (english, chinese, latin, korean, cyrillic, thai, greek, arabic, devanagari, tamil, telugu), downloaded on demand with SHA256 verification, with an engine pool for concurrent multi-language OCR.
- **Backend-agnostic `--ocr-language` CLI flag**: works across tesseract, paddle-ocr, and easyocr.
- **Full DOCX extraction pipeline**: hierarchical `DocumentStructure`, per-page splitting, OCR on embedded images, typed metadata fields, style-based heading detection, headers/footers and footnote references, markdown formatting (bold/italic/underline/strikethrough/links), and merged-cell table metadata.

### Fixed

- **LaTeX zero-arg command handling**: 35 zero-argument commands (`\par`, `\noindent`, etc.) no longer swallow the following `{...}` group, preventing silent text loss.
- **Structured-data field mislabeling**: field-name matching is now exact, so fields like "width" or "valid" are no longer treated as ID text fields.

---

## [4.3.2] - 2026-02-13

### Changed

- **PHP now requires 8.4+**: the PHP binding requires PHP 8.4+ (needed for PHPUnit 13.0).

### Fixed

- **Elixir v4.3.1 publish failure**: fixed a macOS ARM64 build timeout that had prevented the Elixir package from reaching Hex.pm.

---

## [4.3.1] - 2026-02-12

### Fixed

- **Elixir 4.3.0 install failures**: fixed a Hex package checksum mismatch (the release shipped with outdated 4.2.10 checksums) across all 8 precompiled NIF binaries.
- **WASM build failures**: fixed missing WebAssembly support in transitive dependencies (ahash, lopdf, rand_core).

---

## [4.3.0] - 2026-02-11

### Added

- **Blank page detection**: `is_blank` on `PageInfo` and `PageContent` flags near-empty pages (all bindings).
- **PaddleOCR backend** (`paddle-ocr` feature, ONNX Runtime): PP-OCRv4 models for English, Chinese, Japanese, Korean, German, and French with automatic model download/caching and superior CJK recognition; available in all bindings and via `kreuzberg --ocr-backend paddle-ocr`.
- **Structured OCR element output**: `OcrElement` data with bounding geometry, per-element confidence, rotation, and hierarchical levels (word/line/block/page) from both PaddleOCR and Tesseract.
- **Document structure output**: `include_document_structure` option produces structured `DocumentStructure` (all bindings).
- **Native DOC/PPT extraction** via OLE/CFB parsing — legacy Office formats no longer require external tools (LibreOffice removed).
- **musl/Alpine Linux support**: prebuilt CLI binaries, Python musllinux wheels, and Node.js native bindings for musl targets, fixing the glibc 2.38+ requirement on older distros (e.g. Ubuntu 22.04).

### Fixed

- **MSG (Outlook) extraction hang on large attachments**: fixed indefinite hang by switching to direct CFB parsing; also handles truncated sector tables from some Outlook versions.
- **Rotated PDF text extraction**: PDFs with 90°/270° page rotation previously returned empty content; now extract correctly.
- **CSV and Excel extraction quality**: fixed near-empty/garbled output; both now emit clean delimited text.
- **XML extraction quality**: better handling of namespaced elements, CDATA, and mixed content.
- **WASM table extraction**: tables were silently dropped in Deno/Cloudflare Workers due to a field-name mismatch.
- **DOCX formatting**: extraction now produces formatted markdown (bold/italic/underline/strikethrough/links), correct heading hierarchy, bullet/numbered/nested lists, and inline pipe tables (with cell formatting preserved) instead of plain text.
- **Typst tables**: fixed a parser bug that consumed all remaining content after a `#table()` call.
- **PaddleOCR model loading**: fixed a `ShapeInferenceError` on ONNX Runtime 1.23.x and an incorrect detection model filename.
- **Python: `OcrConfig` ignoring `paddle_ocr_config`/`element_config`**, and keyword/additional metadata being dropped from results.
- **TypeScript/Node.js**: PaddleOCR/element config dropped, and `ocr_elements` missing from results.
- **Ruby**: gem build failures (missing vendored crate), PaddleOCR/element config not parsed, and `ocr_elements` missing from results.
- **Go**: `PdfMetadata` deserialization failing when keyword extraction returns object arrays.
- **C#**: keyword extraction data was inaccessible (excluded from serialization).
- **PHP**: `document`/`elements`/`ocrElements` inaccessible, `include_document_structure` silently ignored, missing OCR-backend management functions, and a `page_count` key mismatch.
- **Elixir**: config parser dropped several options (`include_document_structure`, `output_format`, `security_limits`, etc.) and lacked document-extractor management functions.

### Removed

- **LibreOffice is no longer required**: legacy `.doc`/`.ppt` now extract natively, shrinking the full Docker image by ~500-800MB. Users on Kreuzberg <4.3 still need LibreOffice for these formats.
- **Guten OCR backend** references removed; `KREUZBERG_DEBUG_GUTEN` env var renamed to `KREUZBERG_DEBUG_OCR`.

---

## [4.2.15] - 2026-02-08

### Added

- **Agent Skill for document extraction**: Added a `SKILL.md` following the [Agent Skills](https://agentskills.io) open standard, with Python/Node.js/Rust/CLI instructions and reference files; works with Claude Code, Codex, Gemini CLI, Cursor, VS Code, and other compatible tools.
- Added `.docbook` (`application/docbook+xml`) and `.jats` (`application/x-jats+xml`) file extension mappings.

### Fixed

- **ODT lists and sections**: Documents with bulleted/numbered lists or sections no longer return empty content.
- **UTF-16 EML**: EML files encoded in UTF-16 (LE/BE, with or without BOM) no longer return empty content.
- **Email attachments**: Fixed a metadata serialization bug that overwrote the structured attachments array, causing deserialization failures in Go, C#, and other typed bindings.
- **WASM office documents**: DOCX, PPTX, and ODT now extract correctly in WASM builds.
- **WASM PDF in non-browser runtimes**: PDF extraction now works in Node.js, Bun, and Deno, not just the browser.
- **Elixir page metadata**: Encoding page structure metadata (`PageBoundary`, `PageInfo`, `PageStructure`) to JSON no longer fails.
- **Pre-built CLI `mcp` command**: Standalone CLI binaries now include the `mcp` feature, so `kreuzberg mcp` is available.
- **PDF error handling**: Reverted a v4.2.14 regression; corrupted PDFs again return `PdfError::InvalidPdf` and password-protected PDFs return `PdfError::PasswordRequired` instead of silently returning empty results.

### Changed

- Added `security_limits` field to all 9 language bindings for parity with the Rust core `ExtractionConfig`.

---

## [4.2.14] - 2026-02-07

### Fixed

- **Excel file-path extraction**: `.xla` (legacy add-in) and `.xlsb` (binary spreadsheet) files now fall back gracefully when extracted by file path, not just from bytes.

---

## [4.2.13] - 2026-02-07

### Added

- **WASM office formats**: DOCX, PPTX, RTF, reStructuredText, Org-mode, FictionBook, Typst, BibTeX, and Markdown are now available in the browser/WASM build.
- **Citation extraction**: Added structured extraction for RIS (`.ris`), PubMed/MEDLINE (`.nbib`), and EndNote XML (`.enw`), with authors, DOI, year, keywords, and abstract.
- **JPEG 2000 OCR**: JP2 container and J2K codestream images are now decoded for OCR via a pure-Rust decoder, plus JP2 metadata parsing.
- **JBIG2 images**: Added JBIG2 bi-level image decoding for OCR (common in scanned PDFs), with `image/x-jbig2` and `.jbig2`/`.jb2` mappings.
- **Gzip archives**: Added extraction of text from gzip-compressed files (`.gz`), with decompression size limits to guard against gzip bombs.
- **JATS and DocBook**: Registered the JATS and DocBook extractors in the default registry so these formats now extract.
- Added many MIME type and extension mappings for broader format recognition (FictionBook, BibTeX, DocBook, PubMed, Djot, Typst, EPUB, RTF, Org, and more).
- **Configurable archive security**: Added a `security_limits` field to `ExtractionConfig`; ZIP, TAR, 7z, and GZIP extractors now enforce configurable limits for archive size, file count, compression ratio, and content size, and ZIP archives are validated for zip bombs before extraction.

### Fixed

- **Typst / Djot recognition**: `.typ` files are now recognized as Typst and `.djot` files as Djot.
- **Gzip MIME**: `application/gzip` is no longer rejected by MIME validation.
- **Case-sensitive MIME validation**: Valid MIME types with different casing (e.g. `macroEnabled`) are no longer rejected.
- **JPEG 2000 images**: `.jp2` images are now handled by the image extractor (`image/jp2`, `image/jpx`, `image/jpm`, `image/mj2`).
- **YAML**: YAML files are no longer rejected; all four YAML MIME variants including the standard `application/yaml` are accepted.
- **CLI config**: The `--config` flag now accepts both `.yml` and `.yaml`.
- **`.tgz` archives**: `.tgz` files are now correctly handled as gzip-compressed TAR rather than raw TAR.
- **Excel exotic formats**: `.xlam`, `.xla`, and `.xlsb` files lacking standard workbook data now return an empty workbook instead of erroring.

---

## [4.2.12] - 2026-02-06

### Fixed

- **DOCX lists**: Fixed list items missing whitespace between text runs, which merged words together.

---

## [4.2.11] - 2026-02-06

### Fixed

- **Python**: CLI binary is no longer missing from platform wheels.
- **OCR fallback**: Fixed scanned multi-page PDFs skipping OCR; page count is now passed correctly, and each page is evaluated independently so any scanned page triggers OCR for the document.

---

## [4.2.10] - 2026-02-05

### Fixed

- **MIME detection**: DOCX/XLSX/PPTX files are no longer misdetected as `application/zip` when detecting from bytes.
- **Java**: Format-specific metadata (e.g. `sheet_count`, `sheet_names`) is no longer missing from `getMetadataMap()`, and fixed a `ClassCastException` when deserializing nested generic collections.
- **Python**: Fixed the Windows CLI binary still missing from the wheel.

---

## [4.2.9] - 2026-02-03

### Fixed

- **MCP server**: Fixed a "Cannot start a runtime from within a runtime" panic when running the MCP server in Docker.
- **Python**: Fixed an "embedded binary not found" error on Windows caused by missing `.exe` extension handling.
- **OCR fallback**: Fixed scanned PDFs incorrectly skipping OCR; mixed-content PDFs with some scanned pages are now properly OCR'd.

---

## [4.2.8] - 2026-02-02

### Fixed

- **Python**: Fixed `ChunkingConfig` serializing wrong field names (`max_characters`/`overlap` instead of `max_chars`/`max_overlap`).
- **Java**: Fixed an ARM64 SIGBUS crash when reading error details.
- **Ruby**: Fixed a `LoadError` during native extension compilation caused by a missing `rb_sys` runtime dependency.

---

## [4.2.7] - 2026-02-01

### Added

- **API**: Added an OpenAPI schema for the `/extract` endpoint with full type documentation.
- **Chunking**: Added a unified `ChunkingConfig` with canonical field names and serde aliases for backwards compatibility.
- **OCR**: Added `KREUZBERG_OCR_LANGUAGE="all"` to auto-detect and use all installed Tesseract languages.

### Fixed

- **C#**: Fixed `Attributes` deserialization on ARM64 to handle both array-of-arrays and object JSON formats.
- Overhauled type definitions across the Elixir, TypeScript, PHP, Ruby, Python, Java, C#, and Go bindings to match the Rust source, correcting many struct/field names, types, and optionality.

### Performance

- Reduced heap allocations across string-literal fields and the RST, FictionBook, and email extractors, and enabled zero-copy cloning of extracted image data.

---

## [4.2.6] - 2026-01-31

### Fixed

- **Python:** Restored missing `output_format`, `result_format`, `elements`, and `djot_content` fields on `ExtractionResult`.
- **Python:** Chunks now return proper `Chunk` objects with attribute access instead of dicts.

---

## [4.2.5] - 2026-01-30

### Fixed

- **Python:** Restored missing `OutputFormat`/`ResultFormat` exports that caused `ImportError`, and fixed Python 3.10 `StrEnum` compatibility.
- **PHP:** Aligned `ImageExtractionConfig`, `PdfConfig`, `ImagePreprocessingConfig`, and `ExtractionConfig` with the Rust core and removed phantom parameters.
- **TypeScript/Node:** Restored missing `elements` field with `Element`, `ElementMetadata`, and `BoundingBox` types.
- **C#:** Fixed enum serialization on .NET 9+.

### Added

- **Node:** Added Bun runtime support.

### Changed

- Achieved `PageContent` field parity across all language bindings.

---

## [4.2.4] - 2026-01-29

### Fixed

- **TypeScript/Node:** Restored missing `elements` field with `Element`, `ElementType`, `BoundingBox`, and `ElementMetadata` types.
- **Rust Core:** Fixed `KeywordConfig` failing to deserialize partial configs.
- **C#:** Fixed `Element` deserialization for the `element_based` result format.

---

## [4.2.3] - 2026-01-28

### Fixed

- **API:** Endpoints (`/embed`, `/chunk`, others) now reject JSON arrays in request bodies with a 400 instead of malfunctioning.
- **CLI:** `--format json` now serializes the complete `ExtractionResult` including chunks, embeddings, images, pages, and elements.
- **MCP:** Tool responses now return the full JSON-serialized `ExtractionResult`, matching API and CLI output; fixed a config-merge bug that corrupted boolean settings.
- **Elixir:** Added `ExtractionConfig.new/0` and `new/1`; renamed `Chunk.text` to `content` for parity.
- **C#:** File-not-found now throws `KreuzbergIOException` instead of `KreuzbergValidationException`.
- **WASM:** Fixed `initWasm()` failing on Cloudflare Workers and Vercel Edge; added an `initWasm({ wasmModule })` option for explicit module injection.

---

## [4.2.2] - 2026-01-28

### Changed

- **PHP/Go/TypeScript:** Removed non-canonical `ExtractionConfig` and result fields so all bindings match the Rust source.
- **Ruby/Java:** Fixed `enable_quality_processing` default from `false` to `true` to match Rust.

### Fixed

- **Elixir:** Fixed `force_build: true` breaking production installs; source builds now only run in development.
- **Docker:** Fixed "OCR backend 'tesseract' not registered" and "Failed to initialize embedding model" errors via tessdata discovery and a persistent model cache.
- **API:** JSON errors now return a proper `ErrorResponse`; added chunking/embed validation (including `overlap` < `max_characters`); `EmbeddingConfig.model` defaults to the "balanced" preset.
- **Rust Core:** Fixed XLSX out-of-memory on Excel Solver files declaring extreme cell dimensions.

---

## [4.2.1] - 2026-01-27

### Fixed

- **Rust Core:** Fixed reversed PPTX image page numbers; accept all output-format aliases (`plain`, `text`, `markdown`, `md`, `djot`, `html`); return an IO (not validation) error for file-not-found; log previously silent plugin failures.
- **Ruby:** `extract`/`detect` now accept both positional and keyword arguments; renamed `image_extraction` to `images` with a backward-compatible alias.
- **PHP:** Renamed fields to canonical names (`images`, `pages`, `pdfOptions`, `postprocessor`, `tokenReduction`) and added missing `postprocessor`/`tokenReduction`.
- **Java:** Added `getImages()`/`images()` aliases for image extraction.
- **WASM:** Added `outputFormat`, `resultFormat`, and `htmlOptions` to the config interface.
- **Go/Elixir:** Added `text` and `md` output-format aliases.

### Documentation

- Added a Kubernetes deployment guide with health checks and troubleshooting.

---

## [4.2.0] - 2026-01-26

### Added

- **MCP:** Full `config` parameter support on all tools, enabling complete configuration pass-through.
- **CLI:** Added `--output-format` (replaces `--content-format`), `--result-format`, `--config-json`, and `--config-json-base64` flags.
- **All bindings:** Added `outputFormat`/`output_format` (Plain, Markdown, Djot, HTML) and `resultFormat`/`result_format` (Unified, ElementBased) fields; Go adds functional options, Java adds Builder methods.
- **PHP:** Added 6 missing config fields (`useCache`, `enableQualityProcessing`, `forceOcr`, `maxConcurrentExtractions`, `resultFormat`, `outputFormat`).

### Changed

- Configuration precedence is now CLI flag > inline JSON > config file > defaults.

### Fixed

- **Ruby:** Fixed batch chunking operations.

### BREAKING CHANGES

- **MCP only (AI-facing, no end-user impact):** Removed top-level `enable_ocr`/`force_ocr`; use `config.ocr.enable_ocr` and `config.force_ocr`. Old names accepted in v4.2 with deprecation warnings.

### Deprecated

- `--content-format` CLI flag and `KREUZBERG_CONTENT_FORMAT` env var deprecated in favor of `--output-format` / `KREUZBERG_OUTPUT_FORMAT` (backward compatible).

---

## [4.1.2] - 2026-01-25

### Added

- **Ruby:** Added Ruby 4.0 support.

### Fixed

- **Ruby:** Fixed gem native extension build failure.
- **Go:** Fixed Windows hang caused by an FFI mutex deadlock.

---

## [4.1.1] - 2026-01-23

### Fixed

- **PPTX:** Fixed extraction failing on shapes without text (e.g. image placeholders).
- Added PPSX (PowerPoint Show) and PPTM (macro-enabled) file support.

---

## [4.1.0] - 2026-01-21

### Added

- **API:** Added `POST /chunk` endpoint with configurable `max_characters`, `overlap`, and `trim`.
- **Core:** Added Djot markup support (`.djot`) with parser, structured representation, and YAML frontmatter; Djot output for HTML/OCR; a `ContentFormat` enum (Plain, Markdown, Djot, HTML) with `--content-format`; and an element-based result format (Unstructured.io-compatible semantic elements).
- **All bindings:** Added content/result format configuration plus `Element`, `ElementType`, `ElementMetadata`, `BoundingBox`, and `DjotContent` types.

### Fixed

- **Python:** Restored missing type exports (`Element`, `ElementMetadata`, `ElementType`, `BoundingBox`, `HtmlImageMetadata`).
- **Elixir:** Fixed crash when extracting DOCX files with keyword metadata.

---

## [4.0.8] - 2026-01-17

### Changed

- **Docker:** Migrated images to GitHub Container Registry (`ghcr.io/kreuzberg-dev/kreuzberg`).

### Fixed

- **C FFI:** Fixed empty `HtmlConversionOptions` serializing as `null`, causing FFI errors.
- **Python:** Fixed missing `_internal_bindings.pyi` type stub in wheels.

---

## [4.0.6] - 2026-01-14

### Fixed

- **PHP:** Fixed runtime panic from unregistered `ChunkMetadata` and `Keyword` classes.

---

## [4.0.5] - 2026-01-14

### Added

- **Go:** Added an automated installer that downloads the correct platform-specific FFI library from releases.

---

## [4.0.4] - 2026-01-13

### Fixed

- **Docker:** Fixed `MissingDependencyError` when extracting legacy MS Office formats by adding LibreOffice symlinks and runtime dependencies.

---

## [4.0.3] - 2026-01-12

### Added

- **HTML config:** Full `html_options` configuration is now available from config files and all language bindings.

### Fixed

- **Go:** Fixed header include path so `go get` no longer fails with missing-header errors.
- **C#:** Fixed `JsonException` during keyword extraction; keywords now deserialize as `ExtractedKeyword` objects.
- **Distribution:** Made the Homebrew tap public to enable `brew install kreuzberg-dev/tap/kreuzberg`.

---

## [4.0.2] - 2026-01-12

### Fixed

- **Go:** Fixed module tag format so `go get` works correctly.
- **Elixir:** Fixed macOS native library extension (`.dylib` instead of `.so`).

---

## [4.0.1] - 2026-01-11

### Fixed

- **Elixir:** Fixed NIF binaries not being published to releases, which broke `rustler_precompiled`.
- **Python:** Fixed `kreuzberg-tesseract` missing from source distributions, breaking builds from source.
- **WASM:** Removed a call to the non-existent `detectMimeType()` API.
- **Ruby:** Updated RBS type definitions to match keyword-argument signatures.

---

## [4.0.0] - 2026-01-10

### Highlights

First stable release of Kreuzberg v4 — a complete rewrite with a Rust core and polyglot bindings for Python, TypeScript, Ruby, PHP, Java, Go, C#, Elixir, and WebAssembly.

### Added

- Python FFI error introspection via `get_last_error_code()` and `get_last_panic_context()`.
- PHP custom extractor support, with metadata and tables flowing through to results.
- Dynamic Tesseract language discovery from the local installation.

### Removed

- Removed the v3 legacy Python package and infrastructure.

## [3.22.0] - 2025-11-27

### Fixed

- Fixed EasyOCR import error handling.

---

## [3.21.0] - 2025-11-05

### Added

- Rewritten on a Rust core with bindings for Python, TypeScript, Ruby, Java, Go, and C#, plus a REST API and MCP server.

### Changed

- Architecture restructured around a Rust core with thin language-specific wrappers.

### Security

- Sandboxed subprocess execution, input validation, and memory safety via Rust.

### Performance

- Streaming PDF extraction, SIMD optimizations, and ONNX Runtime for embeddings.

---

## [3.20.2] - 2025-10-11

### Fixed

- Fixed missing optional dependency errors in GMFT extractor.

---

## [3.20.1] - 2025-10-11

- Maintenance release (internal changes only).

---

## [3.20.0] - 2025-10-11

### Added

- Python 3.14 support.

### Changed

- Migrated HTML extractor to html-to-markdown v2.

---

## [3.19.1] - 2025-09-30

### Fixed

- Fixed Windows Tesseract 5.5.0 HOCR output compatibility.

---

## [3.19.0] - 2025-09-29

### Fixed

- Fixed Tesseract PSM handling and aligned sync/async OCR pipelines.
- Fixed ValidationError handling in batch processing.

---

## [3.18.0] - 2025-09-27

### Added

- API server configuration via environment variables.
- Auto-download of missing spaCy models for entity extraction.

### Fixed

- Fixed HOCR parsing issues.

---

## [3.17.0] - 2025-09-17

### Added

- Token reduction for text optimization with streaming support.

### Fixed

- Fixed excessive markdown escaping in OCR output. (#133)

---

## [3.16.0] - 2025-09-16

### Added

- Enhanced JSON extraction with schema analysis and custom field detection.

### Fixed

- Fixed `HTMLToMarkdownConfig` not being exported in the public API.
- Fixed Windows-specific path issues.

---

## [3.15.0] - 2025-09-14

### Added

- Comprehensive image extraction support.
- Polars DataFrame and PIL Image serialization for API responses.

### Fixed

- Fixed TypeError with unhashable dict in API config merging.

---

## [3.14.0] - 2025-09-13

### Added

- DPI configuration system for OCR processing.

### Changed

- Enhanced API with 1GB upload limit and comprehensive OpenAPI documentation.
- Completed pandas to polars migration.

---

## [3.13.0] - 2025-09-04

### Added

- Runtime configuration API via query parameters and headers.
- OCR caching for EasyOCR and PaddleOCR backends.

### Fixed

- Fixed Tesseract TSV output and table extraction.
- Fixed UTF-8 encoding handling across document processing.
- Fixed regression in PDF extraction and XLS file handling.

---

## [3.12.0] - 2025-08-30

### Added

- Multilingual OCR support in Docker images with flexible backend selection.

### Fixed

- Fixed naming conflict in CLI config command.

---

## [3.11.1] - 2025-08-13

### Fixed

- Faster startup by deferring numpy import until image processing.

---

## [3.11.0] - 2025-08-01

### Fixed

- Fixed image extractor async delegation.
- Fixed timezone handling in spreadsheet metadata.

---

## [3.10.0] - 2025-07-29

### Added

- PDF password support through new crypto extra feature.

---

## [3.9.0] - 2025-07-17

- Maintenance release (internal changes only).

---

## [3.8.0] - 2025-07-16

- Maintenance release (internal changes only).

---

## [3.7.0] - 2025-07-11

### Added

- MCP server enabling Claude integration with document extraction.

### Fixed

- Fixed chunk parameters to prevent overlap validation errors.

---

## [3.6.0] - 2025-07-04

### Added

- Language detection integrated into the extraction pipeline.

### Changed

- Entity extraction now uses spaCy, replacing gliner.

---

## [3.5.0] - 2025-07-04

### Added

- Language detection with configurable backends.
- Full synchronous support for PaddleOCR and EasyOCR backends.

### Changed

- Python 3.10+ now required (3.9 support dropped).

---

## [3.4.0] - 2025-07-03

### Added

- REST API (Litestar) for web-based document extraction.

### Fixed

- Fixed race conditions in GMFT and Tesseract caching.

---

## [3.3.0] - 2025-06-23

### Added

- GMFT table extraction via an isolated process wrapper.
- CLI support.
- Pure synchronous extractors without anyio dependencies.
- Document-level caching with per-file locks and parallel batch processing.

### Fixed

- Fixed Windows-specific multiprocessing failures.
- Fixed file existence validation in extraction functions.

### Changed

- Faster cache serialization (msgpack replaces msgspec JSON, ~5x).

---

## [3.2.0] - 2025-06-23

### Added

- GPU acceleration for OCR and ML operations.
- Multiple language support for EasyOCR.

### Fixed

- Fixed EasyOCR byte string issues.

---

## [3.1.0] - 2025-03-28

### Added

- GMFT (Give Me Formatted Tables) support for vision-based table extraction.

### Changed

- Image extraction now non-optional in results.

---

## [3.0.0] - 2025-03-23

### Added

- Chunking for document segmentation.
- Hooks system for pre/post-processing.
- OCR backend abstraction with EasyOCR and PaddleOCR support and multi-language OCR.

### Fixed

- Fixed Windows error message handling.
- Fixed PaddleOCR integration issues.

---

## See Also

- [Configuration Reference](https://docs.kreuzberg.dev/reference/configuration/) - Detailed configuration options
- [Migration Guides](https://docs.kreuzberg.dev/migration/from-unstructured/) - Migration from other libraries
- [Format Support](https://docs.kreuzberg.dev/reference/formats/) - Supported file formats
- [Extraction Guide](https://docs.kreuzberg.dev/guides/extraction/) - Extraction examples

[4.8.5]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.8.5
[4.8.4]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.8.4
[4.8.3]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.8.3
[4.8.2]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.8.2
[4.8.1]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.8.1
[4.8.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.8.0
[4.7.4]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.7.4
[4.7.3]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.7.3
[4.7.2]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.7.2
[4.7.1]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.7.1
[4.7.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.7.0
[4.6.3]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.6.3
[4.6.2]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.6.2
[4.6.1]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.6.1
[4.6.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.6.0
[4.5.4]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.5.4
[4.5.3]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.5.3
[4.5.2]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.5.2
[4.5.1]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.5.1
[4.5.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.5.0
[4.4.6]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.4.6
[4.4.5]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.4.5
[4.4.4]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.4.4
[4.4.3]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.4.3
[4.4.2]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.4.2
[4.4.1]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.4.1
[4.4.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.4.0
[4.3.8]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.3.8
[4.3.7]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.3.7
[4.3.6]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.3.6
[4.3.5]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.3.5
[4.3.4]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.3.4
[4.3.3]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.3.3
[4.3.2]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.3.2
[4.3.1]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.3.1
[4.3.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.3.0
[4.2.15]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.15
[4.2.14]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.14
[4.2.13]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.13
[4.2.12]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.12
[4.2.11]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.11
[4.2.10]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.10
[4.2.9]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.9
[4.2.8]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.8
[4.2.7]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.7
[4.2.6]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.6
[4.2.5]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.5
[4.2.4]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.4
[4.2.3]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.3
[4.2.2]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.2
[4.2.1]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.1
[4.2.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.2.0
[4.1.2]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.1.2
[4.1.1]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.1.1
[4.1.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.1.0
[4.0.8]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.0.8
[4.0.6]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.0.6
[4.0.5]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.0.5
[4.0.4]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.0.4
[4.0.3]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.0.3
[4.0.2]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.0.2
[4.0.1]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.0.1
[4.0.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v4.0.0
[3.22.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.22.0
[3.21.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.21.0
[3.20.2]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.20.2
[3.20.1]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.20.1
[3.20.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.20.0
[3.19.1]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.19.1
[3.19.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.19.0
[3.18.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.18.0
[3.17.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.17.0
[3.16.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.16.0
[3.15.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.15.0
[3.14.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.14.0
[3.13.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.13.0
[3.12.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.12.0
[3.11.1]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.11.1
[3.11.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.11.0
[3.10.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.10.0
[3.9.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.9.0
[3.7.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.7.0
[3.6.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.6.0
[3.5.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.5.0
[3.4.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.4.0
[3.3.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.3.0
[3.2.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.2.0
[3.1.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.1.0
[3.0.0]: https://github.com/kreuzberg-dev/kreuzberg/releases/tag/v3.0.0
