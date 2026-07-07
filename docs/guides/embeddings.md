# Embeddings

Turn extracted text into vectors for semantic search and RAG, using local ONNX models or a registered backend — no external API calls. Enable the `embeddings` feature to use in-process embedding backends.

| Preset         | Model                        | Dimensions | Chunk Size (chars) | Use Case                                                |
| -------------- | ---------------------------- | ---------- | ------------------ | ------------------------------------------------------- |
| `fast`         | all-MiniLM-L6-v2 (quantized) | 384        | 512                | Quick prototyping, development, resource-constrained    |
| `balanced`     | BGE-base-en-v1.5             | 768        | 1024               | General-purpose RAG, production deployments, English    |
| `quality`      | BGE-large-en-v1.5            | 1024       | 2000               | Complex documents, maximum accuracy, sufficient compute |
| `multilingual` | multilingual-e5-base         | 768        | 1024               | International documents, mixed-language content         |

The chunk-size column is the preset's target chunk size in characters (its `chunk_size`), not a token limit.

## Model Types

Select the embedding source via the `model` field of `EmbeddingConfig`. `EmbeddingModelType` has four variants, tagged by `type`:

- `preset` — a bundled ONNX model configuration by `name` (see the preset table above). Recommended default.
- `custom` — any ONNX embedding model from HuggingFace, given its `model_id` and output `dimensions`.
- `llm` — a provider-hosted embedding model through liter-llm, configured by a nested `llm` (`LlmConfig`), e.g. `openai/text-embedding-3-small`.
- `plugin` — an in-process backend registered by `name` via the plugin system (see below).

## In-Process Embedding Backends (Plugin Variant)

Plug a caller-managed embedder (e.g. `llama-cpp-python`, `sentence-transformers`) into Xberg via the `Plugin` variant of `EmbeddingModelType` — Xberg calls back into the registered backend instead of running its own ONNX model.

1. Register the backend once at startup via `xberg::plugins::register_embedding_backend(Arc::new(MyEmbedder))`. The backend implements `EmbeddingBackend` (a `Plugin`-inheriting async trait with `dimensions()` and `embed(texts) -> Vec<Vec<f32>>`).
2. Reference it by name in `EmbeddingConfig`: `{ "model": { "type": "plugin", "name": "my-embedder" } }`.
3. Optional: set `EmbeddingConfig.max_embed_duration_secs` (default 60) to bound the wait on a hung backend; `None` disables the timeout.

Rust extraction configuration and `XBERG_EMBEDDING_PLUGIN_NAME` accept the Plugin variant once a backend is registered.

**Fork-safety**: Python callers running under `multiprocessing`, `gunicorn`'s prefork worker, or Celery prefork must re-register the backend in each child process — native-backed embedders (including `llama-cpp-python`) aren't fork-safe. Use `os.register_at_fork(after_in_child=reregister_fn)` to automate the re-registration.

## Configuration

=== "Python"

    --8<-- "snippets/python/utils/embedding_with_chunking.md"

=== "TypeScript"

    --8<-- "snippets/typescript/utils/embedding_with_chunking.md"

=== "Rust"

    --8<-- "snippets/rust/advanced/embedding_with_chunking.md"

=== "Go"

    --8<-- "snippets/go/advanced/embedding_with_chunking.md"

=== "Java"

    --8<-- "snippets/java/advanced/embedding_with_chunking.md"

=== "C#"

    --8<-- "snippets/csharp/advanced/embedding_with_chunking.md"

=== "Ruby"

    --8<-- "snippets/ruby/advanced/embedding_with_chunking.md"

## Standalone Embedding

Call `embed_texts` (or `embed_texts_async`) to embed a list of strings directly with an `EmbeddingConfig`, bypassing extraction and chunking. Each input string maps to one output vector.

=== "Python"

    --8<-- "snippets/python/utils/standalone_embed.md"

=== "TypeScript"

    --8<-- "snippets/typescript/utils/standalone_embed.md"

=== "Rust"

    --8<-- "snippets/rust/utils/standalone_embed.md"

=== "Go"

    --8<-- "snippets/go/utils/standalone_embed.md"

=== "Java"

    --8<-- "snippets/java/utils/standalone_embed.md"

=== "C#"

    --8<-- "snippets/csharp/utils/standalone_embed.md"

=== "Ruby"

    --8<-- "snippets/ruby/utils/standalone_embed.md"

## Vector Database Integration

=== "Python"

    --8<-- "snippets/python/utils/vector_database_integration.md"

=== "TypeScript"

    --8<-- "snippets/typescript/utils/vector_database_integration.md"

=== "Rust"

    --8<-- "snippets/rust/advanced/vector_database_integration.md"

=== "Go"

    --8<-- "snippets/go/advanced/vector_database_integration.md"

=== "Java"

    --8<-- "snippets/java/advanced/vector_database_integration.md"

=== "C#"

    --8<-- "snippets/csharp/advanced/vector_database_integration.md"

=== "Ruby"

    --8<-- "snippets/ruby/advanced/vector_database_integration.md"

## See also

- [Chunking](chunking.md) — split documents before embedding for RAG
- [Configuration Reference](../reference/configuration.md#embeddingconfig) — all embedding options
- [LLM Integration](llm-integration.md) — use embeddings with LLMs
