// Auto-generated tests for contract fixtures.

import { existsSync, readFileSync } from "node:fs";
import { describe, it } from "vitest";
import { assertions, buildConfig, chunkAssertions, resolveDocument, shouldSkipFixture } from "./helpers.js";
import { batchExtractBytes, batchExtractBytesSync, batchExtractFile, batchExtractFileSync, extractBytes, extractBytesSync, extractFile, extractFileSync } from "@kreuzberg/node";
import type { ExtractionResult } from "@kreuzberg/node";

const TEST_TIMEOUT_MS = 60_000;

describe("contract fixtures", () => {
  it("api_batch_bytes_async", async () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping api_batch_bytes_async: missing document at", documentPath);
      return;
    }
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      const fileBytes = readFileSync(documentPath);
      const results = await batchExtractBytes([fileBytes], config);
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
  }, TEST_TIMEOUT_MS);

  it("api_batch_bytes_sync", () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping api_batch_bytes_sync: missing document at", documentPath);
      return;
    }
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      const fileBytes = readFileSync(documentPath);
      const results = batchExtractBytesSync([fileBytes], config);
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
  }, TEST_TIMEOUT_MS);

  it("api_batch_file_async", async () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping api_batch_file_async: missing document at", documentPath);
      return;
    }
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      const results = await batchExtractFile([documentPath], config);
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
  }, TEST_TIMEOUT_MS);

  it("api_batch_file_sync", () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping api_batch_file_sync: missing document at", documentPath);
      return;
    }
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      const results = batchExtractFileSync([documentPath], config);
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
  }, TEST_TIMEOUT_MS);

  it("api_extract_bytes_async", async () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping api_extract_bytes_async: missing document at", documentPath);
      return;
    }
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      const fileBytes = readFileSync(documentPath);
      result = await extractBytes(fileBytes, config);
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
  }, TEST_TIMEOUT_MS);

  it("api_extract_bytes_sync", () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping api_extract_bytes_sync: missing document at", documentPath);
      return;
    }
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      const fileBytes = readFileSync(documentPath);
      result = extractBytesSync(fileBytes, config);
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
  }, TEST_TIMEOUT_MS);

  it("api_extract_file_async", async () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping api_extract_file_async: missing document at", documentPath);
      return;
    }
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      result = await extractFile(documentPath, null, config);
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
  }, TEST_TIMEOUT_MS);

  it("api_extract_file_sync", () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping api_extract_file_sync: missing document at", documentPath);
      return;
    }
    const config = buildConfig(undefined);
    let result: ExtractionResult | null = null;
    try {
      result = extractFileSync(documentPath, null, config);
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
  }, TEST_TIMEOUT_MS);

  it("config_chunking", () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping config_chunking: missing document at", documentPath);
      return;
    }
    const config = buildConfig({"chunking":{"max_chars":500,"overlap":50}});
    let result: ExtractionResult | null = null;
    try {
      result = extractFileSync(documentPath, null, config);
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
    chunkAssertions.assertChunks(result, 1, null, true, null);
  }, TEST_TIMEOUT_MS);

  it("config_force_ocr", () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping config_force_ocr: missing document at", documentPath);
      return;
    }
    const config = buildConfig({"force_ocr":true});
    let result: ExtractionResult | null = null;
    try {
      result = extractFileSync(documentPath, null, config);
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
  }, TEST_TIMEOUT_MS);

  it("config_images", () => {
    const documentPath = resolveDocument("pdfs/embedded_images_tables.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping config_images: missing document at", documentPath);
      return;
    }
    const config = buildConfig({"images":{"extract":true,"format":"png"}});
    let result: ExtractionResult | null = null;
    try {
      result = extractFileSync(documentPath, null, config);
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
    chunkAssertions.assertImages(result, 1, null, null);
  }, TEST_TIMEOUT_MS);

  it("config_language_detection", () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping config_language_detection: missing document at", documentPath);
      return;
    }
    const config = buildConfig({"language_detection":{"enabled":true}});
    let result: ExtractionResult | null = null;
    try {
      result = extractFileSync(documentPath, null, config);
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
  }, TEST_TIMEOUT_MS);

  it("config_pages", () => {
    const documentPath = resolveDocument("pdfs/multi_page.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping config_pages: missing document at", documentPath);
      return;
    }
    const config = buildConfig({"pages":{"end":3,"start":1}});
    let result: ExtractionResult | null = null;
    try {
      result = extractFileSync(documentPath, null, config);
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
  }, TEST_TIMEOUT_MS);

  it("config_use_cache_false", () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping config_use_cache_false: missing document at", documentPath);
      return;
    }
    const config = buildConfig({"use_cache":false});
    let result: ExtractionResult | null = null;
    try {
      result = extractFileSync(documentPath, null, config);
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
  }, TEST_TIMEOUT_MS);

  it("output_format_djot", () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping output_format_djot: missing document at", documentPath);
      return;
    }
    const config = buildConfig({"output_format":"djot"});
    let result: ExtractionResult | null = null;
    try {
      result = extractFileSync(documentPath, null, config);
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
  }, TEST_TIMEOUT_MS);

  it("output_format_html", () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping output_format_html: missing document at", documentPath);
      return;
    }
    const config = buildConfig({"output_format":"html"});
    let result: ExtractionResult | null = null;
    try {
      result = extractFileSync(documentPath, null, config);
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
  }, TEST_TIMEOUT_MS);

  it("output_format_markdown", () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping output_format_markdown: missing document at", documentPath);
      return;
    }
    const config = buildConfig({"output_format":"markdown"});
    let result: ExtractionResult | null = null;
    try {
      result = extractFileSync(documentPath, null, config);
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
  }, TEST_TIMEOUT_MS);

  it("output_format_plain", () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping output_format_plain: missing document at", documentPath);
      return;
    }
    const config = buildConfig({"output_format":"plain"});
    let result: ExtractionResult | null = null;
    try {
      result = extractFileSync(documentPath, null, config);
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
  }, TEST_TIMEOUT_MS);

  it("result_format_element_based", () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping result_format_element_based: missing document at", documentPath);
      return;
    }
    const config = buildConfig({"result_format":"element_based"});
    let result: ExtractionResult | null = null;
    try {
      result = extractFileSync(documentPath, null, config);
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
    chunkAssertions.assertElements(result, 1, null);
  }, TEST_TIMEOUT_MS);

  it("result_format_unified", () => {
    const documentPath = resolveDocument("pdfs/fake_memo.pdf");
    if (!existsSync(documentPath)) {
      console.warn("Skipping result_format_unified: missing document at", documentPath);
      return;
    }
    const config = buildConfig({"result_format":"unified"});
    let result: ExtractionResult | null = null;
    try {
      result = extractFileSync(documentPath, null, config);
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
  }, TEST_TIMEOUT_MS);

});
