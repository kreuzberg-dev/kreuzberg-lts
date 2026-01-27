package com.kreuzberg.e2e;

// CHECKSTYLE.OFF: UnusedImports - generated code
// CHECKSTYLE.OFF: LineLength - generated code
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import dev.kreuzberg.BytesWithMime;
import dev.kreuzberg.ExtractionResult;
import dev.kreuzberg.Kreuzberg;
import dev.kreuzberg.config.ExtractionConfig;
import org.junit.jupiter.api.Test;

import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertTrue;
// CHECKSTYLE.ON: UnusedImports
// CHECKSTYLE.ON: LineLength

/** Auto-generated tests for contract fixtures. */
public class ContractTest {
    private static final ObjectMapper MAPPER = new ObjectMapper();

    @Test
    public void apiBatchBytesAsync() throws Exception {
        JsonNode config = null;
        Path documentPath = E2EHelpers.resolveDocument("pdfs/fake_memo.pdf");

        if (true && !Files.exists(documentPath)) {
            String msg = String.format("Skipping api_batch_bytes_async: missing document at %s", documentPath);
            System.err.println(msg);
            org.junit.jupiter.api.Assumptions.assumeTrue(false, msg);
            return;
        }

        byte[] documentBytes = Files.readAllBytes(documentPath);
        String mimeType = Kreuzberg.detectMimeType(documentBytes);
        ExtractionConfig extractionConfig = E2EHelpers.buildConfig(config);
        List<BytesWithMime> items = Arrays.asList(new BytesWithMime(documentBytes, mimeType));
        java.util.concurrent.CompletableFuture<List<ExtractionResult>> future = Kreuzberg.batchExtractBytesAsync(items, extractionConfig);
        List<ExtractionResult> results;
        try {
            results = future.get();
        } catch (java.util.concurrent.ExecutionException e) {
            Throwable cause = e.getCause() != null ? e.getCause() : e;
            if (cause instanceof Exception) {
                String skipReason = E2EHelpers.skipReasonFor((Exception) cause, "api_batch_bytes_async", Collections.emptyList(), null);
                if (skipReason != null) {
                    org.junit.jupiter.api.Assumptions.assumeTrue(false, skipReason);
                    return;
                }
            }
            throw e;
        }

        assertTrue(results.size() == 1, "Expected exactly 1 result from batch extraction");
        ExtractionResult result = results.get(0);

        E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
        E2EHelpers.Assertions.assertMinContentLength(result, 10);
        E2EHelpers.Assertions.assertContentContainsAny(result, Arrays.asList("May 5, 2023", "Mallori"));
    }

    @Test
    public void apiBatchBytesSync() throws Exception {
        JsonNode config = null;
        Path documentPath = E2EHelpers.resolveDocument("pdfs/fake_memo.pdf");

        if (true && !Files.exists(documentPath)) {
            String msg = String.format("Skipping api_batch_bytes_sync: missing document at %s", documentPath);
            System.err.println(msg);
            org.junit.jupiter.api.Assumptions.assumeTrue(false, msg);
            return;
        }

        byte[] documentBytes = Files.readAllBytes(documentPath);
        String mimeType = Kreuzberg.detectMimeType(documentBytes);
        ExtractionConfig extractionConfig = E2EHelpers.buildConfig(config);
        List<BytesWithMime> items = Arrays.asList(new BytesWithMime(documentBytes, mimeType));
        List<ExtractionResult> results;
        try {
            results = Kreuzberg.batchExtractBytes(items, extractionConfig);
        } catch (Exception e) {
            String skipReason = E2EHelpers.skipReasonFor(e, "api_batch_bytes_sync", Collections.emptyList(), null);
            if (skipReason != null) {
                org.junit.jupiter.api.Assumptions.assumeTrue(false, skipReason);
                return;
            }
            throw e;
        }

        assertTrue(results.size() == 1, "Expected exactly 1 result from batch extraction");
        ExtractionResult result = results.get(0);

        E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
        E2EHelpers.Assertions.assertMinContentLength(result, 10);
        E2EHelpers.Assertions.assertContentContainsAny(result, Arrays.asList("May 5, 2023", "Mallori"));
    }

