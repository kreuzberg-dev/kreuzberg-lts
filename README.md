# Kreuzberg

<div align="center" style="display: flex; flex-wrap: wrap; gap: 8px; justify-content: center; margin: 20px 0;">
  <!-- Language Bindings -->
  <a href="https://crates.io/crates/kreuzberg">
    <img src="https://img.shields.io/crates/v/kreuzberg?label=Rust&color=007ec6" alt="Rust">
  </a>
  <a href="https://hex.pm/packages/kreuzberg">
    <img src="https://img.shields.io/hexpm/v/kreuzberg?label=Elixir&color=007ec6" alt="Elixir">
  </a>
  <a href="https://pypi.org/project/kreuzberg/">
    <img src="https://img.shields.io/pypi/v/kreuzberg?label=Python&color=007ec6" alt="Python">
  </a>
  <a href="https://www.npmjs.com/package/@kreuzberg/node">
    <img src="https://img.shields.io/npm/v/@kreuzberg/node?label=Node.js&color=007ec6" alt="Node.js">
  </a>
  <a href="https://www.npmjs.com/package/@kreuzberg/wasm">
    <img src="https://img.shields.io/npm/v/@kreuzberg/wasm?label=WASM&color=007ec6" alt="WASM">
  </a>

  <a href="https://central.sonatype.com/artifact/dev.kreuzberg/kreuzberg">
    <img src="https://img.shields.io/maven-central/v/dev.kreuzberg/kreuzberg?label=Java&color=007ec6" alt="Java">
  </a>
  <a href="https://github.com/kreuzberg-dev/kreuzberg-lts/releases">
    <img src="https://img.shields.io/github/v/tag/kreuzberg-dev/kreuzberg-lts?label=Go&color=007ec6&filter=v4.10.1" alt="Go">
  </a>
  <a href="https://www.nuget.org/packages/Kreuzberg/">
    <img src="https://img.shields.io/nuget/v/Kreuzberg?label=C%23&color=007ec6" alt="C#">
  </a>
  <a href="https://packagist.org/packages/kreuzberg/kreuzberg">
    <img src="https://img.shields.io/packagist/v/kreuzberg/kreuzberg?label=PHP&color=007ec6" alt="PHP">
  </a>
  <a href="https://rubygems.org/gems/kreuzberg">
    <img src="https://img.shields.io/gem/v/kreuzberg?label=Ruby&color=007ec6" alt="Ruby">
  </a>
  <a href="https://kreuzberg-dev.r-universe.dev/kreuzberg">
    <img src="https://img.shields.io/badge/R-kreuzberg-007ec6" alt="R">
  </a>
  <a href="https://github.com/kreuzberg-dev/kreuzberg-lts/pkgs/container/kreuzberg">
    <img src="https://img.shields.io/badge/Docker-007ec6?logo=docker&logoColor=white" alt="Docker">
  </a>
  <a href="https://github.com/kreuzberg-dev/kreuzberg-lts/releases">
    <img src="https://img.shields.io/badge/C-FFI-007ec6" alt="C">
  </a>
  <a href="https://artifacthub.io/packages/search?repo=kreuzberg">
    <img src="https://img.shields.io/endpoint?url=https://artifacthub.io/badge/repository/kreuzberg" alt="Artifact Hub">
  </a>

  <!-- Project Info -->
  <a href="https://github.com/kreuzberg-dev/kreuzberg-lts/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
  </a>
  <a href="https://docs.kreuzberg.dev">
    <img src="https://img.shields.io/badge/docs-kreuzberg.dev-007ec6" alt="Documentation">
  </a>
  <a href="https://docs.kreuzberg.dev/demo.html">
    <img src="https://img.shields.io/badge/%E2%96%B6%EF%B8%8F_Live_Demo-007ec6" alt="Live Demo">
  </a>
  <a href="https://huggingface.co/Kreuzberg">
    <img src="https://img.shields.io/badge/%F0%9F%A4%97_Hugging_Face-007ec6" alt="Hugging Face">
  </a>
</div>

<div align="center" style="margin-top: 20px;">
  <a href="https://discord.gg/xt9WY3GnKR">
      <img height="22" src="https://img.shields.io/badge/Discord-Join%20our%20community-7289da?logo=discord&logoColor=white" alt="Discord">
  </a>
</div>

