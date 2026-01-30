<?php

declare(strict_types=1);

// Auto-generated tests for contract fixtures.

namespace E2EPhp\Tests;

use E2EPhp\Helpers;
use Kreuzberg\Kreuzberg;
use PHPUnit\Framework\TestCase;

class ContractTest extends TestCase
{
    /**
     * Tests async batch bytes extraction API (batch_extract_bytes)
     */
    public function test_api_batch_bytes_async(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping api_batch_bytes_async: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $bytes = file_get_contents($documentPath);
        $mimeType = Kreuzberg::detectMimeType($bytes);
        $results = $kreuzberg->batchExtractBytes([$bytes], [$mimeType]);
        $result = $results[0];

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
        Helpers::assertContentContainsAny($result, ['May 5, 2023', 'Mallori']);
    }

    /**
     * Tests sync batch bytes extraction API (batch_extract_bytes_sync)
     */
    public function test_api_batch_bytes_sync(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping api_batch_bytes_sync: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $bytes = file_get_contents($documentPath);
        $mimeType = Kreuzberg::detectMimeType($bytes);
        $results = $kreuzberg->batchExtractBytes([$bytes], [$mimeType]);
        $result = $results[0];

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
        Helpers::assertContentContainsAny($result, ['May 5, 2023', 'Mallori']);
    }

    /**
     * Tests async batch file extraction API (batch_extract_file)
     */
    public function test_api_batch_file_async(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping api_batch_file_async: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $results = $kreuzberg->batchExtractFiles([$documentPath]);
        $result = $results[0];

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
        Helpers::assertContentContainsAny($result, ['May 5, 2023', 'Mallori']);
    }

    /**
     * Tests sync batch file extraction API (batch_extract_file_sync)
     */
    public function test_api_batch_file_sync(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping api_batch_file_sync: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $results = $kreuzberg->batchExtractFiles([$documentPath]);
        $result = $results[0];

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
        Helpers::assertContentContainsAny($result, ['May 5, 2023', 'Mallori']);
    }

    /**
     * Tests async bytes extraction API (extract_bytes)
     */
    public function test_api_extract_bytes_async(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping api_extract_bytes_async: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $bytes = file_get_contents($documentPath);
        $mimeType = Kreuzberg::detectMimeType($bytes);
        $result = $kreuzberg->extractBytes($bytes, $mimeType);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
        Helpers::assertContentContainsAny($result, ['May 5, 2023', 'Mallori']);
    }

    /**
     * Tests sync bytes extraction API (extract_bytes_sync)
     */
    public function test_api_extract_bytes_sync(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping api_extract_bytes_sync: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $bytes = file_get_contents($documentPath);
        $mimeType = Kreuzberg::detectMimeType($bytes);
        $result = $kreuzberg->extractBytes($bytes, $mimeType);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
        Helpers::assertContentContainsAny($result, ['May 5, 2023', 'Mallori']);
    }

    /**
     * Tests async file extraction API (extract_file)
     */
    public function test_api_extract_file_async(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping api_extract_file_async: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
        Helpers::assertContentContainsAny($result, ['May 5, 2023', 'Mallori']);
    }

    /**
     * Tests sync file extraction API (extract_file_sync)
     */
    public function test_api_extract_file_sync(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping api_extract_file_sync: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
        Helpers::assertContentContainsAny($result, ['May 5, 2023', 'Mallori']);
    }

    /**
     * Tests chunking configuration with chunk assertions
     */
    public function test_config_chunking(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping config_chunking: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(['chunking' => ['max_chars' => 500, 'max_overlap' => 50]]);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
        Helpers::assertChunks($result, 1, null, true, null);
    }

    /**
     * Tests force_ocr configuration option
     */
    public function test_config_force_ocr(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping config_force_ocr: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(['force_ocr' => true]);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 5);
    }

    /**
     * Tests image extraction configuration with image assertions
     */
    public function test_config_images(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/embedded_images_tables.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping config_images: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(['images' => ['extract_images' => true]]);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertImages($result, 1, null, null);
    }

    /**
     * Tests language detection configuration
     */
    public function test_config_language_detection(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping config_language_detection: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(['language_detection' => ['enabled' => true]]);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
        Helpers::assertDetectedLanguages($result, ['eng'], 0.5);
    }

    /**
     * Tests page configuration with page assertions
     */
    public function test_config_pages(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/multi_page.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping config_pages: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(['pages' => ['end' => 3, 'start' => 1]]);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
    }

    /**
     * Tests use_cache=false configuration option
     */
    public function test_config_use_cache_false(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping config_use_cache_false: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(['use_cache' => false]);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
    }

    /**
     * Tests Djot output format
     */
    public function test_output_format_djot(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping output_format_djot: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(['output_format' => 'djot']);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
    }

    /**
     * Tests HTML output format
     */
    public function test_output_format_html(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping output_format_html: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(['output_format' => 'html']);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
    }

    /**
     * Tests Markdown output format
     */
    public function test_output_format_markdown(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping output_format_markdown: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(['output_format' => 'markdown']);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
    }

    /**
     * Tests Plain output format
     */
    public function test_output_format_plain(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping output_format_plain: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(['output_format' => 'plain']);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
    }

    /**
     * Tests ElementBased result format with element assertions
     */
    public function test_result_format_element_based(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping result_format_element_based: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(['result_format' => 'element_based']);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertElements($result, 1, null);
    }

    /**
     * Tests Unified result format (default)
     */
    public function test_result_format_unified(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping result_format_unified: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(['result_format' => 'unified']);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 10);
    }

}
