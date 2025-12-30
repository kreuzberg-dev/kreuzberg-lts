<?php

declare(strict_types=1);

namespace Kreuzberg;

use Kreuzberg\Config\ExtractionConfig;
use Kreuzberg\Exceptions\KreuzbergException;
use Kreuzberg\Types\ExtractionResult;

/**
 * Main Kreuzberg API class for document extraction.
 *
 * Provides high-performance document intelligence powered by a Rust core.
 * Extract text, metadata, and structured data from PDFs, Office documents,
 * images, and 56+ file formats.
 *
 * @example
 * ```php
 * use Kreuzberg\Kreuzberg;
 * use Kreuzberg\Config\ExtractionConfig;
 * use Kreuzberg\Config\OcrConfig;
 *
 * $kreuzberg = new Kreuzberg();
 * $result = $kreuzberg->extractFile('document.pdf');
 * echo $result->content;
 *
 * // With configuration
 * $config = new ExtractionConfig(
 *     ocr: new OcrConfig(backend: 'tesseract', language: 'eng')
 * );
 * $kreuzberg = new Kreuzberg($config);
 * $result = $kreuzberg->extractFile('scanned.pdf');
 * ```
 */
final readonly class Kreuzberg
{
    public const VERSION = '4.0.0-rc.20';

    public function __construct(
        private ?ExtractionConfig $defaultConfig = null,
    ) {
    }

    /**
     * Extract content from a file.
     *
     * @param string $filePath Path to the file to extract
     * @param string|null $mimeType Optional MIME type hint (auto-detected if null)
     * @param ExtractionConfig|null $config Extraction configuration (uses constructor config if null)
     * @return ExtractionResult Extraction result with content, metadata, and tables
     * @throws KreuzbergException If extraction fails
     */
    public function extractFile(
        string $filePath,
        ?string $mimeType = null,
        ?ExtractionConfig $config = null,
    ): ExtractionResult {
        $config ??= $this->defaultConfig ?? new ExtractionConfig();

        return \Kreuzberg\extract_file($filePath, $mimeType, $config);
    }

    /**
     * Extract content from bytes.
     *
     * @param string $data File content as bytes
     * @param string $mimeType MIME type of the data (required for format detection)
     * @param ExtractionConfig|null $config Extraction configuration (uses constructor config if null)
     * @return ExtractionResult Extraction result with content, metadata, and tables
     * @throws KreuzbergException If extraction fails
     */
    public function extractBytes(
        string $data,
        string $mimeType,
        ?ExtractionConfig $config = null,
    ): ExtractionResult {
        $config ??= $this->defaultConfig ?? new ExtractionConfig();

        return \Kreuzberg\extract_bytes($data, $mimeType, $config);
    }

    /**
     * Extract content from multiple files in parallel.
     *
     * @param array<string> $paths List of file paths
     * @param ExtractionConfig|null $config Extraction configuration (uses constructor config if null)
     * @return array<ExtractionResult> List of extraction results (one per file)
     * @throws KreuzbergException If extraction fails
     */
    public function batchExtractFiles(
        array $paths,
        ?ExtractionConfig $config = null,
    ): array {
        $config ??= $this->defaultConfig ?? new ExtractionConfig();

        return \Kreuzberg\batch_extract_files($paths, $config);
    }

    /**
     * Extract content from multiple byte arrays in parallel.
     *
     * @param array<string> $dataList List of file contents as bytes
     * @param array<string> $mimeTypes List of MIME types (one per data item)
     * @param ExtractionConfig|null $config Extraction configuration (uses constructor config if null)
     * @return array<ExtractionResult> List of extraction results (one per data item)
     * @throws KreuzbergException If extraction fails
     */
    public function batchExtractBytes(
        array $dataList,
        array $mimeTypes,
        ?ExtractionConfig $config = null,
    ): array {
        $config ??= $this->defaultConfig ?? new ExtractionConfig();

        return \Kreuzberg\batch_extract_bytes($dataList, $mimeTypes, $config);
    }

    /**
     * Extract content from a file (static synchronous method).
     *
     * @param string $filePath Path to the file to extract
     * @param string|null $mimeType Optional MIME type hint (auto-detected if null)
     * @param ExtractionConfig|null $config Extraction configuration (uses defaults if null)
     * @return ExtractionResult Extraction result with content, metadata, and tables
     * @throws KreuzbergException If extraction fails
     */
    public static function extractFileSync(
        string $filePath,
        ?string $mimeType = null,
        ?ExtractionConfig $config = null,
    ): ExtractionResult {
        $config ??= new ExtractionConfig();

        return \Kreuzberg\extract_file($filePath, $mimeType, $config);
    }

    /**
     * Extract content from bytes (static synchronous method).
     *
     * @param string $data File content as bytes
     * @param string $mimeType MIME type of the data (required for format detection)
     * @param ExtractionConfig|null $config Extraction configuration (uses defaults if null)
     * @return ExtractionResult Extraction result with content, metadata, and tables
     * @throws KreuzbergException If extraction fails
     */
    public static function extractBytesSync(
        string $data,
        string $mimeType,
        ?ExtractionConfig $config = null,
    ): ExtractionResult {
        $config ??= new ExtractionConfig();

        return \Kreuzberg\extract_bytes($data, $mimeType, $config);
    }

    /**
     * Extract content from multiple files in parallel (static synchronous method).
     *
     * @param array<string> $paths List of file paths
     * @param ExtractionConfig|null $config Extraction configuration (uses defaults if null)
     * @return array<ExtractionResult> List of extraction results (one per file)
     * @throws KreuzbergException If extraction fails
     */
    public static function batchExtractFilesSync(
        array $paths,
        ?ExtractionConfig $config = null,
    ): array {
        $config ??= new ExtractionConfig();

        return \Kreuzberg\batch_extract_files($paths, $config);
    }

    /**
     * Extract content from multiple byte arrays in parallel (static synchronous method).
     *
     * @param array<string> $dataList List of file contents as bytes
     * @param array<string> $mimeTypes List of MIME types (one per data item)
     * @param ExtractionConfig|null $config Extraction configuration (uses defaults if null)
     * @return array<ExtractionResult> List of extraction results (one per data item)
     * @throws KreuzbergException If extraction fails
     */
    public static function batchExtractBytesSync(
        array $dataList,
        array $mimeTypes,
        ?ExtractionConfig $config = null,
    ): array {
        $config ??= new ExtractionConfig();

        return \Kreuzberg\batch_extract_bytes($dataList, $mimeTypes, $config);
    }

    /**
     * Get the library version.
     */
    public static function version(): string
    {
        return self::VERSION;
    }
}