> [!NOTE]
> **Kreuzberg v4 LTS** — the long-term-support line for Kreuzberg **v4**. Active development has moved to
> **[Xberg](https://github.com/xberg-io/xberg)** (v5+). This repository receives critical bug and security
> fixes until the **end of 2026, on a best-effort basis**. New features land in Xberg.
> See the **[migration guide & LTS policy →](https://docs.kreuzberg.dev/lts/)**

Extract text, metadata, and code intelligence from 97+ file formats and 300+ programming languages at native speeds without needing a GPU.

## Key Features

- **Code intelligence** – Functions, classes, imports, symbols, and docstrings from [248 programming languages](https://docs.tree-sitter-language-pack.xberg.io) via tree-sitter, with semantic chunking
- **91+ file formats** – PDF, Office documents, images, HTML/XML, emails, archives, and academic formats across 8 categories
- **Polyglot** – Native bindings for Rust, Python, TypeScript/Node.js, Ruby, Go, Java, C#, PHP, Elixir, R, and C
- **LLM intelligence** – VLM OCR, schema-constrained JSON extraction, and provider-hosted embeddings across 146 LLM providers (including local engines) via [liter-llm](https://github.com/xberg-io/liter-llm)
- **OCR support** – Tesseract (incl. WASM), PaddleOCR, EasyOCR (Python), and VLM OCR; extensible via plugin API
- **High performance** – Rust core with native PDFium, SIMD, full parallelism, and streaming parsers for multi-GB files
- **Flexible deployment** – Library, CLI, REST API server, or MCP server
- **GFM-quality output** – Comrak rendering with cross-format parity (Markdown, HTML, Djot, Plain) plus token-efficient TOON serialization

**[Documentation](https://docs.kreuzberg.dev/)** | **[Live Demo](https://docs.kreuzberg.dev/demo.html)** | **[Installation](#installation)**

## Installation

Each language binding provides comprehensive documentation with examples and best practices. Choose your platform to get started:

**Scripting Languages:**

- **[Python](https://github.com/kreuzberg-dev/kreuzberg-lts/tree/main/packages/python)** – PyPI package, async/sync APIs, OCR backends (Tesseract, PaddleOCR, EasyOCR)
- **[Ruby](https://github.com/kreuzberg-dev/kreuzberg-lts/tree/main/packages/ruby)** – RubyGems package, idiomatic Ruby API, native bindings
- **[PHP](https://github.com/kreuzberg-dev/kreuzberg-lts/tree/main/packages/php)** – Composer package, modern PHP 8.4+ support, type-safe API, async extraction
- **[Elixir](https://github.com/kreuzberg-dev/kreuzberg-lts/tree/main/packages/elixir)** – Hex package, OTP integration, concurrent processing
- **[R](https://github.com/kreuzberg-dev/kreuzberg-lts/tree/main/packages/r)** – r-universe package, idiomatic R API, extendr bindings

**JavaScript/TypeScript:**

- **[@kreuzberg/node](https://github.com/kreuzberg-dev/kreuzberg-lts/tree/main/crates/kreuzberg-node)** – Native NAPI-RS bindings for Node.js/Bun, fastest performance
- **[@kreuzberg/wasm](https://github.com/kreuzberg-dev/kreuzberg-lts/tree/main/packages/typescript)** – WebAssembly for browsers/Deno/Cloudflare Workers, full feature parity (PDF, Excel, OCR, archives)

**Compiled Languages:**

- **[Go](https://github.com/kreuzberg-dev/kreuzberg-lts/tree/main/v4)** – Go module with FFI bindings, context-aware async
- **[Java](https://github.com/kreuzberg-dev/kreuzberg-lts/tree/main/packages/java)** – Maven Central, Foreign Function & Memory API
- **[C#](https://github.com/kreuzberg-dev/kreuzberg-lts/tree/main/packages/csharp)** – NuGet package, .NET 6.0+, full async/await support

**Native:**

- **[Rust](https://github.com/kreuzberg-dev/kreuzberg-lts/tree/main/crates/kreuzberg)** – Core library, flexible feature flags, zero-copy APIs
- **[C (FFI)](https://github.com/kreuzberg-dev/kreuzberg-lts/tree/main/crates/kreuzberg-ffi)** – C header + shared library, pkg-config/CMake support, cross-platform

**Containers:**

- **[Docker](https://docs.kreuzberg.dev/guides/docker/)** – Official images with API, CLI, and MCP server modes (Core: ~1.0-1.3GB, Full: ~1.0-1.3GB with OCR + legacy format support)

**Command-Line:**

- **[CLI](https://docs.kreuzberg.dev/cli/usage/)** – Cross-platform binary, batch processing, MCP server mode

> All language bindings include precompiled binaries for both x86_64 and aarch64 architectures on Linux and macOS.

## Platform Support

Complete architecture coverage across all language bindings:

| Language | Linux x86_64 | Linux aarch64 | macOS ARM64 | Windows x64 |
|----------|:------------:|:-------------:|:-----------:|:-----------:|
| Python | ✅ | ✅ | ✅ | ✅ |
| Node.js | ✅ | ✅ | ✅ | ✅ |
| WASM | ✅ | ✅ | ✅ | ✅ |
| Ruby | ✅ | ✅ | ✅ | - |
| R | ✅ | ✅ | ✅ | ✅ |
| Elixir | ✅ | ✅ | ✅ | ✅ |
| Go | ✅ | ✅ | ✅ | ✅ |
| Java | ✅ | ✅ | ✅ | ✅ |
| C# | ✅ | ✅ | ✅ | ✅ |
| PHP | ✅ | ✅ | ✅ | ✅ |
| Rust | ✅ | ✅ | ✅ | ✅ |
| C (FFI) | ✅ | ✅ | ✅ | ✅ |
| CLI | ✅ | ✅ | ✅ | ✅ |
| Docker | ✅ | ✅ | ✅ | - |

**Note**: ✅ = Precompiled binaries available with instant installation. WASM runs in any environment with WebAssembly support (browsers, Deno, Bun, Cloudflare Workers). All platforms are tested in CI. MacOS support is Apple Silicon only.

### Embeddings Support (Optional)

Embeddings require **ONNX Runtime 1.24+** (`brew install onnxruntime`, or [download a release](https://github.com/microsoft/onnxruntime/releases)). All other features work without it. See the [Embeddings Guide](https://docs.kreuzberg.dev/features/#embeddings).

## Supported Formats

91+ file formats across 8 categories with intelligent format detection and comprehensive metadata extraction:

- **Office** – Word (`.docx`/`.odt`/`.pages`), spreadsheets (`.xlsx`/`.ods`/`.numbers`), presentations (`.pptx`/`.key`), `.pdf`, eBooks (`.epub`/`.fb2`), `.dbf`, Hangul (`.hwp`/`.hwpx`)
- **Images (OCR)** – `.png`/`.jpg`/`.gif`/`.webp`/`.bmp`/`.tiff`, advanced (`.jp2`/`.jbig2`/`.pnm` via pure-Rust decoders), and `.svg`
- **Web & Data** – `.html`/`.xml`, structured data (`.json`/`.yaml`/`.toml`/`.csv`), and text/markup (`.md`/`.djot`/`.rst`/`.org`/`.rtf`)
- **Email & Archives** – `.eml`/`.msg`, and `.zip`/`.tar`/`.gz`/`.7z` with recursive extraction
- **Academic & Scientific** – citations (`.bib`/`.ris`/`.nbib`/`.csl`), `.tex`/`.typ`/`.jats`/`.ipynb`, and publishing/documentation formats (`.docbook`/`.opml`/`.pod`/`.troff`)

**[Complete Format Reference →](https://docs.kreuzberg.dev/reference/formats/)**

### Code Intelligence (248 Languages)

Extract structure (functions, classes, imports, symbols), parse docstrings (Google, NumPy, Sphinx, JSDoc, RustDoc, +10 more), and split code by semantic boundaries. Powered by [tree-sitter-language-pack](https://github.com/xberg-io/tree-sitter-language-pack) with dynamic grammar download — see the [TSLP documentation](https://docs.tree-sitter-language-pack.xberg.io) for the full language list.

## AI Coding Assistants

Kreuzberg ships with an [Agent Skill](https://agentskills.io) that teaches AI coding assistants how to use the library correctly. It works with Claude Code, Codex, Gemini CLI, Cursor, VS Code, Amp, Goose, Roo Code, and any tool supporting the Agent Skills standard.

Install the skill into any project using the [Vercel Skills CLI](https://github.com/vercel-labs/skills):

```bash
npx skills add kreuzberg-dev/kreuzberg-lts
```

The skill is located at [`skills/kreuzberg/SKILL.md`](skills/kreuzberg/SKILL.md) and is automatically discovered by supported AI coding tools once installed.

## Documentation

- **[Installation Guide](https://docs.kreuzberg.dev/getting-started/installation/)** – Setup and dependencies
- **[User Guide](https://docs.kreuzberg.dev/guides/extraction/)** – Comprehensive usage guide
- **[API Reference](https://docs.kreuzberg.dev/reference/api-python/)** – Complete API documentation
- **[Format Support](https://docs.kreuzberg.dev/reference/formats/)** – Supported file formats
- **[OCR Backends](https://docs.kreuzberg.dev/guides/ocr/)** – OCR engine setup
- **[CLI Guide](https://docs.kreuzberg.dev/cli/usage/)** – Command-line usage
- **[Migration Guides](https://docs.kreuzberg.dev/migration/from-unstructured/)** – Upgrading from other libraries

## Part of Xberg.io

Kreuzberg v4 LTS is the legacy line of the document-intelligence library now developed as **Xberg**. It is one of the open-source projects from Kreuzberg, Inc.:

- [Xberg](https://github.com/xberg-io/xberg) — document intelligence: text, tables, metadata from 91+ formats with optional OCR (the active successor to Kreuzberg).
- [Xberg Enterprise](https://github.com/xberg-io/xberg-enterprise) — managed extraction API with SDKs, dashboards, and observability.
- [crawlberg](https://github.com/xberg-io/crawlberg) — web crawling and scraping with HTML→Markdown and headless-Chrome fallback.
- [html-to-markdown](https://github.com/xberg-io/html-to-markdown) — fast, lossless HTML→Markdown engine.
- [liter-llm](https://github.com/xberg-io/liter-llm) — universal LLM API client with native bindings for 14 languages and 143 providers.
- [tree-sitter-language-pack](https://github.com/xberg-io/tree-sitter-language-pack) — tree-sitter grammars and code-intelligence primitives.
- [alef](https://github.com/xberg-io/alef) — the polyglot binding generator that produces every per-language binding across the polyglot repos.

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.
