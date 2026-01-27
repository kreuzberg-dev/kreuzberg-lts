<?php

declare(strict_types=1);

// Auto-generated tests for smoke fixtures.

namespace E2EPhp\Tests;

use E2EPhp\Helpers;
use Kreuzberg\Kreuzberg;
use PHPUnit\Framework\TestCase;

class SmokeTest extends TestCase
{
    /**
     * Smoke test: DOCX with formatted text
     */
    public function test_smoke_docx_basic(): void
    {
        $documentPath = Helpers::resolveDocument('documents/fake.docx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping smoke_docx_basic: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.wordprocessingml.document']);
        Helpers::assertMinContentLength($result, 20);
        Helpers::assertContentContainsAny($result, ['Lorem', 'ipsum', 'document', 'text']);
    }

    /**
     * Smoke test: HTML converted to Markdown
     */
    public function test_smoke_html_basic(): void
    {
        $documentPath = Helpers::resolveDocument('web/simple_table.html');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping smoke_html_basic: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['text/html']);
        Helpers::assertMinContentLength($result, 10);
        Helpers::assertContentContainsAny($result, ['#', '**', 'simple', 'HTML']);
    }

    /**
     * Smoke test: PNG image (without OCR, metadata only)
     */
    public function test_smoke_image_png(): void
    {
        $documentPath = Helpers::resolveDocument('images/sample.png');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping smoke_image_png: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['image/png']);
        Helpers::assertMetadataExpectation($result, 'format', ['eq' => 'PNG']);
    }

    /**
     * Smoke test: JSON file extraction
     */
    public function test_smoke_json_basic(): void
    {
        $documentPath = Helpers::resolveDocument('data_formats/simple.json');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping smoke_json_basic: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/json']);
        Helpers::assertMinContentLength($result, 5);
    }

    /**
     * Smoke test: PDF with simple text extraction
     */
    public function test_smoke_pdf_basic(): void
    {
        $documentPath = Helpers::resolveDocument('pdfs/fake_memo.pdf');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping smoke_pdf_basic: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/pdf']);
        Helpers::assertMinContentLength($result, 50);
        Helpers::assertContentContainsAny($result, ['May 5, 2023', 'To Whom it May Concern']);
    }

    /**
     * Smoke test: Plain text file
     */
    public function test_smoke_txt_basic(): void
    {
        $documentPath = Helpers::resolveDocument('text/report.txt');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping smoke_txt_basic: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['text/plain']);
        Helpers::assertMinContentLength($result, 5);
    }

    /**
     * Smoke test: XLSX with basic spreadsheet data including tables
     */
    public function test_smoke_xlsx_basic(): void
    {
        $documentPath = Helpers::resolveDocument('spreadsheets/stanley_cups.xlsx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping smoke_xlsx_basic: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.spreadsheetml.sheet']);
        Helpers::assertMinContentLength($result, 100);
        Helpers::assertContentContainsAll($result, ['Team', 'Location', 'Stanley Cups', 'Blues', 'Flyers', 'Maple Leafs', 'STL', 'PHI', 'TOR']);
        Helpers::assertTableCount($result, 1, null);
        Helpers::assertMetadataExpectation($result, 'sheet_count', ['gte' => 2]);
        Helpers::assertMetadataExpectation($result, 'sheet_names', ['contains' => ['Stanley Cups']]);
    }

}
