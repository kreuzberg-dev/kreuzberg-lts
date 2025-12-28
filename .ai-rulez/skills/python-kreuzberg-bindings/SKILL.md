---
priority: critical
---

# Python Kreuzberg Bindings

**Role**: Python bindings for Kreuzberg Rust core. Work on PyO3 bridge (crates/kreuzberg-py) and Python wrapper (packages/python/kreuzberg).

**Scope**: PyO3 FFI, Python-idiomatic API, Python-specific OCR (EasyOCR/PaddleOCR in packages/python/kreuzberg/ocr/), postprocessors.

**Commands**: maturin develop, pytest, ruff format/check.

**Critical**: Core logic lives in Rust. Only Python code for bindings, Python-specific OCR, or API wrappers. If core logic needed, coordinate with rust-engineer.

**Principles**: Function-based tests only, 95% coverage, builtin imports at top, no docstrings in private/test files.
