<?php

declare(strict_types=1);

// Auto-generated tests for office fixtures.

namespace E2EPhp\Tests;

use E2EPhp\Helpers;
use Kreuzberg\Kreuzberg;
use PHPUnit\Framework\TestCase;

class OfficeTest extends TestCase
{
    /**
     * Legacy .doc document conversion via LibreOffice.
     */
    public function test_office_doc_legacy(): void
    {
        $documentPath = Helpers::resolveDocument('legacy_office/unit_test_lists.doc');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_doc_legacy: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/msword']);
        Helpers::assertMinContentLength($result, 20);
    }

    /**
     * DOCX document extraction baseline.
     */
    public function test_office_docx_basic(): void
    {
        $documentPath = Helpers::resolveDocument('office/document.docx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_docx_basic: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.wordprocessingml.document']);
        Helpers::assertMinContentLength($result, 10);
    }

    /**
     * DOCX file containing equations to validate math extraction.
     */
    public function test_office_docx_equations(): void
    {
        $documentPath = Helpers::resolveDocument('documents/equations.docx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_docx_equations: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.wordprocessingml.document']);
        Helpers::assertMinContentLength($result, 20);
    }

    /**
     * Simple DOCX document to verify baseline extraction.
     */
    public function test_office_docx_fake(): void
    {
        $documentPath = Helpers::resolveDocument('documents/fake.docx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_docx_fake: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.wordprocessingml.document']);
        Helpers::assertMinContentLength($result, 20);
    }

    /**
     * DOCX document heavy on formatting for style preservation.
     */
    public function test_office_docx_formatting(): void
    {
        $documentPath = Helpers::resolveDocument('documents/unit_test_formatting.docx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_docx_formatting: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.wordprocessingml.document']);
        Helpers::assertMinContentLength($result, 20);
    }

    /**
     * DOCX document with complex headers.
     */
    public function test_office_docx_headers(): void
    {
        $documentPath = Helpers::resolveDocument('documents/unit_test_headers.docx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_docx_headers: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.wordprocessingml.document']);
        Helpers::assertMinContentLength($result, 20);
    }

    /**
     * DOCX document emphasizing list formatting.
     */
    public function test_office_docx_lists(): void
    {
        $documentPath = Helpers::resolveDocument('documents/unit_test_lists.docx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_docx_lists: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.wordprocessingml.document']);
        Helpers::assertMinContentLength($result, 20);
    }

    /**
     * DOCX document containing tables for table-aware extraction.
     */
    public function test_office_docx_tables(): void
    {
        $documentPath = Helpers::resolveDocument('documents/docx_tables.docx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_docx_tables: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.wordprocessingml.document']);
        Helpers::assertMinContentLength($result, 50);
        Helpers::assertContentContainsAll($result, ['Simple uniform table', 'Nested Table', 'merged cells', 'Header Col']);
        Helpers::assertTableCount($result, 1, null);
    }

    /**
     * PPSX (PowerPoint Show) files should extract slides content identical to PPTX. GitHub Issue #321 Bug 2.
     */
    public function test_office_ppsx_slideshow(): void
    {
        $documentPath = Helpers::resolveDocument('presentations/sample.ppsx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_ppsx_slideshow: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.presentationml.slideshow']);
        Helpers::assertMinContentLength($result, 10);
    }

    /**
     * Legacy PowerPoint .ppt file requiring LibreOffice conversion.
     */
    public function test_office_ppt_legacy(): void
    {
        $documentPath = Helpers::resolveDocument('legacy_office/simple.ppt');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_ppt_legacy: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.ms-powerpoint']);
        Helpers::assertMinContentLength($result, 10);
    }

    /**
     * PPTX deck should extract slides content.
     */
    public function test_office_pptx_basic(): void
    {
        $documentPath = Helpers::resolveDocument('presentations/simple.pptx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_pptx_basic: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.presentationml.presentation']);
        Helpers::assertMinContentLength($result, 50);
    }

    /**
     * PPTX presentation containing images to ensure metadata extraction.
     */
    public function test_office_pptx_images(): void
    {
        $documentPath = Helpers::resolveDocument('presentations/powerpoint_with_image.pptx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_pptx_images: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.presentationml.presentation']);
        Helpers::assertMinContentLength($result, 20);
    }

    /**
     * Pitch deck PPTX used to validate large slide extraction.
     */
    public function test_office_pptx_pitch_deck(): void
    {
        $documentPath = Helpers::resolveDocument('presentations/pitch_deck_presentation.pptx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_pptx_pitch_deck: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.presentationml.presentation']);
        Helpers::assertMinContentLength($result, 100);
    }

    /**
     * Legacy XLS spreadsheet to ensure backward compatibility.
     */
    public function test_office_xls_legacy(): void
    {
        $documentPath = Helpers::resolveDocument('spreadsheets/test_excel.xls');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_xls_legacy: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.ms-excel']);
        Helpers::assertMinContentLength($result, 10);
    }

    /**
     * XLSX spreadsheet should produce metadata and table content.
     */
    public function test_office_xlsx_basic(): void
    {
        $documentPath = Helpers::resolveDocument('spreadsheets/stanley_cups.xlsx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_xlsx_basic: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.spreadsheetml.sheet']);
        Helpers::assertMinContentLength($result, 100);
        Helpers::assertContentContainsAll($result, ['Team', 'Location', 'Stanley Cups']);
        Helpers::assertTableCount($result, 1, null);
        Helpers::assertMetadataExpectation($result, 'sheet_count', ['gte' => 2]);
        Helpers::assertMetadataExpectation($result, 'sheet_names', ['contains' => ['Stanley Cups']]);
    }

    /**
     * XLSX workbook with multiple sheets.
     */
    public function test_office_xlsx_multi_sheet(): void
    {
        $documentPath = Helpers::resolveDocument('spreadsheets/excel_multi_sheet.xlsx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_xlsx_multi_sheet: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.spreadsheetml.sheet']);
        Helpers::assertMinContentLength($result, 20);
        Helpers::assertMetadataExpectation($result, 'sheet_count', ['gte' => 2]);
    }

    /**
     * Simple XLSX spreadsheet shipped alongside office integration tests.
     */
    public function test_office_xlsx_office_example(): void
    {
        $documentPath = Helpers::resolveDocument('office/excel.xlsx');
        if (!file_exists($documentPath)) {
            $this->markTestSkipped('Skipping office_xlsx_office_example: missing document at ' . $documentPath);
        }

        $config = Helpers::buildConfig(null);

        $kreuzberg = new Kreuzberg($config);
        $result = $kreuzberg->extractFile($documentPath);

        Helpers::assertExpectedMime($result, ['application/vnd.openxmlformats-officedocument.spreadsheetml.sheet']);
        Helpers::assertMinContentLength($result, 10);
    }

}
