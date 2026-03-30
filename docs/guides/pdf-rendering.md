# PDF Page Rendering

!!! info "Added in v4.6.2"

## Overview

The PDF page rendering API renders individual PDF pages as PNG images. Unlike the extraction pipeline -- which parses text, tables, and metadata -- this API produces raw pixel data suitable for downstream visual processing.

Use this API when you need the visual representation of a page rather than (or in addition to) its textual content.

## Use Cases

- **Thumbnails** -- Generate preview images for document browsers and search results.
- **Vision model input** -- Feed pages to vision-language models (VLMs) such as Qwen, GPT-4V, or Gemini for visual question answering, chart understanding, and layout-aware extraction.
- **Page-by-page processing** -- Process pages individually in pipelines that mix OCR, layout detection, and custom logic.
- **Custom OCR pipelines** -- Render pages to images and route them to your own OCR backend instead of using Kreuzberg's built-in OCR.

---

## Two Approaches

Kreuzberg provides two complementary APIs:

| API | When to Use |
|---|---|
| **Single page** (`render_pdf_page`) | You know which page you need, or you only need a few pages. |
| **Iterator** (`PdfPageIterator`) | You need to process every page in sequence without loading all images into memory. |

### Single Page

Render one page by its zero-based index.

=== "Python"

    --8<-- "snippets/python/api/render_pdf_page.md"

=== "TypeScript"

    --8<-- "snippets/typescript/api/render_pdf_page.md"

=== "Rust"

    --8<-- "snippets/rust/api/render_pdf_page.md"

=== "Go"

    --8<-- "snippets/go/api/render_pdf_page.md"

=== "Java"

    --8<-- "snippets/java/api/render_pdf_page.md"

=== "C#"

    --8<-- "snippets/csharp/api/render_pdf_page.md"

=== "Ruby"

    --8<-- "snippets/ruby/api/render_pdf_page.md"

=== "PHP"

    --8<-- "snippets/php/api/render_pdf_page.md"

=== "R"

    --8<-- "snippets/r/api/render_pdf_page.md"

=== "Elixir"

    --8<-- "snippets/elixir/api/render_pdf_page.md"

=== "C"

    --8<-- "snippets/c/api/render_pdf_page.md"

### Page Iterator

Iterate all pages in sequence. The iterator renders one page at a time and releases its memory before advancing, so only one PNG is resident at any moment.

=== "Python"

    --8<-- "snippets/python/api/render_pdf_page_iterator.md"

=== "TypeScript"

    --8<-- "snippets/typescript/api/render_pdf_page_iterator.md"

=== "Rust"

    --8<-- "snippets/rust/api/render_pdf_page_iterator.md"

=== "Go"

    --8<-- "snippets/go/api/render_pdf_page_iterator.md"

=== "Java"

    --8<-- "snippets/java/api/render_pdf_page_iterator.md"

=== "C#"

    --8<-- "snippets/csharp/api/render_pdf_page_iterator.md"

=== "C"

    --8<-- "snippets/c/api/render_pdf_page_iterator.md"

!!! note "Iterator availability"
    `PdfPageIterator` is available in Python, TypeScript, Rust, Go, Java, C#, and C. Ruby, PHP, R, and Elixir provide `render_pdf_page` only -- iterate pages with a loop over page indices.

---

## DPI Configuration

The `dpi` parameter controls the resolution of the rendered PNG. Higher DPI produces larger, more detailed images.

| DPI | Pixel Size (Letter) | Use Case |
|---|---|---|
| 72 | 612 x 792 | Thumbnails, quick previews |
| 150 (default) | 1275 x 1650 | General-purpose rendering, screen display |
| 300 | 2550 x 3300 | OCR input, print-quality output |

!!! tip "DPI for OCR"
    When rendering pages for OCR (Tesseract, PaddleOCR, or a vision model), use **300 DPI** for best recognition accuracy. The default 150 DPI is sufficient for display but may reduce OCR quality on small text.

---

## Memory Efficiency

Rendering a full PDF into memory can be expensive. A 100-page document at 300 DPI produces roughly 2.5 GB of uncompressed pixel data before PNG compression.

The `PdfPageIterator` addresses this by rendering one page at a time:

1. The iterator opens the PDF and reads its page tree (lightweight).
2. Each call to `next()` renders a single page to PNG bytes.
3. The previous page's pixel buffer is released before the next page is rendered.

This keeps peak memory proportional to **one page** rather than the entire document, regardless of page count.

For single-page rendering with `render_pdf_page`, memory usage is inherently bounded to one page.

---

## Examples

### Generate Thumbnails

```python title="Python"
from kreuzberg import render_pdf_page

# Low DPI for small thumbnails
thumbnail = render_pdf_page("report.pdf", page_index=0, dpi=72)
with open("thumbnail.png", "wb") as f:
    f.write(thumbnail)
```

### Feed Pages to a Vision Model

```python title="Python"
import base64
from kreuzberg import render_pdf_page

png = render_pdf_page("chart.pdf", page_index=2, dpi=300)
b64 = base64.b64encode(png).decode()

# Pass b64 to your VLM API as an image input
```

### Batch-Render All Pages

```python title="Python"
from pathlib import Path
from kreuzberg import render_pdf_page

output_dir = Path("pages")
output_dir.mkdir(exist_ok=True)

# Use render_pdf_page in a loop for simple cases
for i in range(total_pages):
    png = render_pdf_page("document.pdf", page_index=i, dpi=150)
    (output_dir / f"page_{i}.png").write_bytes(png)
```