    @Test
    public void apiBatchFileAsync() throws Exception {
        JsonNode config = null;
        Path documentPath = E2EHelpers.resolveDocument("pdfs/fake_memo.pdf");

        if (true && !Files.exists(documentPath)) {
            String msg = String.format("Skipping api_batch_file_async: missing document at %s", documentPath);
            System.err.println(msg);
            org.junit.jupiter.api.Assumptions.assumeTrue(false, msg);
            return;
        }

        ExtractionConfig extractionConfig = E2EHelpers.buildConfig(config);
        List<String> paths = Arrays.asList(documentPath.toString());
        java.util.concurrent.CompletableFuture<List<ExtractionResult>> future = Kreuzberg.batchExtractFilesAsync(paths, extractionConfig);
        List<ExtractionResult> results;
        try {
            results = future.get();
        } catch (java.util.concurrent.ExecutionException e) {
            Throwable cause = e.getCause() != null ? e.getCause() : e;
            if (cause instanceof Exception) {
                String skipReason = E2EHelpers.skipReasonFor((Exception) cause, "api_batch_file_async", Collections.emptyList(), null);
                if (skipReason != null) {
                    org.junit.jupiter.api.Assumptions.assumeTrue(false, skipReason);
                    return;
                }
            }
            throw e;
        }

        assertTrue(results.size() == 1, "Expected exactly 1 result from batch extraction");
        ExtractionResult result = results.get(0);

        E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
        E2EHelpers.Assertions.assertMinContentLength(result, 10);
        E2EHelpers.Assertions.assertContentContainsAny(result, Arrays.asList("May 5, 2023", "Mallori"));
    }

    @Test
    public void apiBatchFileSync() throws Exception {
        JsonNode config = null;
        Path documentPath = E2EHelpers.resolveDocument("pdfs/fake_memo.pdf");

        if (true && !Files.exists(documentPath)) {
            String msg = String.format("Skipping api_batch_file_sync: missing document at %s", documentPath);
            System.err.println(msg);
            org.junit.jupiter.api.Assumptions.assumeTrue(false, msg);
            return;
        }

        ExtractionConfig extractionConfig = E2EHelpers.buildConfig(config);
        List<String> paths = Arrays.asList(documentPath.toString());
        List<ExtractionResult> results;
        try {
            results = Kreuzberg.batchExtractFiles(paths, extractionConfig);
        } catch (Exception e) {
            String skipReason = E2EHelpers.skipReasonFor(e, "api_batch_file_sync", Collections.emptyList(), null);
            if (skipReason != null) {
                org.junit.jupiter.api.Assumptions.assumeTrue(false, skipReason);
                return;
            }
            throw e;
        }

        assertTrue(results.size() == 1, "Expected exactly 1 result from batch extraction");
        ExtractionResult result = results.get(0);

        E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
        E2EHelpers.Assertions.assertMinContentLength(result, 10);
        E2EHelpers.Assertions.assertContentContainsAny(result, Arrays.asList("May 5, 2023", "Mallori"));
    }

    @Test
    public void apiExtractBytesAsync() throws Exception {
        JsonNode config = null;
        Path documentPath = E2EHelpers.resolveDocument("pdfs/fake_memo.pdf");

        if (true && !Files.exists(documentPath)) {
            String msg = String.format("Skipping api_extract_bytes_async: missing document at %s", documentPath);
            System.err.println(msg);
            org.junit.jupiter.api.Assumptions.assumeTrue(false, msg);
            return;
        }

        byte[] documentBytes = Files.readAllBytes(documentPath);
        String mimeType = Kreuzberg.detectMimeType(documentBytes);
        ExtractionConfig extractionConfig = E2EHelpers.buildConfig(config);
        java.util.concurrent.CompletableFuture<ExtractionResult> future = Kreuzberg.extractBytesAsync(documentBytes, mimeType, extractionConfig);
        ExtractionResult result;
        try {
            result = future.get();
        } catch (java.util.concurrent.ExecutionException e) {
            Throwable cause = e.getCause() != null ? e.getCause() : e;
            if (cause instanceof Exception) {
                String skipReason = E2EHelpers.skipReasonFor((Exception) cause, "api_extract_bytes_async", Collections.emptyList(), null);
                if (skipReason != null) {
                    org.junit.jupiter.api.Assumptions.assumeTrue(false, skipReason);
                    return;
                }
            }
            throw e;
        }

        E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
        E2EHelpers.Assertions.assertMinContentLength(result, 10);
        E2EHelpers.Assertions.assertContentContainsAny(result, Arrays.asList("May 5, 2023", "Mallori"));
    }

