# Token Reduction

Reduce token count while preserving meaning for LLM pipelines.

Set the reduction mode on `token_reduction` (a `TokenReductionOptions`).

| Mode         | Effect                                                      |
| ------------ | ----------------------------------------------------------- |
| `off`        | No reduction; text returned as-is.                          |
| `light`      | Remove only the most common stopwords.                      |
| `moderate`   | Balanced stopword removal and redundancy filtering.         |
| `aggressive` | Aggressive filtering; may remove less common content words. |
| `maximum`    | Maximum compression; prioritizes brevity over completeness. |

## Configuration

=== "Python"

    --8<-- "snippets/python/config/token_reduction_config.md"

=== "TypeScript"

    --8<-- "snippets/typescript/config/token_reduction_config.md"

=== "Rust"

    --8<-- "snippets/rust/advanced/token_reduction_config.md"

=== "Go"

    --8<-- "snippets/go/config/token_reduction_config.md"

=== "Java"

    --8<-- "snippets/java/config/token_reduction_config.md"

=== "C#"

    --8<-- "snippets/csharp/advanced/token_reduction_config.md"

=== "Ruby"

    --8<-- "snippets/ruby/config/token_reduction_config.md"

## Example

=== "Python"

    --8<-- "snippets/python/utils/token_reduction_example.md"

=== "TypeScript"

    --8<-- "snippets/typescript/utils/token_reduction_example.md"

=== "Rust"

    --8<-- "snippets/rust/advanced/token_reduction_example.md"

=== "Go"

    --8<-- "snippets/go/advanced/token_reduction_example.md"

=== "Java"

    --8<-- "snippets/java/advanced/token_reduction_example.md"

=== "C#"

    --8<-- "snippets/csharp/advanced/token_reduction_example.md"

=== "Ruby"

    --8<-- "snippets/ruby/advanced/token_reduction_example.md"

## See also

- [Configuration Reference](../reference/configuration.md#tokenreductionoptions) — all reduction options
- [LLM Integration](llm-integration.md) — use token reduction with LLM pipelines
