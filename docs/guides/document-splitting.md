---
description: Split a multi-document PDF into logical sub-documents and extract each from a single parse, using caller-supplied page ranges or heuristic boundary detection.
---

# Document Splitting

A single PDF often concatenates several logical documents — a scanned batch of invoices, a bundle of letters, a stack of forms. Use `split_and_extract` to detect sub-document boundaries and return one `ExtractedDocument` per segment from a **single parse**. You do not slice the PDF externally and re-parse each slice; the whole file is parsed once and partitioned by page attribution.

!!! info "Core-Rust only"
    `split_and_extract` is not part of the language-binding surface. It is excluded from the generated bindings (`split_and_extract` is listed in `alef.toml`), so it is available from Rust only. It also requires the `pdf` feature.

## When to Use It

Reach for splitting when one input file holds multiple independent documents that you want to process, store, or route separately, and you want to avoid parsing the PDF once per boundary. The single-parse guarantee matters when parsing is expensive (OCR, layout detection). Page, table, and image attribution is preserved in the original page coordinate system, so each segment reports its own counts and page numbers.

PDF is the only supported format today — it is the only page-addressable one. Any other MIME type returns `XbergError::UnsupportedFormat`.

## Strategies

`SplitStrategy` selects how the document is partitioned.

```rust
pub enum SplitStrategy {
    /// Heuristic boundary detection. Requires the `heuristics` feature.
    Auto,
    /// Caller-supplied 1-indexed, inclusive page ranges.
    PageRanges(Vec<RangeInclusive<u32>>),
}
```

- **`PageRanges`** — Deterministic. Supply 1-indexed inclusive page ranges when you already know the boundaries. Ranges are validated: each must satisfy `1 <= start <= end <= total_pages`, and the list must be non-empty. An out-of-bounds or reversed range returns `XbergError::Validation`.
- **`Auto`** (default) — Heuristic boundary detection via page-one markers, letterhead resets, and text-density shifts. Requires the `heuristics` feature; without it, `Auto` returns a `Validation` error directing you to `PageRanges`. A cohesive document with no boundary signals yields a single segment spanning all pages.

## Configuration

```rust
pub struct SplitConfig {
    /// Boundary-detection strategy.
    pub strategy: SplitStrategy,
    /// Extraction config applied to every segment.
    pub extraction: ExtractionConfig,
}
```

`SplitConfig::default()` uses `SplitStrategy::Auto` and a default `ExtractionConfig`. Construct with struct-update syntax to override the strategy. Per-page content extraction is forced on internally regardless of what `extraction` specifies — that is how segments are partitioned without a re-parse.

## Returned Segments

Each detected segment is a `SplitSegment`:

```rust
pub struct SplitSegment {
    /// 1-indexed inclusive page range in the original document.
    pub page_range: RangeInclusive<u32>,
    /// The sub-document extracted from `page_range`.
    pub document: ExtractedDocument,
}
```

The `document` is a normal `ExtractedDocument` scoped to its pages:

- `content` is reassembled from the segment's pages (blank pages skipped), not copied from the whole document.
- `tables`, `images`, and `pages` are filtered to those falling in `page_range`, with original page numbers preserved.
- `counts` (`DocumentCounts { pages, tables, images }`) and `metadata.pages.total_count` are recomputed for the segment.

## Example

```rust
use std::ops::RangeInclusive;

use xberg::{split_and_extract, SplitConfig, SplitStrategy, ExtractionConfig};

# async fn run(bytes: &[u8]) -> xberg::Result<()> {
// Deterministic: three known sub-documents in a 5-page batch.
let config = SplitConfig {
    strategy: SplitStrategy::PageRanges(vec![1..=2, 3..=3, 4..=5]),
    extraction: ExtractionConfig::default(),
};

let segments = split_and_extract(bytes, &config).await?;

for segment in &segments {
    let range: &RangeInclusive<u32> = &segment.page_range;
    let counts = &segment.document.counts;
    println!(
        "pages {}..={}: {} pages, {} tables, {} images",
        range.start(),
        range.end(),
        counts.pages,
        counts.tables,
        counts.images,
    );
    // segment.document.content holds the reassembled text for this sub-document.
}
# Ok(())
# }
```

For heuristic splitting, take the default strategy (requires the `heuristics` feature):

```rust
# use xberg::{split_and_extract, SplitConfig};
# async fn run(bytes: &[u8]) -> xberg::Result<()> {
let segments = split_and_extract(bytes, &SplitConfig::default()).await?;
# let _ = segments;
# Ok(())
# }
```

## Errors

- `XbergError::UnsupportedFormat` — input is not a PDF.
- `XbergError::Validation` — empty document, invalid or reversed page range, an empty `PageRanges` list, or `Auto` requested without the `heuristics` feature.
- Any error propagated from the underlying single-document extraction.

## See also

- [Rust Core API](rust-core-api.md) — the wider Rust-only surface, including `ExtractedDocument` fields
- [Extraction Basics](extraction.md) — the two-function extraction API mirrored by the bindings
