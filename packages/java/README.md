# Java

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
  <a href="https://github.com/kreuzberg-dev/kreuzberg/releases">
    <img src="https://img.shields.io/github/v/tag/kreuzberg-dev/kreuzberg?label=Go&color=007ec6&filter=v4.0.0-*" alt="Go">
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

<!-- Project Info -->

<a href="https://github.com/kreuzberg-dev/kreuzberg/blob/main/LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
  </a>
  <a href="https://docs.kreuzberg.dev">
    <img src="https://img.shields.io/badge/docs-kreuzberg.dev-blue" alt="Documentation">
  </a>
</div>

<img width="1128" height="191" alt="Banner2" src="https://github.com/user-attachments/assets/419fc06c-8313-4324-b159-4b4d3cfce5c0" />

<div align="center" style="margin-top: 20px;">
  <a href="https://discord.gg/pXxagNK2zN">
      <img height="22" src="https://img.shields.io/badge/Discord-Join%20our%20community-7289da?logo=discord&logoColor=white" alt="Discord">
  </a>
</div>

Extract text, tables, images, and metadata from 56 file formats including PDF, Office documents, and images. Java bindings with type-safe API, Foreign Function & Memory API integration, and native performance.

> **Version 4.0.0 Release Candidate**
> Kreuzberg v4.0.0 is in **Release Candidate** stage. Bugs and breaking changes are expected.
> This is a pre-release version. Please test the library and [report any issues](https://github.com/kreuzberg-dev/kreuzberg/issues) you encounter.

## Installation

### Package Installation

Install via one of the supported package managers:

**Maven:**

```xml
<dependency>
    <groupId>dev.kreuzberg</groupId>
    <artifactId>kreuzberg</artifactId>
    <version>4.0.0-rc.24</version>
</dependency>
```

**Gradle:**

```gradle
implementation 'dev.kreuzberg:kreuzberg:4.0.0-rc.24'
```

**Maven with Specific Classifier (if needed):**

For platform-specific native libraries, Maven handles automatic classifier selection. If you need explicit control:

```xml
<dependency>
    <groupId>dev.kreuzberg</groupId>
    <artifactId>kreuzberg</artifactId>
    <version>4.0.0-rc.24</version>
    <!-- Classifiers: linux-x86_64, macos-aarch64, macos-x86_64, windows-x86_64 -->
</dependency>
```

### System Requirements

- **Java 21+** required (Java 25 recommended for best FFM API performance)
- **FFM API** enabled by default (no additional flags needed in Java 21+)
- Optional: [ONNX Runtime](https://github.com/microsoft/onnxruntime/releases) version 1.22.x for embeddings support
- Optional: [Tesseract OCR](https://github.com/tesseract-ocr/tesseract) for OCR functionality

### FFM API (Foreign Function & Memory API)

Kreuzberg uses the modern Foreign Function & Memory API (FFM API) for native interop instead of JNI. This provides:

- Type-safe native access without unsafe native code
- Better performance with reduced overhead
- Memory safety guarantees with Arena allocation
- Cleaner API without JNI boilerplate

No additional configuration is required. The FFM API is enabled by default in Java 21+.

## Quick Start

### Basic Extraction

Extract text, metadata, and structure from any supported document format:

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import java.io.IOException;
import java.util.Map;

public class BasicUsage {
    public static void main(String[] args) throws IOException {
        ExtractionResult result = Kreuzberg.extractFile("document.pdf");

        System.out.println("Content:");
        System.out.println(result.getContent());

        System.out.println("\nMetadata:");
        Map<String, Object> metadata = result.getMetadata();
        if (metadata != null) {
            System.out.println("Title: " + metadata.get("title"));
            System.out.println("Author: " + metadata.get("author"));
        }

        System.out.println("\nTables found: " + result.getTables().size());
        System.out.println("Images found: " + result.getImages().size());
    }
}
```

### Common Use Cases

#### Extract with Custom Configuration Using Builder Pattern

Kreuzberg uses a fluent builder pattern for configuring extraction behavior. All configuration is optional:

**With OCR (for scanned documents):**

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import dev.kreuzberg.KreuzbergException;
import dev.kreuzberg.config.ExtractionConfig;
import dev.kreuzberg.config.OcrConfig;
import java.io.IOException;

public class Main {
    public static void main(String[] args) {
        try {
            ExtractionConfig config = ExtractionConfig.builder()
                .ocr(OcrConfig.builder()
                    .backend("tesseract")
                    .language("eng")
                    .build())
                .build();

            ExtractionResult result = Kreuzberg.extractFile("scanned.pdf", config);
            System.out.println(result.getContent());
        } catch (IOException | KreuzbergException e) {
            System.err.println("Extraction failed: " + e.getMessage());
        }
    }
}
```

**Building Complex Configurations:**

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.config.ExtractionConfig;
import dev.kreuzberg.config.OcrConfig;
import dev.kreuzberg.config.TableConfig;

public class ConfigurationExample {
    public static void main(String[] args) throws Exception {
        // Build a comprehensive configuration using fluent API
        ExtractionConfig config = ExtractionConfig.builder()
            .ocr(OcrConfig.builder()
                .backend("tesseract")
                .language("eng")
                .build())
            .extractTables(true)
            .extractImages(true)
            .useCache(true)
            .build();

        ExtractionResult result = Kreuzberg.extractFile("document.pdf", config);

        System.out.println("Text: " + result.getContent());
        System.out.println("Tables: " + result.getTables().size());
        System.out.println("Images: " + result.getImages().size());
    }
}
```

#### Table Extraction

See [Table Extraction Guide](https://kreuzberg.dev/features/table-extraction/) for detailed examples.

#### Processing Multiple Files

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import dev.kreuzberg.KreuzbergException;
import java.io.IOException;
import java.util.Arrays;
import java.util.List;

try {
    List<String> files = Arrays.asList("doc1.pdf", "doc2.docx", "doc3.pptx");

    List<ExtractionResult> results = Kreuzberg.batchExtractFiles(files, null);

    for (int i = 0; i < results.size(); i++) {
        ExtractionResult result = results.get(i);
        System.out.println("File " + (i + 1) + ": " + result.getContent().length() + " characters");
    }
} catch (IOException | KreuzbergException e) {
    e.printStackTrace();
}
```

#### Async Processing

For non-blocking document processing:

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import java.nio.file.Path;
import java.util.concurrent.CompletableFuture;

public class Example {
    public static void main(String[] args) {
        CompletableFuture<ExtractionResult> future =
            Kreuzberg.extractFileAsync(Path.of("document.pdf"), null);

        future.thenAccept(result -> {
            System.out.println(result.getContent());
            System.out.println("Tables: " + result.getTables().size());
        }).join();
    }
}
```

### Next Steps

- **[Installation Guide](https://kreuzberg.dev/getting-started/installation/)** - Platform-specific setup
- **[API Documentation](https://kreuzberg.dev/api/)** - Complete API reference
- **[Examples & Guides](https://kreuzberg.dev/guides/)** - Full code examples and usage guides
- **[Configuration Guide](https://kreuzberg.dev/configuration/)** - Advanced configuration options
- **[Troubleshooting](https://kreuzberg.dev/troubleshooting/)** - Common issues and solutions

## Features

### Supported File Formats (56+)

56 file formats across 8 major categories with intelligent format detection and comprehensive metadata extraction.

#### Office Documents

| Category | Formats | Capabilities |
|----------|---------|--------------|
| **Word Processing** | `.docx`, `.odt` | Full text, tables, images, metadata, styles |
| **Spreadsheets** | `.xlsx`, `.xlsm`, `.xlsb`, `.xls`, `.xla`, `.xlam`, `.xltm`, `.ods` | Sheet data, formulas, cell metadata, charts |
| **Presentations** | `.pptx`, `.ppt`, `.ppsx` | Slides, speaker notes, images, metadata |
| **PDF** | `.pdf` | Text, tables, images, metadata, OCR support |
| **eBooks** | `.epub`, `.fb2` | Chapters, metadata, embedded resources |

#### Images (OCR-Enabled)

| Category | Formats | Features |
|----------|---------|----------|
| **Raster** | `.png`, `.jpg`, `.jpeg`, `.gif`, `.webp`, `.bmp`, `.tiff`, `.tif` | OCR, table detection, EXIF metadata, dimensions, color space |
| **Advanced** | `.jp2`, `.jpx`, `.jpm`, `.mj2`, `.pnm`, `.pbm`, `.pgm`, `.ppm` | OCR, table detection, format-specific metadata |
| **Vector** | `.svg` | DOM parsing, embedded text, graphics metadata |

#### Web & Data

| Category | Formats | Features |
|----------|---------|----------|
| **Markup** | `.html`, `.htm`, `.xhtml`, `.xml`, `.svg` | DOM parsing, metadata (Open Graph, Twitter Card), link extraction |
| **Structured Data** | `.json`, `.yaml`, `.yml`, `.toml`, `.csv`, `.tsv` | Schema detection, nested structures, validation |
| **Text & Markdown** | `.txt`, `.md`, `.markdown`, `.rst`, `.org`, `.rtf` | CommonMark, GFM, reStructuredText, Org Mode |

#### Email & Archives

| Category | Formats | Features |
|----------|---------|----------|
| **Email** | `.eml`, `.msg` | Headers, body (HTML/plain), attachments, threading |
| **Archives** | `.zip`, `.tar`, `.tgz`, `.gz`, `.7z` | File listing, nested archives, metadata |

#### Academic & Scientific

| Category | Formats | Features |
|----------|---------|----------|
| **Citations** | `.bib`, `.biblatex`, `.ris`, `.enw`, `.csl` | Bibliography parsing, citation extraction |
| **Scientific** | `.tex`, `.latex`, `.typst`, `.jats`, `.ipynb`, `.docbook` | LaTeX, Jupyter notebooks, PubMed JATS |
| **Documentation** | `.opml`, `.pod`, `.mdoc`, `.troff` | Technical documentation formats |

**[Complete Format Reference](https://kreuzberg.dev/reference/formats/)**

### Key Capabilities

- **Text Extraction** - Extract all text content with position and formatting information

- **Metadata Extraction** - Retrieve document properties, creation date, author, etc.

- **Table Extraction** - Parse tables with structure and cell content preservation

- **Image Extraction** - Extract embedded images and render page previews

- **OCR Support** - Integrate multiple OCR backends for scanned documents

- **Async/Await** - Non-blocking document processing with concurrent operations

- **Plugin System** - Extensible post-processing for custom text transformation

- **Embeddings** - Generate vector embeddings using ONNX Runtime models

- **Batch Processing** - Efficiently process multiple documents in parallel

- **Memory Efficient** - Stream large files without loading entirely into memory

- **Language Detection** - Detect and support multiple languages in documents

- **Configuration** - Fine-grained control over extraction behavior

### Performance Characteristics

| Format | Speed | Memory | Notes |
|--------|-------|--------|-------|
| **PDF (text)** | 10-100 MB/s | ~50MB per doc | Fastest extraction |
| **Office docs** | 20-200 MB/s | ~100MB per doc | DOCX, XLSX, PPTX |
| **Images (OCR)** | 1-5 MB/s | Variable | Depends on OCR backend |
| **Archives** | 5-50 MB/s | ~200MB per doc | ZIP, TAR, etc. |
| **Web formats** | 50-200 MB/s | Streaming | HTML, XML, JSON |

## OCR Support

Kreuzberg supports multiple OCR backends for extracting text from scanned documents and images:

- **Tesseract**

### OCR Configuration Example

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import dev.kreuzberg.KreuzbergException;
import dev.kreuzberg.config.ExtractionConfig;
import dev.kreuzberg.config.OcrConfig;
import java.io.IOException;

public class Main {
    public static void main(String[] args) {
        try {
            ExtractionConfig config = ExtractionConfig.builder()
                .ocr(OcrConfig.builder()
                    .backend("tesseract")
                    .language("eng")
                    .build())
                .build();

            ExtractionResult result = Kreuzberg.extractFile("scanned.pdf", config);
            System.out.println(result.getContent());
        } catch (IOException | KreuzbergException e) {
            System.err.println("Extraction failed: " + e.getMessage());
        }
    }
}
```

## Async Support

This binding provides full async/await support for non-blocking document processing:

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import java.nio.file.Path;
import java.util.concurrent.CompletableFuture;

public class Example {
    public static void main(String[] args) {
        CompletableFuture<ExtractionResult> future =
            Kreuzberg.extractFileAsync(Path.of("document.pdf"), null);

        future.thenAccept(result -> {
            System.out.println(result.getContent());
            System.out.println("Tables: " + result.getTables().size());
        }).join();
    }
}
```

### Keywords Extraction

Extract key terms and concepts from documents using YAKE or RAKE algorithms:

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import dev.kreuzberg.KreuzbergException;
import dev.kreuzberg.config.ExtractionConfig;
import dev.kreuzberg.config.KeywordConfig;
import java.io.IOException;

public class KeywordExample {
    public static void main(String[] args) {
        try {
            // Extract keywords using YAKE algorithm
            ExtractionConfig config = ExtractionConfig.builder()
                .keywords(KeywordConfig.builder()
                    .algorithm("yake")
                    .maxKeywords(10)
                    .minScore(0.5)
                    .language("en")
                    .yakeParams(KeywordConfig.YakeParams.builder()
                        .windowSize(3)
                        .build())
                    .build())
                .build();

            ExtractionResult result = Kreuzberg.extractFile("document.pdf", config);
            System.out.println("Keywords extracted successfully");

            // Or use RAKE algorithm
            ExtractionConfig rakeConfig = ExtractionConfig.builder()
                .keywords(KeywordConfig.builder()
                    .algorithm("rake")
                    .maxKeywords(15)
                    .rakeParams(KeywordConfig.RakeParams.builder()
                        .minWordLength(3)
                        .maxWordsPerPhrase(3)
                        .build())
                    .build())
                .build();
        } catch (IOException | KreuzbergException e) {
            System.err.println("Keyword extraction failed: " + e.getMessage());
        }
    }
}
```


### Table Extraction with Detailed Access

Extract tables with cell-level access and Markdown representation:

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import dev.kreuzberg.Table;
import dev.kreuzberg.KreuzbergException;
import dev.kreuzberg.config.ExtractionConfig;
import java.io.IOException;

public class TableExtractionExample {
    public static void main(String[] args) {
        try {
            ExtractionConfig config = ExtractionConfig.builder()
                .extractTables(true)
                .build();

            ExtractionResult result = Kreuzberg.extractFile("spreadsheet.xlsx", config);

            for (Table table : result.getTables()) {
                System.out.println("Table on page: " + table.pageNumber());
                System.out.println("Dimensions: " + table.getRowCount() + "x" + table.getColumnCount());

                // Access individual cells
                for (int row = 0; row < table.getRowCount(); row++) {
                    for (int col = 0; col < table.getColumnCount(); col++) {
                        String cell = table.getCell(row, col);
                        System.out.print(cell + " | ");
                    }
                    System.out.println();
                }

                // Get headers (first row)
                if (table.getRowCount() > 0) {
                    System.out.println("\nHeaders: " + table.cells().get(0));
                }

                // Get Markdown representation
                System.out.println("\nMarkdown:\n" + table.markdown());
            }
        } catch (IOException | KreuzbergException e) {
            System.err.println("Table extraction failed: " + e.getMessage());
        }
    }
}
```

### Image Extraction with Metadata

Extract images and access their metadata:

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import dev.kreuzberg.ExtractedImage;
import dev.kreuzberg.KreuzbergException;
import dev.kreuzberg.config.ExtractionConfig;
import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

public class ImageExtractionExample {
    public static void main(String[] args) {
        try {
            ExtractionConfig config = ExtractionConfig.builder()
                .extractImages(true)
                .build();

            ExtractionResult result = Kreuzberg.extractFile("document.pdf", config);

            for (ExtractedImage image : result.getImages()) {
                System.out.println("Image Index: " + image.getImageIndex());
                System.out.println("Format: " + image.getFormat());

                // Access metadata
                if (image.getPageNumber().isPresent()) {
                    System.out.println("Page: " + image.getPageNumber().get());
                }
                if (image.getWidth().isPresent() && image.getHeight().isPresent()) {
                    System.out.println("Dimensions: " + image.getWidth().get()
                        + "x" + image.getHeight().get());
                }
                if (image.getColorspace().isPresent()) {
                    System.out.println("Colorspace: " + image.getColorspace().get());
                }
                if (image.getDescription().isPresent()) {
                    System.out.println("Description: " + image.getDescription().get());
                }

                // Save image to file
                byte[] imageData = image.getData();
                Path outputPath = Path.of("image_" + image.getImageIndex() + "." + image.getFormat());
                Files.write(outputPath, imageData);
                System.out.println("Saved to: " + outputPath);
            }
        } catch (IOException | KreuzbergException e) {
            System.err.println("Image extraction failed: " + e.getMessage());
        }
    }
}
```

### Pages Extraction

Track and access page information and boundaries:

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import dev.kreuzberg.PageStructure;
import dev.kreuzberg.KreuzbergException;
import dev.kreuzberg.config.ExtractionConfig;
import dev.kreuzberg.config.PageConfig;
import java.io.IOException;
import java.util.Optional;

public class PagesExtractionExample {
    public static void main(String[] args) {
        try {
            // Extract with page tracking enabled
            ExtractionConfig config = ExtractionConfig.builder()
                .page(PageConfig.builder()
                    .extractPages(true)
                    .insertPageMarkers(true)
                    .markerFormat("\n\n<!-- PAGE {page_num} -->\n\n")
                    .build())
                .build();

            ExtractionResult result = Kreuzberg.extractFile("document.pdf", config);

            System.out.println("Total pages: " + result.getPageCount());
            System.out.println("Content length: " + result.getContent().length());

            // Access page structure if available
            Optional<PageStructure> pageStructure = result.getPageStructure();
            if (pageStructure.isPresent()) {
                PageStructure pages = pageStructure.get();
                System.out.println("Page structure available");
                // Page markers are inserted in the content as: <!-- PAGE 1 -->, <!-- PAGE 2 -->, etc.
            }
        } catch (IOException | KreuzbergException e) {
            System.err.println("Pages extraction failed: " + e.getMessage());
        }
    }
}
```

### PostProcessor Registration

Create and register custom post-processors to transform extraction results:

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import dev.kreuzberg.PostProcessor;
import dev.kreuzberg.ProcessingStage;
import dev.kreuzberg.KreuzbergException;
import java.io.IOException;

public class PostProcessorExample {
    public static void main(String[] args) {
        try {
            // Create a custom post-processor for trimming whitespace
            PostProcessor trimProcessor = new PostProcessor() {
                @Override
                public ExtractionResult process(ExtractionResult result) {
                    return result.withContent(result.getContent().trim());
                }

                @Override
                public ProcessingStage processingStage() {
                    return ProcessingStage.FINAL;
                }

                @Override
                public int priority() {
                    return 100;
                }
            };

            // Register the post-processor
            Kreuzberg.registerPostProcessor("trim-whitespace", trimProcessor);

            // Use it during extraction
            ExtractionResult result = Kreuzberg.extractFile("document.pdf");
            System.out.println("Content (trimmed): " + result.getContent());

            // List registered processors
            var processors = Kreuzberg.listPostProcessors();
            System.out.println("Registered processors: " + processors);

            // Unregister when done
            Kreuzberg.unregisterPostProcessor("trim-whitespace");
        } catch (IOException | KreuzbergException e) {
            System.err.println("PostProcessor registration failed: " + e.getMessage());
        }
    }
}
```

### Validator Registration

Create and register custom validators to check extraction quality:

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import dev.kreuzberg.Validator;
import dev.kreuzberg.ValidationException;
import dev.kreuzberg.KreuzbergException;
import java.io.IOException;

public class ValidatorExample {
    public static void main(String[] args) {
        try {
            // Create a custom validator for minimum content length
            Validator minLengthValidator = new Validator() {
                private static final int MIN_CHARS = 100;

                @Override
                public void validate(ExtractionResult result) throws ValidationException {
                    if (result.getContent().length() < MIN_CHARS) {
                        throw new ValidationException(
                            "Content too short: " + result.getContent().length()
                            + " chars (minimum: " + MIN_CHARS + ")");
                    }
                }

                @Override
                public int priority() {
                    return 50;
                }
            };

            // Create a validator for table extraction
            Validator tableValidator = result -> {
                if (result.getTables().isEmpty()) {
                    throw new ValidationException("No tables found in document");
                }
            };

            // Register validators
            Kreuzberg.registerValidator("min-length", minLengthValidator);
            Kreuzberg.registerValidator("has-tables", tableValidator, 100);

            // Extract and validate
            ExtractionResult result = Kreuzberg.extractFile("document.pdf");

            try {
                minLengthValidator.validate(result);
                System.out.println("Content length validation passed");
            } catch (ValidationException e) {
                System.err.println("Validation failed: " + e.getMessage());
            }

            // Clean up
            Kreuzberg.unregisterValidator("min-length");
            Kreuzberg.unregisterValidator("has-tables");
        } catch (IOException | KreuzbergException e) {
            System.err.println("Validator registration failed: " + e.getMessage());
        }
    }
}
```

### Plugin System Overview

Kreuzberg provides a comprehensive plugin architecture with post-processors and validators:

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import dev.kreuzberg.PostProcessor;
import dev.kreuzberg.Validator;
import dev.kreuzberg.ValidationException;
import dev.kreuzberg.ProcessingStage;
import dev.kreuzberg.KreuzbergException;
import java.io.IOException;
import java.util.Map;
import java.util.HashMap;

public class PluginSystemExample {
    public static void main(String[] args) {
        try {
            // Create a preprocessing validator
            Validator fileTypeValidator = result -> {
                String mimeType = result.getMimeType();
                if (!mimeType.startsWith("application/pdf")) {
                    throw new ValidationException("Expected PDF, got: " + mimeType);
                }
            };

            // Create a post-processor that enriches metadata
            PostProcessor metadataEnricher = new PostProcessor() {
                @Override
                public ExtractionResult process(ExtractionResult result) {
                    Map<String, Object> enrichedMetadata = new HashMap<>(result.getMetadata());
                    enrichedMetadata.put("processed_at", System.currentTimeMillis());
                    enrichedMetadata.put("table_count", result.getTables().size());
                    enrichedMetadata.put("image_count", result.getImages().size());
                    return result.withMetadata(enrichedMetadata);
                }

                @Override
                public ProcessingStage processingStage() {
                    return ProcessingStage.FINAL;
                }
            };

            // Create a cleaning post-processor
            PostProcessor contentCleaner = result -> {
                String cleaned = result.getContent()
                    .replaceAll("\\s+", " ")  // Normalize whitespace
                    .trim();
                return result.withContent(cleaned);
            };

            // Register the pipeline
            Kreuzberg.registerValidator("file-type-check", fileTypeValidator, 100);
            Kreuzberg.registerPostProcessor("metadata-enricher", metadataEnricher, 50);
            Kreuzberg.registerPostProcessor("content-cleaner", contentCleaner, 75);

            // Extract with the plugin pipeline
            ExtractionResult result = Kreuzberg.extractFile("document.pdf");

            System.out.println("Extraction completed with plugins:");
            System.out.println("- Content length: " + result.getContent().length());
            System.out.println("- Tables: " + result.getTables().size());
            System.out.println("- Images: " + result.getImages().size());
            System.out.println("- Processing time: "
                + result.getMetadata().get("processed_at"));

            // List all registered plugins
            System.out.println("\nRegistered post-processors: " + Kreuzberg.listPostProcessors());

            // Clean up
            Kreuzberg.clearPostProcessors();
            Kreuzberg.unregisterValidator("file-type-check");
        } catch (IOException | KreuzbergException e) {
            System.err.println("Plugin system error: " + e.getMessage());
        }
    }
}
```

## Plugin System

Kreuzberg supports extensible post-processing plugins for custom text transformation and filtering.

For detailed plugin documentation, visit [Plugin System Guide](https://kreuzberg.dev/plugins/).

## Embeddings Support

Kreuzberg provides vector embedding support for semantic search and similarity analysis using ONNX Runtime models.

### Embeddings Configuration

Embeddings are configured using the `EmbeddingConfig` class, which provides type-safe configuration for embedding generation:

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import dev.kreuzberg.KreuzbergException;
import dev.kreuzberg.config.ExtractionConfig;
import dev.kreuzberg.config.EmbeddingConfig;
import java.io.IOException;

public class EmbeddingsExample {
    public static void main(String[] args) {
        try {
            // Configure embeddings with EmbeddingConfig
            EmbeddingConfig embeddingConfig = EmbeddingConfig.builder()
                .model("all-MiniLM-L6-v2")
                .dimensions(384)
                .normalize(true)
                .batchSize(64)
                .useCache(true)
                .build();

            ExtractionConfig config = ExtractionConfig.builder()
                .embedding(embeddingConfig)
                .build();

            ExtractionResult result = Kreuzberg.extractFile("document.pdf", config);
            System.out.println("Extracted with embeddings");
        } catch (IOException | KreuzbergException e) {
            System.err.println("Embeddings extraction failed: " + e.getMessage());
        }
    }
}
```

### Available Embedding Models

Common embedding models supported by Kreuzberg:

| Model | Dimensions | Use Case |
|-------|------------|----------|
| `all-MiniLM-L6-v2` | 384 | Lightweight, fast, good for general purpose |
| `all-MiniLM-L12-v2` | 384 | Balanced quality/speed |
| `all-mpnet-base-v2` | 768 | High quality embeddings |
| `paraphrase-MiniLM-L6-v2` | 384 | Optimized for semantic similarity |
| `multi-qa-MiniLM-L6-cos-v1` | 384 | Optimized for Q&A and search |

### Embedding Presets

List and use embedding presets for quick configuration:

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.EmbeddingPreset;
import dev.kreuzberg.KreuzbergException;
import java.util.List;
import java.util.Optional;

public class EmbeddingPresetsExample {
    public static void main(String[] args) {
        try {
            // List available embedding presets
            List<String> presets = Kreuzberg.listEmbeddingPresets();
            System.out.println("Available embedding presets:");
            for (String preset : presets) {
                System.out.println("  - " + preset);
            }

            // Get details about a specific preset
            Optional<EmbeddingPreset> preset = Kreuzberg.getEmbeddingPreset("multilingual-e5-small");
            if (preset.isPresent()) {
                EmbeddingPreset embeddingPreset = preset.get();
                System.out.println("Preset: " + embeddingPreset.getName());
                System.out.println("Model: " + embeddingPreset.getModel());
                System.out.println("Dimensions: " + embeddingPreset.getDimensions());
            }
        } catch (KreuzbergException e) {
            System.err.println("Failed to list embeddings: " + e.getMessage());
        }
    }
}
```

### Advanced Embedding Configuration

Configure embeddings for specific use cases:

```java
// Lightweight model for memory-constrained environments
EmbeddingConfig lightweightConfig = EmbeddingConfig.builder()
    .model("all-MiniLM-L6-v2")
    .dimensions(384)
    .batchSize(16)
    .normalize(true)
    .useCache(true)
    .build();

// High-quality model with download progress
EmbeddingConfig highQualityConfig = EmbeddingConfig.builder()
    .model("all-mpnet-base-v2")
    .dimensions(768)
    .batchSize(32)
    .normalize(true)
    .showDownloadProgress(true)
    .cacheDir("/custom/cache/path")
    .build();

// Q&A optimized configuration
EmbeddingConfig qaConfig = EmbeddingConfig.builder()
    .model("multi-qa-MiniLM-L6-cos-v1")
    .dimensions(384)
    .batchSize(128)
    .normalize(true)
    .useCache(true)
    .build();
```

**[Embeddings Guide](https://kreuzberg.dev/features/#embeddings)**

## Batch Processing

Process multiple documents efficiently:

```java
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.ExtractionResult;
import dev.kreuzberg.KreuzbergException;
import java.io.IOException;
import java.util.Arrays;
import java.util.List;

try {
    List<String> files = Arrays.asList("doc1.pdf", "doc2.docx", "doc3.pptx");

    List<ExtractionResult> results = Kreuzberg.batchExtractFiles(files, null);

    for (int i = 0; i < results.size(); i++) {
        ExtractionResult result = results.get(i);
        System.out.println("File " + (i + 1) + ": " + result.getContent().length() + " characters");
    }
} catch (IOException | KreuzbergException e) {
    e.printStackTrace();
}
```

## Configuration

For advanced configuration options including language detection, table extraction, OCR settings, and more:

**[Configuration Guide](https://kreuzberg.dev/configuration/)**

## Documentation

- **[Official Documentation](https://kreuzberg.dev/)**
- **[API Reference](https://kreuzberg.dev/reference/api-java/)**
- **[Examples & Guides](https://kreuzberg.dev/guides/)**

## Troubleshooting

For common issues and solutions, visit [Troubleshooting Guide](https://kreuzberg.dev/troubleshooting/).

## Contributing

Contributions are welcome! See [Contributing Guide](https://github.com/kreuzberg-dev/kreuzberg/blob/main/CONTRIBUTING.md).

## License

MIT License - see LICENSE file for details.

## Support

- **Discord Community**: [Join our Discord](https://discord.gg/pXxagNK2zN)
- **GitHub Issues**: [Report bugs](https://github.com/kreuzberg-dev/kreuzberg/issues)
- **Discussions**: [Ask questions](https://github.com/kreuzberg-dev/kreuzberg/discussions)
