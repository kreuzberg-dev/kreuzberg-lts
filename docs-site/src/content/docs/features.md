---
title: Features
description: "Capability overview for Kreuzberg v4 LTS: extraction, OCR, chunking and embeddings, output formats, servers, and plugins, each linked to its guide or reference."
---

Kreuzberg is a document intelligence library built on a Rust core with bindings for 12 languages. It extracts text, tables, and metadata from 91+ file formats, runs OCR when needed, and feeds the results through a configurable post-processing pipeline. Each capability below links to the guide or reference page with configuration details and code examples.

## Capabilities

- **Extraction** — Pull text, tables, metadata, and images from 91+ formats through native Rust extractors. No external tools such as LibreOffice are required. See [Extraction Basics](/guides/extraction/).
- **OCR** — Recognize text in scanned documents and images with Tesseract, PaddleOCR, or EasyOCR, optionally chained into a quality-driven fallback pipeline. See the [OCR Guide](/guides/ocr/).
- **Chunking and embeddings** — Split extracted text into sized chunks and generate local vector embeddings with FastEmbed for RAG pipelines. See [Chunking and Embeddings](/guides/chunking-embeddings/).
- **Output formats** — Render results as plain text, Markdown, Djot, HTML, or structured JSON. The HTML renderer ships styled `kb-*` classes and built-in themes. See [Output Formats](/guides/output-formats/) and [HTML Output](/guides/html-output/).
- **API and MCP server** — Serve extraction over HTTP (`kreuzberg serve`) or expose it to AI agents over MCP (`kreuzberg mcp`). See the [API Server Guide](/guides/api-server/).
- **Plugins** — Extend the pipeline with custom document extractors, OCR backends, validators, and post-processors. See the [Plugin System](/concepts/plugin-system/) concept and the [Creating Plugins](/guides/plugins/) guide.

## Format Support

Kreuzberg handles 91+ file formats across these categories:

- **Documents** — PDF, Word, PowerPoint, OpenDocument, RTF, plain text, Markdown, and more
- **Spreadsheets** — Excel, Numbers, OpenDocument, CSV, TSV, dBASE
- **Images** — JPEG, PNG, GIF, BMP, TIFF, WebP, JPEG 2000, and more (routed to OCR)
- **Email** — EML, MSG
- **Web and markup** — HTML, XHTML, XML, SVG
- **Structured data** — JSON, YAML, TOML
- **Archives** — ZIP, TAR, GZIP, 7-Zip
- **Academic** — EPUB, BibTeX, RIS, LaTeX, Typst, JATS, DocBook, and more

For the full format matrix with MIME types and extraction methods, see the [Format Support Reference](/reference/formats/).

## Pipeline

Every file flows through the same multi-stage pipeline: MIME detection, format extraction, optional OCR, post-processing (validators, quality processing, chunking, embeddings), and caching. For a stage-by-stage walkthrough, see [Extraction Pipeline](/concepts/extraction-pipeline/).

## Deployment Modes

| Mode | When to Use | Details |
|---|---|---|
| **Library** | Embedding extraction into your application | Import the package in Python, TypeScript, Rust, Go, Ruby, C#, Java, PHP, Elixir, R, or C |
| **CLI** | One-off extractions, scripting, CI pipelines | `kreuzberg extract document.pdf --format json` — see [CLI Usage](/cli/usage/) |
| **REST API** | Multi-service architectures, language-agnostic access | `kreuzberg serve --port 8000` — see [API Server Guide](/guides/api-server/) |
| **MCP Server** | AI agent integration | `kreuzberg mcp` — stdio transport with JSON-RPC 2.0 |
| **Docker** | Reproducible deployments with all dependencies bundled | `ghcr.io/kreuzberg-dev/kreuzberg-full:4` — see [Docker Guide](/guides/docker/) |

## Configuration

Kreuzberg reads configuration programmatically or from `kreuzberg.toml`, `kreuzberg.yaml`, or `kreuzberg.json`, auto-discovered from the current directory, `~/.config/kreuzberg/`, and `/etc/kreuzberg/`. Environment variables override file-based settings. See the [Configuration Guide](/guides/configuration/).

## Next Steps

- [Installation](/getting-started/installation/) — Install Kreuzberg for your language
- [Quick Start](/getting-started/quickstart/) — Extract your first document in minutes
- [Architecture](/concepts/architecture/) — Understand the Rust core and binding layers