    @Test
    public void apiExtractBytesSync() throws Exception {
        JsonNode config = null;
        Path documentPath = E2EHelpers.resolveDocument("pdfs/fake_memo.pdf");

        if (true && !Files.exists(documentPath)) {
            String msg = String.format("Skipping api_extract_bytes_sync: missing document at %s", documentPath);
            System.err.println(msg);
            org.junit.jupiter.api.Assumptions.assumeTrue(false, msg);
            return;
        }

        byte[] documentBytes = Files.readAllBytes(documentPath);
        String mimeType = Kreuzberg.detectMimeType(documentBytes);
        ExtractionConfig extractionConfig = E2EHelpers.buildConfig(config);
        ExtractionResult result;
        try {
            result = Kreuzberg.extractBytes(documentBytes, mimeType, extractionConfig);
        } catch (Exception e) {
            String skipReason = E2EHelpers.skipReasonFor(e, "api_extract_bytes_sync", Collections.emptyList(), null);
            if (skipReason != null) {
                org.junit.jupiter.api.Assumptions.assumeTrue(false, skipReason);
                return;
            }
            throw e;
        }

        E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
        E2EHelpers.Assertions.assertMinContentLength(result, 10);
        E2EHelpers.Assertions.assertContentContainsAny(result, Arrays.asList("May 5, 2023", "Mallori"));
    }

    @Test
    public void apiExtractFileAsync() throws Exception {
        JsonNode config = null;
        Path documentPath = E2EHelpers.resolveDocument("pdfs/fake_memo.pdf");

        if (true && !Files.exists(documentPath)) {
            String msg = String.format("Skipping api_extract_file_async: missing document at %s", documentPath);
            System.err.println(msg);
            org.junit.jupiter.api.Assumptions.assumeTrue(false, msg);
            return;
        }

        ExtractionConfig extractionConfig = E2EHelpers.buildConfig(config);
        java.util.concurrent.CompletableFuture<ExtractionResult> future = Kreuzberg.extractFileAsync(documentPath, extractionConfig);
        ExtractionResult result;
        try {
            result = future.get();
        } catch (java.util.concurrent.ExecutionException e) {
            Throwable cause = e.getCause() != null ? e.getCause() : e;
            if (cause instanceof Exception) {
                String skipReason = E2EHelpers.skipReasonFor((Exception) cause, "api_extract_file_async", Collections.emptyList(), null);
                if (skipReason != null) {
                    org.junit.jupiter.api.Assumptions.assumeTrue(false, skipReason);
                    return;
                }
            }
            throw e;
        }

        E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
        E2EHelpers.Assertions.assertMinContentLength(result, 10);
        E2EHelpers.Assertions.assertContentContainsAny(result, Arrays.asList("May 5, 2023", "Mallori"));
    }

    @Test
    public void apiExtractFileSync() throws Exception {
        JsonNode config = null;
        E2EHelpers.runFixture(
            "api_extract_file_sync",
            "pdfs/fake_memo.pdf",
            config,
            Collections.emptyList(),
            null,
            true,
            result -> {
                E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
                E2EHelpers.Assertions.assertMinContentLength(result, 10);
                E2EHelpers.Assertions.assertContentContainsAny(result, Arrays.asList("May 5, 2023", "Mallori"));
            }
        );
    }

    @Test
    public void configChunking() throws Exception {
        JsonNode config = MAPPER.readTree("{\"chunking\":{\"max_chars\":500,\"overlap\":50}}");
        E2EHelpers.runFixture(
            "config_chunking",
            "pdfs/fake_memo.pdf",
            config,
            Collections.emptyList(),
            null,
            true,
            result -> {
                E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
                E2EHelpers.Assertions.assertMinContentLength(result, 10);
                E2EHelpers.Assertions.assertChunks(result, 1, null, true, null);
            }
        );
    }

    @Test
    public void configForceOcr() throws Exception {
        JsonNode config = MAPPER.readTree("{\"force_ocr\":true}");
        E2EHelpers.runFixture(
            "config_force_ocr",
            "pdfs/fake_memo.pdf",
            config,
            Arrays.asList("tesseract"),
            null,
            true,
            result -> {
                E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
                E2EHelpers.Assertions.assertMinContentLength(result, 5);
            }
        );
    }

    @Test
    public void configImages() throws Exception {
        JsonNode config = MAPPER.readTree("{\"images\":{\"extract\":true,\"format\":\"png\"}}");
        E2EHelpers.runFixture(
            "config_images",
            "pdfs/embedded_images_tables.pdf",
            config,
            Collections.emptyList(),
            null,
            true,
            result -> {
                E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
                E2EHelpers.Assertions.assertImages(result, 1, null, null);
            }
        );
    }

