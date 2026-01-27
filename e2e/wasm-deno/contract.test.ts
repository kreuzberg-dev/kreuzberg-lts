// Auto-generated tests for contract fixtures.
// Run with: deno test --allow-read

import { assertions, buildConfig, extractBytes, initWasm, resolveDocument, shouldSkipFixture } from "./helpers.ts";
import type { ExtractionResult } from "./helpers.ts";

// Initialize WASM module once at module load time
await initWasm();

Deno.test("api_batch_bytes_async", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      // Batch async extraction - WASM simulates with single extraction
      const results = [await extractBytes(documentBytes, "application/octet-stream", config)];
      result = results[0];
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

Deno.test("api_batch_bytes_sync", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      // Batch sync extraction - WASM simulates with single extraction
      const results = [await extractBytes(documentBytes, "application/octet-stream", config)];
      result = results[0];
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

Deno.test("api_batch_file_async", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      // Batch async extraction - WASM simulates with single extraction
      const results = [await extractBytes(documentBytes, "application/octet-stream", config)];
      result = results[0];
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

Deno.test("api_batch_file_sync", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      // Batch sync extraction - WASM simulates with single extraction
      const results = [await extractBytes(documentBytes, "application/octet-stream", config)];
      result = results[0];
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

Deno.test("api_extract_bytes_async", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      // Async bytes extraction - native WASM pattern
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

Deno.test("api_extract_bytes_sync", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      // Sync bytes extraction - WASM uses extractBytes with Uint8Array
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

Deno.test("api_extract_file_async", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      // Async file extraction - native WASM pattern
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

Deno.test("api_extract_file_sync", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      // Sync file extraction - WASM uses extractBytes with pre-read bytes
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

Deno.test("config_chunking", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig({"chunking":{"max_chars":500,"overlap":50}});
    let result: ExtractionResult | null = null;
    try {
      // Sync file extraction - WASM uses extractBytes with pre-read bytes
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

Deno.test("config_force_ocr", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig({"force_ocr":true});
    let result: ExtractionResult | null = null;
    try {
      // Sync file extraction - WASM uses extractBytes with pre-read bytes
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

Deno.test("config_images", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/embedded_images_tables.pdf");
    const config = buildConfig({"images":{"extract":true,"format":"png"}});
    let result: ExtractionResult | null = null;
    try {
      // Sync file extraction - WASM uses extractBytes with pre-read bytes
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

Deno.test("config_language_detection", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig({"language_detection":{"enabled":true}});
    let result: ExtractionResult | null = null;
    try {
      // Sync file extraction - WASM uses extractBytes with pre-read bytes
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

Deno.test("config_pages", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/multi_page.pdf");
    const config = buildConfig({"pages":{"end":3,"start":1}});
    let result: ExtractionResult | null = null;
    try {
      // Sync file extraction - WASM uses extractBytes with pre-read bytes
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

Deno.test("config_use_cache_false", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig({"use_cache":false});
    let result: ExtractionResult | null = null;
    try {
      // Sync file extraction - WASM uses extractBytes with pre-read bytes
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

Deno.test("output_format_djot", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig({"output_format":"djot"});
    let result: ExtractionResult | null = null;
    try {
      // Sync file extraction - WASM uses extractBytes with pre-read bytes
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

Deno.test("output_format_html", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig({"output_format":"html"});
    let result: ExtractionResult | null = null;
    try {
      // Sync file extraction - WASM uses extractBytes with pre-read bytes
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

Deno.test("output_format_markdown", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig({"output_format":"markdown"});
    let result: ExtractionResult | null = null;
    try {
      // Sync file extraction - WASM uses extractBytes with pre-read bytes
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

Deno.test("output_format_plain", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig({"output_format":"plain"});
    let result: ExtractionResult | null = null;
    try {
      // Sync file extraction - WASM uses extractBytes with pre-read bytes
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

Deno.test("result_format_element_based", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig({"result_format":"element_based"});
    let result: ExtractionResult | null = null;
    try {
      // Sync file extraction - WASM uses extractBytes with pre-read bytes
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

Deno.test("result_format_unified", { permissions: { read: true } }, async () => {
    const documentBytes = await resolveDocument("pdfs/fake_memo.pdf");
    const config = buildConfig({"result_format":"unified"});
    let result: ExtractionResult | null = null;
    try {
      // Sync file extraction - WASM uses extractBytes with pre-read bytes
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
