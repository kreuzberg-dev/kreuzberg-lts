# Kreuzberg vs Unstructured: Feature Comparison

A comprehensive comparison of Kreuzberg and Unstructured.io for document intelligence workloads.

## Executive Summary

| Aspect | Kreuzberg | Unstructured |
|--------|-----------|--------------|
| **Core Language** | Rust | Python |
| **Performance** | Rust-based native speed | Python-based |
| **Formats Supported** | 88+ | ~30 |
| **Language Bindings** | 10 (Python, TS, Ruby, PHP, Go, Java, C#, Elixir, Rust, WASM) | Python + API |
| **Deployment** | Self-hosted (CLI, API, library) | Cloud API + self-hosted |
| **Pricing** | Free & open-source | Free tier + paid plans |
| **Best For** | High-performance, polyglot stacks, self-hosted | Rapid prototyping, managed service |

## Feature Matrix

### Document Processing

| Feature | Kreuzberg | Unstructured | Notes |
|---------|-----------|--------------|-------|
| **PDF Extraction** | ✅ Full support | ✅ Full support | Kreuzberg has native hierarchy detection |
| **PDF Hierarchy (h1-h6)** | ✅ Font-size clustering | ✅ ML-based layout | Kreuzberg uses statistical clustering |
| **OCR (Tesseract)** | ✅ Built-in | ✅ Built-in | Both support Tesseract |
| **Table Detection** | ✅ Native | ✅ ML-based | Unstructured has better complex table support |
| **Image Extraction** | ✅ Full support | ✅ Full support | Both extract images with metadata |
| **Bounding Boxes** | ✅ Native (PDF) | ✅ Available | Kreuzberg preserves from source |
| **Multi-Page Support** | ✅ Per-page content | ✅ Page numbers | Kreuzberg has richer per-page metadata |

### Output Formats

| Format | Kreuzberg | Unstructured |
|--------|-----------|--------------|
| **Unified Text** | ✅ Default | ❌ |
| **Element-Based** | ✅ Optional | ✅ Default |
| **Per-Page JSON** | ✅ Native | ⚠️ Via elements |
| **Markdown** | ✅ Native (HTML→MD) | ❌ |
| **Structured Data** | ✅ JSON/YAML/TOML | ✅ JSON |

### Element Types

| Element Type | Kreuzberg | Unstructured | Notes |
|--------------|-----------|--------------|-------|
| Title | ✅ | ✅ | Kreuzberg adds hierarchy level metadata |
| NarrativeText | ✅ | ✅ | Both detect paragraphs |
| ListItem | ✅ | ✅ | Kreuzberg: bullets, numbered, lettered, indented |
| Table | ✅ | ✅ | Kreuzberg: tab-separated text |
| Image | ✅ | ✅ | Both include dimensions, format |
| PageBreak | ✅ | ✅ | Between multi-page content |
| Header | ⚠️ → Title | ✅ | Kreuzberg maps to title |
| Footer | ⚠️ → NarrativeText | ✅ | Kreuzberg treats as narrative |
| Address | ❌ | ✅ | Unstructured-specific |
| EmailAddress | ❌ | ✅ | Unstructured-specific |
| Formula | ❌ | ✅ | Unstructured-specific |

### Supported File Formats

**Kreuzberg (88+ formats)**:
- Documents: PDF, DOCX, DOC, ODT, RTF, TXT, Markdown, RST, LaTeX, Typst
- Presentations: PPTX, PPT, ODP, Keynote
- Spreadsheets: XLSX, XLS, ODS, CSV
- Web: HTML, XML, EPUB, FictionBook
- Code: Jupyter Notebooks, Source code (via tree-sitter)
- Data: JSON, YAML, TOML, BibTeX, OPML, OrgMode
- Images: PNG, JPEG, TIFF, WebP (via OCR)
- Email: EML, MSG

**Unstructured (~30 formats)**:
- Documents: PDF, DOCX, DOC, ODT, RTF, TXT
- Presentations: PPTX, PPT
- Spreadsheets: XLSX, XLS, CSV
- Web: HTML, XML, EPUB
- Data: JSON, Markdown
- Images: PNG, JPEG, TIFF (via OCR)
- Email: EML, MSG

**Winner**: Kreuzberg (broader format coverage)

### Metadata Richness

**Kreuzberg Metadata** (format-specific discriminated unions):
```json
{
  "title": "Document Title",
  "authors": ["Author 1", "Author 2"],
  "created_at": "2024-01-15T10:30:00Z",
  "modified_at": "2024-01-20T14:45:00Z",
  "language": "en",
  "format": {
    "format_type": "pdf",
    "page_count": 10,
    "version": "1.7",
    "is_encrypted": false,
    "permissions": {
      "print": true,
      "modify": false
    }
  }
}
```

**Unstructured Metadata**:
```json
{
  "filename": "document.pdf",
  "page_number": 1,
  "filetype": "application/pdf"
}
```

**Winner**: Kreuzberg (richer, format-specific metadata)

### Chunking & Embeddings

| Feature | Kreuzberg | Unstructured | Notes |
|---------|-----------|--------------|-------|
| **Text Chunking** | ✅ Basic (fixed-size) | ✅ Advanced (by_title) | Unstructured has smarter strategies |
| **Chunk Overlap** | ✅ Configurable | ✅ Configurable | Both support overlap |
| **Embedding Generation** | ✅ Built-in (ONNX) | ⚠️ External API | Kreuzberg: local ONNX models |
| **Embedding Models** | ✅ ONNX-based presets | ✅ OpenAI, Cohere, etc. | Kreuzberg: offline ONNX, Unstructured: API-based |
| **Page Range Tracking** | ✅ Native | ✅ Via metadata | Kreuzberg tracks first_page/last_page |

### Language Bindings & Integrations

**Kreuzberg**:
- ✅ Python (PyO3)
- ✅ TypeScript (NAPI-RS)
- ✅ Ruby (Magnus)
- ✅ PHP (ext-php-rs)
- ✅ Go (cgo FFI)
- ✅ Java (JNI FFI)
- ✅ C# (P/Invoke FFI)
- ✅ Elixir (Rustler NIFs)
- ✅ Rust (native)
- ✅ WASM (browser/Node/Deno/Workers)

**Unstructured**:
- ✅ Python (native)
- ✅ REST API (language-agnostic)
- ⚠️ Other languages via API only

**Winner**: Kreuzberg (native bindings for 10 languages)

### Deployment Options

**Kreuzberg**:
- ✅ CLI (single binary)
- ✅ Self-hosted API (Docker, native)
- ✅ Library (embedded in applications)
- ✅ WASM (browser-based processing)
- ❌ Managed cloud service

**Unstructured**:
- ✅ Managed API (cloud-hosted)
- ✅ Self-hosted API (Docker)
- ✅ Python library
- ❌ CLI
- ❌ Browser-based

### Cost Analysis

**Kreuzberg**:
- **License**: Apache 2.0 (free, open-source)
- **Infrastructure**: Self-hosted only (compute costs)
- **Total Cost**: Infrastructure + maintenance

**Unstructured**:
- **License**: Apache 2.0 (free, open-source)
- **Managed API**: Free tier (100 pages/month) + paid plans ($0.01-0.10/page)
- **Self-hosted**: Infrastructure costs only
- **Total Cost**: API fees OR infrastructure + maintenance

### Security & Compliance

| Feature | Kreuzberg | Unstructured |
|---------|-----------|--------------|
| **Data Privacy** | ✅ 100% on-prem | ⚠️ Cloud API or on-prem |
| **GDPR Compliance** | ✅ Self-managed | ⚠️ Varies (cloud API) |
| **SOC 2** | N/A (self-hosted) | ✅ (managed API) |
| **Air-Gapped** | ✅ Fully supported | ⚠️ Self-hosted only |
| **Audit Logs** | ⚠️ Basic (via API logs) | ✅ Advanced (managed) |

## Use Case Recommendations

### Choose Kreuzberg If:
- ✅ You need **maximum performance** (Rust-based native speed)
- ✅ You're building a **polyglot stack** (Python, TS, Go, etc.)
- ✅ You require **strict data privacy** (on-prem processing)
- ✅ You need to process **88+ file formats**
- ✅ You want **zero API fees** (fully self-hosted)
- ✅ You need **native bindings** for your language
- ✅ You're processing **large document volumes** (high throughput)
- ✅ You need **offline embeddings** (no external API calls)

### Choose Unstructured If:
- ✅ You need **ML-based layout detection** (GPU-accelerated)
- ✅ You want a **managed cloud service** (zero ops)
- ✅ You need **advanced chunking strategies** (by_title, semantic)
- ✅ You're prototyping and want **fast setup**
- ✅ You need **more granular element types** (Address, Formula, etc.)
- ✅ You're already using **OpenAI/Cohere APIs** for embeddings
- ✅ You have **low document volume** (free tier sufficient)

## Migration Path

**From Unstructured to Kreuzberg**:
1. Deploy Kreuzberg API (Docker or native)
2. Update endpoint URLs in your code
3. Add `output_format=element_based` for Unstructured-compatible output
4. Test with sample documents
5. Optimize with Kreuzberg-specific features (hierarchy, per-page, embeddings)

**From Kreuzberg to Unstructured**:
1. Sign up for Unstructured API key
2. Update endpoint URLs
3. Remove `output_format` parameter (element-based is default)
4. Adjust for different metadata structure

## Roadmap & Future Features

### Kreuzberg Planned Features:
- ⏳ Enhanced chunking strategies (by_title, semantic)
- ⏳ Layout detection models (optional GPU acceleration)
- ⏳ More element types (Header, Footer, Formula)
- ⏳ Cloud-hosted option (for non-self-hosters)

### Unstructured Strengths:
- ✅ Mature ML models (layout, tables)
- ✅ Large community & integrations
- ✅ Managed service with SLA

## Verdict

**Kreuzberg** excels at:
- Performance (Rust native)
- Polyglot support (10 language bindings)
- Format coverage (88+ formats)
- Self-hosted deployments
- Cost efficiency (zero API fees)

**Unstructured** excels at:
- ML-powered layout analysis
- Managed cloud service
- Advanced chunking strategies
- Larger ecosystem

**Recommendation**:
- **High-volume, polyglot, self-hosted** → Kreuzberg
- **Rapid prototyping, managed service** → Unstructured
- **Hybrid approach**: Use both (Kreuzberg for bulk processing, Unstructured for complex layouts)

## Further Reading

- [Migration Guide: Unstructured → Kreuzberg](../migration/from-unstructured.md)
- [Kreuzberg Documentation](https://github.com/kreuzberg-dev/kreuzberg)
- [Unstructured Documentation](https://unstructured.io/docs)
