# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [4.0.0-rc.24] - 2026-01-01

### Fixed

- **Go Windows CI**: Added explicit CGO directives to bypass pkg-config on Windows
- **Ruby Windows Build**: Added Windows platform handling in build.rs, enabled embeddings feature on Windows GNU
- **Node Windows Tests**: Fixed symlink resolution using realpathSync for Windows compatibility
- **C# Tests**: Fixed null reference warnings in config tests
- **WASM CI**: Fixed artifact download path to include pkg/ subdirectory
- **Homebrew Formula**: Fixed bottle naming convention, added source sha256 fetching
- **PHP PIE Build**: Corrected task name and extension filename
- **C# NuGet Upload**: Added proper conditional check for artifact existence
- **Python CI**: Fixed test failures and compatibility issues
- **Elixir CI**: Fixed build and compilation warnings
- **WASM Deno**: Fixed type definitions and Ruby Windows build

## [4.0.0-rc.23] - 2026-01-01

### Added

#### Java
- **EmbeddingConfig class**: New type-safe configuration class with builder pattern for embedding generation
  - 7 configurable fields: model, normalize, batchSize, dimensions, useCache, showDownloadProgress, cacheDir
  - Comprehensive test suite with 27 test methods (321 lines)
  - Full integration with ExtractionConfig
  - toMap/fromMap serialization support

#### C#
- **EmbeddingConfig sealed class**: Type-safe replacement for Dictionary-based embedding configuration
  - 5 properties with init-only accessors
  - JSON serialization with snake_case mapping
  - Comprehensive test suite with 50 test methods
  - Updated ChunkingConfig to use EmbeddingConfig instead of Dictionary<string, object?>

#### Node.js (NAPI-RS)
- **Worker Thread Pool APIs**: Complete concurrent extraction system
  - `createWorkerPool(size?)`: Create worker pool with configurable size
  - `getWorkerPoolStats(pool)`: Monitor pool utilization
  - `extractFileInWorker(pool, ...)`: Extract single file in worker thread
  - `batchExtractFilesInWorker(pool, ...)`: Extract multiple files concurrently
  - `closeWorkerPool(pool)`: Graceful pool shutdown
  - 17 test methods (468 lines) covering all APIs
  - Auto-generated TypeScript type definitions via NAPI-RS

#### Test Coverage
- **Node.js**: 54 new tests (batch operations, worker pool, 15 config types)
- **WASM**: 122 new tests (batch operations, embeddings, keywords, tables, 8 config suites)
- **TypeScript**: 62 new tests (async operations, batch, 19 config types)
- **Java**: 27 new EmbeddingConfig tests, 13 new config type tests
- **C#**: 50 new EmbeddingConfig tests, 14 new config type tests
- **Python**: 14 new config type tests, batch operations, embeddings advanced tests
- **Ruby**: 14 new config type tests, async operations, batch operations
- **Go**: Comprehensive config tests, mutex safety tests
- **Total**: 200+ new tests across all bindings

#### Documentation
- **README Template System**: Template-based generation for all binding READMEs
  - Created `scripts/readme_templates/` with Jinja2 templates
  - Created `scripts/readme_config.yaml` for language-specific configurations
  - Added snippet system in `docs/snippets/` for code examples
  - Template partials for badges, installation, features, quick start
- **Worker Pool Documentation**: Complete examples and best practices
  - Code snippet: `docs/snippets/typescript/advanced/worker_pool.md`
  - Performance benefits and usage patterns documented
- **Config Discovery Documentation**: Automatic config file loading examples
  - Code snippet: `docs/snippets/typescript/config/config_discovery.md`
- **NAPI-RS Implementation Details**: Technical documentation for Node.js binding
  - Template partial: `scripts/readme_templates/partials/napi_implementation.md.jinja`
  - Threading model, memory management, performance characteristics

### Fixed

- **Page Marker Bug**: Fixed page markers to include page 1 (previously only inserted for page > 1)
  - Modified `crates/kreuzberg/src/pdf/text.rs:292` to fix insertion logic
  - Fixed C# default marker format in `packages/csharp/Kreuzberg/Serialization.cs:109`
  - Fixed C# config serialization in `packages/csharp/Kreuzberg/KreuzbergClient.cs:1274`
  - Added comprehensive test suite: `crates/kreuzberg/tests/page_markers.rs` (13 tests)

- **Go Concurrency Crashes**: Fixed segfaults and SIGTRAP errors in concurrent operations
  - Added `ffiMutex sync.Mutex` in `packages/go/v4/binding.go` for thread-safe FFI calls
  - PDFium is not thread-safe; all FFI calls now protected by mutex
  - Verified with `-race` flag: zero race conditions
  - All 410 tests now pass consistently without crashes

### Changed

- **Code Formatting**: Standardized formatting across all 10 language bindings
  - Rust: Applied `rustfmt` to all crates
  - Java: Applied Spotless (Google Java Format)
  - C#: Applied `dotnet format`
  - PHP: Applied PHP CS Fixer
  - Shell: Applied `shfmt` formatting
  - All pre-commit hooks now passing

- **README Updates**: Regenerated all binding READMEs from templates
  - Node.js: Added worker pool section, LibreOffice notes, NAPI-RS details
  - TypeScript: Updated with all new config types
  - All bindings: Consistent structure and formatting

### Performance

- **Node.js Worker Pools**:
  - Parallel document processing across CPU cores
  - Configurable pool size (defaults to CPU count)
  - Queue management for efficient task distribution
  - Prevents thread exhaustion with bounded concurrency

## [4.0.0-rc.22] - 2025-12-27
