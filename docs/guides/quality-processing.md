# Quality Processing

Score extracted text on a 0.0–1.0 scale, where 1.0 is highest quality. Scoring starts from `1.0` for clean prose: OCR, script, and navigation noise subtract penalties, while document structure and metadata add bonuses. The result is clamped to `[0.0, 1.0]`. Empty text scores `0.0`; text shorter than 10 characters scores `0.1`.

| Factor              | Max weight | Effect  | Detects                                                |
| ------------------- | ---------- | ------- | ------------------------------------------------------ |
| OCR Artifacts       | 30%        | Penalty | Scattered chars, repeated punctuation, malformed words |
| Script Content      | 20%        | Penalty | JavaScript, CSS, HTML tags                             |
| Navigation Elements | 10%        | Penalty | Breadcrumbs, pagination, skip links                    |
| Document Structure  | 20%        | Bonus   | Sentence/paragraph length, punctuation distribution    |
| Metadata Quality    | 10%        | Bonus   | Title, author, subject, description, keywords          |

Script and navigation penalties apply only to text longer than 1000 characters; shorter text is scored on OCR artifacts and structure alone.

## Configuration

=== "Python"

    --8<-- "snippets/python/config/quality_processing_config.md"

=== "TypeScript"

    --8<-- "snippets/typescript/config/quality_processing_config.md"

=== "Rust"

    --8<-- "snippets/rust/advanced/quality_processing_config.md"

=== "Go"

    --8<-- "snippets/go/config/quality_processing_config.md"

=== "Java"

    --8<-- "snippets/java/config/quality_processing_config.md"

=== "C#"

    --8<-- "snippets/csharp/advanced/quality_processing_config.md"

=== "Ruby"

    --8<-- "snippets/ruby/config/quality_processing_config.md"

## Example

=== "Python"

    --8<-- "snippets/python/utils/quality_processing_example.md"

=== "TypeScript"

    --8<-- "snippets/typescript/utils/quality_processing_example.md"

=== "Rust"

    --8<-- "snippets/rust/advanced/quality_processing_example.md"

=== "Go"

    --8<-- "snippets/go/advanced/quality_processing_example.md"

=== "Java"

    --8<-- "snippets/java/advanced/quality_processing_example.md"

=== "C#"

    --8<-- "snippets/csharp/advanced/quality_processing_example.md"

=== "Ruby"

    --8<-- "snippets/ruby/advanced/quality_processing_example.md"

## See also

- [Configuration Reference](../reference/configuration.md#extractionconfig) — the `enable_quality_processing` option
- [Extraction Basics](extraction.md) — core extraction pipeline
