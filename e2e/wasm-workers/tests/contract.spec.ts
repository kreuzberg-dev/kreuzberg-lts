// Auto-generated tests for contract fixtures.
// Designed for Cloudflare Workers with Vitest + Miniflare

import { describe, it, expect } from "vitest";
import { extractBytes, batchExtractBytes, batchExtractBytesSync } from "@kreuzberg/wasm";
import { assertions, buildConfig, getFixture, shouldSkipFixture } from "./helpers.js";
import type { ExtractionResult } from "@kreuzberg/wasm";

describe("contract", () => {
    it("api_batch_bytes_async", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig(undefined);
        let result: ExtractionResult | null = null;
        try {
            const results = await batchExtractBytes([{ data: documentBytes, mimeType: "application/octet-stream" }], config);
            result = results[0] ?? null;
        } catch (error) {
            if (shouldSkipFixture(error, "api_batch_bytes_async", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
        assertions.assertContentContainsAny(result, ["May 5, 2023", "Mallori"]);
    });

    it("api_batch_bytes_sync", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig(undefined);
        let result: ExtractionResult | null = null;
        try {
            const results = await batchExtractBytesSync([{ data: documentBytes, mimeType: "application/octet-stream" }], config);
            result = results[0] ?? null;
        } catch (error) {
            if (shouldSkipFixture(error, "api_batch_bytes_sync", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
        assertions.assertContentContainsAny(result, ["May 5, 2023", "Mallori"]);
    });

    it("api_batch_file_async", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig(undefined);
        let result: ExtractionResult | null = null;
        try {
            const results = await batchExtractBytes([{ data: documentBytes, mimeType: "application/octet-stream" }], config);
            result = results[0] ?? null;
        } catch (error) {
            if (shouldSkipFixture(error, "api_batch_file_async", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
        assertions.assertContentContainsAny(result, ["May 5, 2023", "Mallori"]);
    });

    it("api_batch_file_sync", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig(undefined);
        let result: ExtractionResult | null = null;
        try {
            const results = await batchExtractBytesSync([{ data: documentBytes, mimeType: "application/octet-stream" }], config);
            result = results[0] ?? null;
        } catch (error) {
            if (shouldSkipFixture(error, "api_batch_file_sync", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
        assertions.assertContentContainsAny(result, ["May 5, 2023", "Mallori"]);
    });

    it("api_extract_bytes_async", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig(undefined);
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "api_extract_bytes_async", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
        assertions.assertContentContainsAny(result, ["May 5, 2023", "Mallori"]);
    });

    it("api_extract_bytes_sync", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig(undefined);
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "api_extract_bytes_sync", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
        assertions.assertContentContainsAny(result, ["May 5, 2023", "Mallori"]);
    });

    it("api_extract_file_async", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig(undefined);
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "api_extract_file_async", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
        assertions.assertContentContainsAny(result, ["May 5, 2023", "Mallori"]);
    });

    it("api_extract_file_sync", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig(undefined);
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "api_extract_file_sync", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
        assertions.assertContentContainsAny(result, ["May 5, 2023", "Mallori"]);
    });

    it("config_chunking", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig({"chunking":{"max_chars":500,"overlap":50}});
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "config_chunking", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
        assertions.assertChunks(result, 1, null, true, null);
    });

    it("config_force_ocr", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig({"force_ocr":true});
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "config_force_ocr", ["tesseract"], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 5);
    });

    it("config_images", async () => {
        const documentBytes = getFixture("pdfs/embedded_images_tables.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig({"images":{"extract":true,"format":"png"}});
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "config_images", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertImages(result, 1, null, null);
    });

    it("config_language_detection", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig({"language_detection":{"enabled":true}});
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "config_language_detection", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
        assertions.assertDetectedLanguages(result, ["eng"], 0.5);
    });

    it("config_pages", async () => {
        const documentBytes = getFixture("pdfs/multi_page.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig({"pages":{"end":3,"start":1}});
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "config_pages", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
    });

    it("config_use_cache_false", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig({"use_cache":false});
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "config_use_cache_false", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
    });

    it("output_format_djot", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig({"output_format":"djot"});
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "output_format_djot", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
    });

    it("output_format_html", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig({"output_format":"html"});
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "output_format_html", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
    });

    it("output_format_markdown", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig({"output_format":"markdown"});
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "output_format_markdown", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
    });

    it("output_format_plain", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig({"output_format":"plain"});
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "output_format_plain", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
    });

    it("result_format_element_based", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig({"result_format":"element_based"});
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "result_format_element_based", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertElements(result, 1, null);
    });

    it("result_format_unified", async () => {
        const documentBytes = getFixture("pdfs/fake_memo.pdf");
        if (documentBytes === null) {
            console.warn("[SKIP] Test skipped: fixture not available in Cloudflare Workers environment");
            return;
        }

        const config = buildConfig({"result_format":"unified"});
        let result: ExtractionResult | null = null;
        try {
            result = await extractBytes(documentBytes, "application/octet-stream", config);
        } catch (error) {
            if (shouldSkipFixture(error, "result_format_unified", [], undefined)) {
                return;
            }
            throw error;
        }
        if (result === null) {
            return;
        }
        assertions.assertExpectedMime(result, ["application/pdf"]);
        assertions.assertMinContentLength(result, 10);
    });

});