    @Test
    public void configLanguageDetection() throws Exception {
        JsonNode config = MAPPER.readTree("{\"language_detection\":{\"enabled\":true}}");
        E2EHelpers.runFixture(
            "config_language_detection",
            "pdfs/fake_memo.pdf",
            config,
            Collections.emptyList(),
            null,
            true,
            result -> {
                E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
                E2EHelpers.Assertions.assertMinContentLength(result, 10);
                E2EHelpers.Assertions.assertDetectedLanguages(result, Arrays.asList("eng"), 0.50);
            }
        );
    }

    @Test
    public void configPages() throws Exception {
        JsonNode config = MAPPER.readTree("{\"pages\":{\"end\":3,\"start\":1}}");
        E2EHelpers.runFixture(
            "config_pages",
            "pdfs/multi_page.pdf",
            config,
            Collections.emptyList(),
            null,
            true,
            result -> {
                E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
                E2EHelpers.Assertions.assertMinContentLength(result, 10);
            }
        );
    }

    @Test
    public void configUseCacheFalse() throws Exception {
        JsonNode config = MAPPER.readTree("{\"use_cache\":false}");
        E2EHelpers.runFixture(
            "config_use_cache_false",
            "pdfs/fake_memo.pdf",
            config,
            Collections.emptyList(),
            null,
            true,
            result -> {
                E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
                E2EHelpers.Assertions.assertMinContentLength(result, 10);
            }
        );
    }

    @Test
    public void outputFormatDjot() throws Exception {
        JsonNode config = MAPPER.readTree("{\"output_format\":\"djot\"}");
        E2EHelpers.runFixture(
            "output_format_djot",
            "pdfs/fake_memo.pdf",
            config,
            Collections.emptyList(),
            null,
            true,
            result -> {
                E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
                E2EHelpers.Assertions.assertMinContentLength(result, 10);
            }
        );
    }

    @Test
    public void outputFormatHtml() throws Exception {
        JsonNode config = MAPPER.readTree("{\"output_format\":\"html\"}");
        E2EHelpers.runFixture(
            "output_format_html",
            "pdfs/fake_memo.pdf",
            config,
            Collections.emptyList(),
            null,
            true,
            result -> {
                E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
                E2EHelpers.Assertions.assertMinContentLength(result, 10);
            }
        );
    }

    @Test
    public void outputFormatMarkdown() throws Exception {
        JsonNode config = MAPPER.readTree("{\"output_format\":\"markdown\"}");
        E2EHelpers.runFixture(
            "output_format_markdown",
            "pdfs/fake_memo.pdf",
            config,
            Collections.emptyList(),
            null,
            true,
            result -> {
                E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
                E2EHelpers.Assertions.assertMinContentLength(result, 10);
            }
        );
    }

    @Test
    public void outputFormatPlain() throws Exception {
        JsonNode config = MAPPER.readTree("{\"output_format\":\"plain\"}");
        E2EHelpers.runFixture(
            "output_format_plain",
            "pdfs/fake_memo.pdf",
            config,
            Collections.emptyList(),
            null,
            true,
            result -> {
                E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
                E2EHelpers.Assertions.assertMinContentLength(result, 10);
            }
        );
    }

    @Test
    public void resultFormatElementBased() throws Exception {
        // Note: The C FFI doesn't expose elements_json field yet.
        // This test is skipped until the FFI is updated to support element-based extraction.
        // See CExtractionResult in kreuzberg-ffi/src/types.rs - needs elements_json field.
        org.junit.jupiter.api.Assumptions.assumeTrue(
            false,
            "Skipping result_format_element_based: C FFI does not expose elements_json field yet"
        );

        JsonNode config = MAPPER.readTree("{\"result_format\":\"element_based\"}");
        E2EHelpers.runFixture(
            "result_format_element_based",
            "pdfs/fake_memo.pdf",
            config,
            Collections.emptyList(),
            null,
            true,
            result -> {
                E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
                E2EHelpers.Assertions.assertElements(result, 1, null);
            }
        );
    }

    @Test
    public void resultFormatUnified() throws Exception {
        JsonNode config = MAPPER.readTree("{\"result_format\":\"unified\"}");
        E2EHelpers.runFixture(
            "result_format_unified",
            "pdfs/fake_memo.pdf",
            config,
            Collections.emptyList(),
            null,
            true,
            result -> {
                E2EHelpers.Assertions.assertExpectedMime(result, Arrays.asList("application/pdf"));
                E2EHelpers.Assertions.assertMinContentLength(result, 10);
            }
        );
    }

}
