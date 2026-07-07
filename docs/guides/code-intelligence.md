# Code Intelligence

Xberg integrates [tree-sitter-language-pack](https://docs.tree-sitter-language-pack.xberg.io) (TSLP) to parse source code files. When you extract a source code file, Xberg detects the programming language, parses it with tree-sitter, and emits the source as content split at semantic boundaries -- functions, classes, and modules -- rather than fixed line counts.

Language support covers **306 programming languages** via tree-sitter grammars. See the [TSLP documentation](https://docs.tree-sitter-language-pack.xberg.io) for the full language list.

See the [TreeSitterConfig reference](../reference/configuration.md#treesitterconfig) for all configuration options.

## What You Get

Extracting a source code file produces:

- **`content`** -- the source text, split into code segments at tree-sitter chunk boundaries and rendered in the configured output format. When chunking is enabled, each segment is preceded by a heading naming its enclosing function, class, or module. With chunking disabled (the default), the whole source is emitted as a single code block.
- **`metadata.format`** -- tagged as `code` (`format_type: "code"`). This is a marker only; it carries no additional fields such as language, structure, or imports.

Tree-sitter's structured analysis (structure, imports, exports, comments, docstrings, symbols, diagnostics) is computed internally to drive chunking, but is **not currently exposed** in the extraction result. The `code_intelligence` field on the result is reserved for this data and is always `null` in the current release. Read the rendered `content` rather than expecting structured code-intelligence fields.

## Getting Started

Code extraction is enabled by default when the `tree-sitter` feature flag is active. Extract a source code file and read `content`:

=== "Rust"

    ```rust title="basic.rs"
    use xberg::{extract, ExtractInput, ExtractionConfig};

    let config = ExtractionConfig::default();
    let output = extract(ExtractInput::from_uri("app.py"), &config).await?;
    let result = &output.results[0];

    // The rendered source is in the content field.
    println!("{}", result.content);

    // metadata.format is tagged as Code -- a marker with no payload.
    if matches!(result.metadata.format, Some(xberg::types::FormatMetadata::Code)) {
        println!("Detected a source code file");
    }
    ```

=== "Python"

    ```python title="basic.py"
    import xberg

    config = xberg.ExtractionConfig()
    output = await xberg.extract(xberg.ExtractInput(kind="uri", uri="app.py"), config=config)
    result = output.results[0]

    # The rendered source is in the content field.
    print(result.content)

    # metadata["format"] is tagged as code -- a marker with no extra fields.
    fmt = result.metadata.get("format")
    if fmt and fmt.get("format_type") == "code":
        print("Detected a source code file")
    ```

=== "TypeScript"

    ```typescript title="basic.ts"
    import { ExtractInputKind, extract } from "@xberg-io/xberg";

    const output = await extract({
      kind: ExtractInputKind.Uri,
      uri: "app.ts",
    });
    const result = output.results[0];

    // The rendered source is in the content field.
    console.log(result.content);

    // metadata.format is tagged "code" -- a marker with no extra fields.
    const fmt = result.metadata?.format;
    if (fmt?.formatType === "code") {
      console.log("Detected a source code file");
    }
    ```

=== "Go"

    ```go title="basic.go"
    result, err := xberg.Extract("app.py", nil)
    if err != nil {
        log.Fatal(err)
    }

    fmt.Println(result.Content)
    // result.Metadata.Format is tagged "code" for source files.
    ```

## Configuration

Use `TreeSitterConfig` to control tree-sitter processing. Set `enabled: false` to skip code intelligence entirely. `chunk_max_size` controls where the source is split into segments; when unset (the default), the whole file is emitted as a single block.

!!! warning "Analysis toggles are inert in the current release"

    Only `enabled` and `chunk_max_size` affect the emitted output. The `structure`, `imports`, `exports`, `comments`, `docstrings`, `symbols`, and `diagnostics` toggles configure tree-sitter's internal analysis, but that analysis is not surfaced in the result -- setting them has no observable effect on `content` or `metadata`.

=== "Rust"

    ```rust title="config.rs"
    use xberg::{ExtractionConfig, TreeSitterConfig, TreeSitterProcessConfig};

    let config = ExtractionConfig {
        tree_sitter: Some(TreeSitterConfig {
            process: TreeSitterProcessConfig {
                chunk_max_size: Some(4096),  // split source at chunk boundaries
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    };
    ```

=== "Python"

    ```python title="config.py"
    import xberg

    config = xberg.ExtractionConfig(
        tree_sitter={
            "process": {
                "chunk_max_size": 4096,
            }
        }
    )
    ```

=== "TypeScript"

    ```typescript title="config.ts"
    import { ExtractionConfig } from "@xberg-io/xberg";

    const config: ExtractionConfig = {
      treeSitter: {
        process: {
          chunkMaxSize: 4096,
        },
      },
    };
    ```

=== "TOML"

    ```toml title="xberg.toml"
    [tree_sitter.process]
    chunk_max_size = 4096
    ```

### Configuration Fields

See [`TreeSitterConfig`](../reference/configuration.md#treesitterconfig) and [`TreeSitterProcessConfig`](../reference/configuration.md#treesitterprocessconfig) for all fields.

## Chunked Content

With `chunk_max_size` set, tree-sitter splits the source at function, class, and module boundaries and Xberg emits each chunk as a separate code segment in `content`, preceded by a heading naming its enclosing scope:

```python title="chunked.py"
import xberg

config = xberg.ExtractionConfig(
    tree_sitter={"process": {"chunk_max_size": 2048}}
)

output = await xberg.extract(
    xberg.ExtractInput(kind="uri", uri="large_module.py"), config=config
)
result = output.results[0]

# Chunk boundaries are reflected in the layout of result.content.
print(result.content)
```

Structured per-chunk data (byte spans, parent context, per-chunk language) is not exposed in the current release. For programmatic chunking with offsets and metadata -- for example, in a RAG pipeline -- use Xberg's [chunking pipeline](chunking.md) instead.

## Language Detection

Xberg detects the programming language in two ways:

1. **File extension** (fast path) -- when using `extract`, the extension is matched against 248 known language extensions
2. **Shebang line** (fallback) -- when the extension is missing or ambiguous, the first line is checked for `#!/usr/bin/env python`, `#!/bin/bash`, and so on.

If neither method identifies the language, extraction returns an `UnsupportedFormat` error. The detected language is used internally for parsing and is not exposed as a structured field on the result.

## Language Support

Tree-sitter-language-pack supports 306 programming languages, including Python, Rust, TypeScript, JavaScript, Go, Java, C/C++, Ruby, PHP, C#, Swift, Kotlin, and Elixir. For the full list, see the [TSLP language reference](https://docs.tree-sitter-language-pack.xberg.io).

## Related Documentation

- [Configuration Reference](../reference/configuration.md#treesitterconfig) -- TreeSitterConfig and TreeSitterProcessConfig fields
- [Chunking Guide](chunking.md) -- programmatic chunking with offsets and metadata
- [tree-sitter-language-pack documentation](https://docs.tree-sitter-language-pack.xberg.io) -- Full language support reference
