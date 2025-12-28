---
priority: critical
---

# Rust Kreuzberg-Specific Conventions

**Edition 2024**: let-chains, gen blocks, if/match guards. **Naming**: PascalCase (types), snake_case (fns/vars/modules), SCREAMING_SNAKE_CASE (consts).

**Error handling**: Result<T, KreuzbergError>, never .unwrap() in production, use `?`, KreuzbergError::Io always bubbles up (CRITICAL), SAFETY comments for unsafe, handle lock poisoning.

**Async**: Tokio throughout, #[tokio::main]/#[tokio::test], provide _sync wrappers, never std::thread::sleep.

**Memory**: Arc for shared ownership, Mutex/RwLock for interior mutability, streaming for large files, RAII patterns.

**Performance**: ahash for HashMap, lazy_static/once_cell, SIMD where appropriate, zero-copy (&str/&[u8]).

**Plugins**: Traits (DocumentExtractor, OcrBackend, PostProcessor, Validator), Arc<dyn Trait> storage, Send+Sync, registry pattern.

**Zero clippy warnings** (cargo clippy -- -D warnings).

**Core structure**: src/{api,cache,chunking,core,extraction,extractors,image,keywords,language_detection,mcp,ocr,pdf,plugins,stopwords,text,utils}.
Plugin flow: File→MIME→Registry→Extractor→Pipeline→Result.
