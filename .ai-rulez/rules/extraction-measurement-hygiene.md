---
type: Decision
title: Extraction Measurement Hygiene
description: Isolate Kreuzberg 4.x extraction and OCR caches and read its flat CLI results correctly
x-ai-rulez:
  kind: rule
  id: extraction-measurement-hygiene
  metadata:
    priority: high
---

# Extraction Measurement Hygiene

These are the Kreuzberg 4.x contracts. The shared successor-product examples have different cache variables and CLI result shapes.

Isolate the cache per invocation and guard the directory before using it:

```sh
cache_directory=$(mktemp -d) || exit 90
[ -n "$cache_directory" ] && [ -d "$cache_directory" ] || exit 90
KREUZBERG_CACHE_DIR="$cache_directory" kreuzberg extract --config "$cfg" --no-cache true "$file"
```

Keep the temporary directory until its evidence is recorded; remove only that invocation's owned directory afterward. `cache_dir.rs` accepts an empty environment value as a relative path, so an empty variable can write into the current directory. `XBERG_CACHE_DIR` does not configure this legacy product.

Kreuzberg 4.x has no `--ocr-no-cache` CLI flag. Disable Tesseract caching in the extraction configuration:

```toml
[ocr]
backend = "tesseract"
language = "eng"

[ocr.tesseract_config]
language = "eng"
use_cache = false
```

When `tesseract_config` is present, its language is used; the outer OCR language is inherited only when that nested configuration is absent. Select the actual corpus language explicitly.

Extraction cache keys combine file content and configuration hashes. OCR keys combine image, backend and configuration hashes. They do not automatically isolate builds with a build-ID salt. A comparison across binaries must use separate cache directories for both runs. Prove a cache is active before drawing a conclusion about cache isolation.

The single-file CLI JSON output is a flat `ExtractionResult` object; batch JSON is an array of such objects. Read `content` and other fields directly from each result. There is no successor-style `result` envelope or CLI timing wrapper. MCP tools return their payload inside text content; decode that text using the requested response format first.

Measure wall time outside the CLI if needed. On a shared machine, elapsed timings are contended upper bounds; record that limitation. Compare absolute counts of preserved content and passing cases, and retain revision-specific binaries outside shared build directories before rebuilding.
