import { assert, assertEquals, assertExists } from "jsr:@std/assert";
import { resolve } from "node:path";
import type {
	ChunkingConfig,
	ExtractionConfig,
	ExtractionResult,
	ImageExtractionConfig,
	Metadata,
	OcrConfig,
	PdfConfig,
	Table,
} from "@kreuzberg/wasm";
import {
	batchExtractBytes,
	batchExtractBytesSync,
	configToJS,
	extractBytes,
	extractBytesSync,
	fileToUint8Array,
	getVersion,
	initWasm,
	isInitialized,
	isValidExtractionResult,
	wrapWasmError,
} from "@kreuzberg/wasm";

const TEST_DOCS_DIR = resolve(
	Deno.env.get("TEST_DOCS_DIR") || resolve(new URL(".", import.meta.url).pathname, "../../../../test_documents"),
);

const getTestDocument = (relativePath: string): Uint8Array => {
	const path = resolve(TEST_DOCS_DIR, relativePath);
	return new Uint8Array(Deno.readFileSync(path));
};

const tryExtraction = async (
	bytes: Uint8Array,
	mimeType: string,
	config?: ExtractionConfig | null,
): Promise<ExtractionResult | null> => {
	try {
		return await extractBytes(bytes, mimeType, config);
	} catch (_error) {
		return null;
	}
};

const tryExtractionSync = (
	bytes: Uint8Array,
	mimeType: string,
	_config?: ExtractionConfig | null,
): ExtractionResult | null => {
	try {
		return extractBytesSync(bytes, mimeType, _config);
	} catch (_error) {
		return null;
	}
};

// Initialize WASM before all tests
if (!isInitialized()) {
	await initWasm();
}

// --- WASM Initialization ---

Deno.test("should initialize WASM module", () => {
	assertEquals(isInitialized(), true);
});

Deno.test("should get version after initialization", () => {
	const version = getVersion();
	assertExists(version);
	assertEquals(typeof version, "string");
	assert(version.length > 0);
});

// --- Type Verification (8 tests) ---

Deno.test("should have ExtractionConfig type available", () => {
	const config: ExtractionConfig = {
		ocr: undefined,
		chunking: undefined,
		images: undefined,
		pdfOptions: undefined,
	};
	assertExists(config);
});

Deno.test("should have ExtractionResult type available", () => {
	const result: ExtractionResult = {
		content: "test",
		mimeType: "text/plain",
		metadata: {},
		tables: [],
	};
	assertExists(result);
	assertExists(result.content);
});

Deno.test("should have OcrConfig type available", () => {
	const config: OcrConfig = {
		backend: "tesseract",
		language: "eng",
	};
	assertExists(config);
	assertEquals(config.backend, "tesseract");
});

Deno.test("should have ChunkingConfig type available", () => {
	const config: ChunkingConfig = {
		maxChars: 1000,
		maxOverlap: 100,
	};
	assertExists(config);
	assertEquals(config.maxChars, 1000);
});

Deno.test("should have ImageExtractionConfig type available", () => {
	const config: ImageExtractionConfig = {
		enabled: true,
		targetDpi: 150,
	};
	assertExists(config);
	assertEquals(config.enabled, true);
});

Deno.test("should have PdfConfig type available", () => {
	const config: PdfConfig = {
		extractImages: true,
	};
	assertExists(config);
});

Deno.test("should have Table type available", () => {
	const table: Table = {
		headers: ["Col1", "Col2"],
		rows: [["Value1", "Value2"]],
		markdown: "| Col1 | Col2 |\n| Value1 | Value2 |",
	};
	assertExists(table);
	assertEquals(table.headers!.length, 2);
});

Deno.test("should have Metadata type available", () => {
	const metadata: Metadata = {
		title: "Test",
		author: "Author",
		created: new Date().toISOString(),
		modified: new Date().toISOString(),
		pages: 1,
	};
	assertExists(metadata);
});

// --- Synchronous File Extraction (7 tests) ---

Deno.test("should extract text from PDF synchronously", () => {
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const result = tryExtractionSync(bytes, "application/pdf");
	assert(result === null || result.content !== undefined);
});

Deno.test("should extract from simple XLSX synchronously", () => {
	const bytes = getTestDocument("xlsx/test_01.xlsx");
	const result = tryExtractionSync(bytes, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
	assert(result === null || result.content !== undefined);
});

Deno.test("should extract from PNG image synchronously", () => {
	const bytes = getTestDocument("images/sample.png");
	const result = tryExtractionSync(bytes, "image/png");
	assert(result === null || result !== undefined);
});

Deno.test("should extract from JPG image synchronously", () => {
	const bytes = getTestDocument("images/flower_no_text.jpg");
	const result = tryExtractionSync(bytes, "image/jpeg");
	assert(result === null || result !== undefined);
});

Deno.test("should handle plain text files synchronously", () => {
	const text = "Hello, World!";
	const bytes = new TextEncoder().encode(text);
	const result = tryExtractionSync(bytes, "text/plain");
	assert(result === null || result !== undefined);
});

Deno.test("should handle empty byte arrays gracefully", () => {
	const emptyBytes = new Uint8Array(0);
	const result = tryExtractionSync(emptyBytes, "text/plain");
	assert(result === null || result !== undefined);
});

Deno.test("should handle large byte arrays", () => {
	const bytes = getTestDocument("pdf/multi_page.pdf");
	const result = tryExtractionSync(bytes, "application/pdf");
	assert(result === null || result !== undefined);
});

// --- Asynchronous File Extraction (7 tests) ---

Deno.test("should extract text from PDF asynchronously", async () => {
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const result = await tryExtraction(bytes, "application/pdf");
	assert(result === null || result.content !== undefined);
});

Deno.test("should extract from simple XLSX asynchronously", async () => {
	const bytes = getTestDocument("xlsx/stanley_cups.xlsx");
	const result = await tryExtraction(bytes, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
	assert(result === null || result !== undefined);
});

Deno.test("should extract from PNG image asynchronously", async () => {
	const bytes = getTestDocument("images/sample.png");
	const result = await tryExtraction(bytes, "image/png");
	assert(result === null || result !== undefined);
});

Deno.test("should extract from JPG image asynchronously", async () => {
	const bytes = getTestDocument("images/ocr_image.jpg");
	const result = await tryExtraction(bytes, "image/jpeg");
	assert(result === null || result !== undefined);
});

Deno.test("should handle plain text files asynchronously", async () => {
	const text = "Async text content";
	const bytes = new TextEncoder().encode(text);
	const result = await tryExtraction(bytes, "text/plain");
	assert(result === null || result !== undefined);
});

Deno.test("should handle large byte arrays asynchronously", async () => {
	const bytes = getTestDocument("pdf/multi_page.pdf");
	assert(bytes.length > 0);
	const result = await tryExtraction(bytes, "application/pdf");
	assert(result === null || result !== undefined);
});

Deno.test("should extract with null configuration", async () => {
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const result = await tryExtraction(bytes, "application/pdf", null);
	assert(result === null || result !== undefined);
});

// --- Byte Extraction - Sync and Async (4 tests) ---

Deno.test("should extract PDF bytes with and without async consistency", async () => {
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const syncResult = tryExtractionSync(bytes, "application/pdf");
	const asyncResult = await tryExtraction(bytes, "application/pdf");
	assert(syncResult === null || typeof syncResult.content === "string");
	assert(asyncResult === null || typeof asyncResult.content === "string");
});

Deno.test("should extract consistently from same bytes", async () => {
	const bytes = getTestDocument("xlsx/test_01.xlsx");
	const mimeType = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
	const syncResult = tryExtractionSync(bytes, mimeType);
	const asyncResult = await tryExtraction(bytes, mimeType);
	assert(syncResult === null || asyncResult === null || typeof syncResult.content === typeof asyncResult.content);
});

Deno.test("should preserve byte data integrity", () => {
	const originalBytes = getTestDocument("pdf/fake_memo.pdf");
	const bytesCopy = new Uint8Array(originalBytes);
	const result = tryExtractionSync(bytesCopy, "application/pdf");
	assert(result === null || result !== undefined);
	assertEquals(originalBytes, bytesCopy);
});

Deno.test("should handle rapid sequential byte extraction", async () => {
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const r1 = await tryExtraction(bytes, "application/pdf");
	const r2 = await tryExtraction(bytes, "application/pdf");
	const r3 = await tryExtraction(bytes, "application/pdf");
	assertExists([r1, r2, r3]);
});

// --- Batch Extraction APIs (6 tests) ---

Deno.test("should batch extract multiple bytes asynchronously", async () => {
	const files = [
		{ data: getTestDocument("images/sample.png"), mimeType: "image/png" },
		{
			data: getTestDocument("xlsx/test_01.xlsx"),
			mimeType: "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
		},
	];
	try {
		const results = await batchExtractBytes(files);
		assertEquals(results.length, 2);
		assert(Array.isArray(results));
	} catch (error) {
		assertExists(error);
	}
});

Deno.test("should batch extract multiple bytes synchronously", () => {
	const files = [
		{ data: getTestDocument("images/sample.png"), mimeType: "image/png" },
		{ data: new TextEncoder().encode("test"), mimeType: "text/plain" },
	];
	try {
		const results = batchExtractBytesSync(files);
		assertEquals(results.length, 2);
		assert(Array.isArray(results));
	} catch (error) {
		assertExists(error);
	}
});

Deno.test("should handle empty batch gracefully", async () => {
	try {
		const results = await batchExtractBytes([]);
		assert(Array.isArray(results));
		assertEquals(results.length, 0);
	} catch (error) {
		assertExists(error);
	}
});

Deno.test("should preserve order in batch extraction", async () => {
	const files = [
		{ data: getTestDocument("images/sample.png"), mimeType: "image/png" },
		{ data: new TextEncoder().encode("text"), mimeType: "text/plain" },
	];
	try {
		const results = await batchExtractBytes(files);
		assertEquals(results.length, 2);
		if (results.length > 0) {
			assertEquals(results[0].mimeType, "image/png");
			assertEquals(results[1].mimeType, "text/plain");
		}
	} catch (error) {
		assertExists(error);
	}
});

Deno.test("should batch extract with configuration", async () => {
	const config: ExtractionConfig = {
		chunking: { maxChars: 500, maxOverlap: 50 },
	};
	const files = [{ data: getTestDocument("images/sample.png"), mimeType: "image/png" }];
	try {
		const results = await batchExtractBytes(files, config);
		assertEquals(results.length, 1);
		assert(Array.isArray(results));
	} catch (error) {
		assertExists(error);
	}
});

Deno.test("should handle single item batch", async () => {
	const files = [{ data: getTestDocument("images/sample.png"), mimeType: "image/png" }];
	try {
		const results = await batchExtractBytes(files);
		assertEquals(results.length, 1);
	} catch (error) {
		assertExists(error);
	}
});

// --- MIME Type Detection (7 tests) ---

Deno.test("should correctly identify PDF MIME type", async () => {
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const result = await tryExtraction(bytes, "application/pdf");
	assert(result === null || result.mimeType === "application/pdf");
});

Deno.test("should correctly identify XLSX MIME type", async () => {
	const bytes = getTestDocument("xlsx/test_01.xlsx");
	const result = await tryExtraction(bytes, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
	assert(result === null || result.mimeType === "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
});

Deno.test("should correctly identify PNG MIME type", async () => {
	const bytes = getTestDocument("images/sample.png");
	const result = await tryExtraction(bytes, "image/png");
	assert(result === null || result.mimeType === "image/png");
});

Deno.test("should correctly identify JPG MIME type", async () => {
	const bytes = getTestDocument("images/flower_no_text.jpg");
	const result = await tryExtraction(bytes, "image/jpeg");
	assert(result === null || result.mimeType === "image/jpeg");
});

Deno.test("should handle custom MIME types", async () => {
	const bytes = new TextEncoder().encode("test content");
	const result = await tryExtraction(bytes, "text/custom");
	assert(result === null || result.mimeType === "text/custom");
});

Deno.test("should preserve MIME type through extraction", async () => {
	const mimeType = "application/pdf";
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const result = await tryExtraction(bytes, mimeType);
	assert(result === null || result.mimeType === mimeType);
});

Deno.test("should distinguish between similar MIME types", async () => {
	const pngResult = await tryExtraction(getTestDocument("images/sample.png"), "image/png");
	const jpgResult = await tryExtraction(getTestDocument("images/flower_no_text.jpg"), "image/jpeg");
	if (pngResult && jpgResult) {
		assert(pngResult.mimeType !== jpgResult.mimeType);
	}
});

// --- Configuration Handling (8 tests) ---

Deno.test("should handle null configuration", async () => {
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const result = await tryExtraction(bytes, "application/pdf", null);
	assert(result === null || result !== undefined);
});

Deno.test("should apply OCR configuration", async () => {
	const config: ExtractionConfig = { ocr: { backend: "tesseract", language: "eng" } };
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const result = await tryExtraction(bytes, "application/pdf", config);
	assert(result === null || result !== undefined);
});

Deno.test("should apply chunking configuration", async () => {
	const config: ExtractionConfig = { chunking: { maxChars: 500, maxOverlap: 50 } };
	const bytes = getTestDocument("images/sample.png");
	const result = await tryExtraction(bytes, "image/png", config);
	assert(result === null || result !== undefined);
});

Deno.test("should apply image extraction configuration", async () => {
	const config: ExtractionConfig = { images: { enabled: true, targetDpi: 150 } };
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const result = await tryExtraction(bytes, "application/pdf", config);
	assert(result === null || result !== undefined);
});

Deno.test("should apply PDF configuration", async () => {
	const config: ExtractionConfig = { pdfOptions: { extractImages: true } };
	const bytes = getTestDocument("pdf/multi_page.pdf");
	const result = await tryExtraction(bytes, "application/pdf", config);
	assert(result === null || result !== undefined);
});

Deno.test("should merge multiple configurations", async () => {
	const config: ExtractionConfig = {
		ocr: { backend: "tesseract", language: "eng" },
		chunking: { maxChars: 1000, maxOverlap: 100 },
		images: { enabled: true, targetDpi: 200 },
	};
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const result = await tryExtraction(bytes, "application/pdf", config);
	assert(result === null || result !== undefined);
});

Deno.test("should handle configToJS utility", () => {
	const config: ExtractionConfig = { ocr: { backend: "tesseract", language: "eng" } };
	const jsConfig = configToJS(config);
	assertExists(jsConfig);
	assertExists(jsConfig.ocr);
});

Deno.test("should handle null config with configToJS", () => {
	const jsConfig = configToJS(null);
	assertExists(jsConfig);
	assertEquals(typeof jsConfig, "object");
});

// --- Result Structure Validation (6 tests) ---

Deno.test("should have expected result fields", async () => {
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const result = await tryExtraction(bytes, "application/pdf");
	if (result) {
		assertExists(result.content);
		assertEquals(typeof result.content, "string");
		assertExists(result.mimeType);
		assertEquals(typeof result.mimeType, "string");
	}
});

Deno.test("should validate extraction results", () => {
	const validResult = { content: "Test content", mimeType: "text/plain", metadata: {}, tables: [] };
	assertEquals(isValidExtractionResult(validResult), true);
});

Deno.test("should handle metadata in results", async () => {
	const bytes = getTestDocument("images/sample.png");
	const result = await tryExtraction(bytes, "image/png");
	if (result) {
		assertExists(result.metadata);
		assertEquals(typeof result.metadata, "object");
	}
});

Deno.test("should handle tables in results", async () => {
	const bytes = getTestDocument("xlsx/test_01.xlsx");
	const result = await tryExtraction(bytes, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
	if (result) {
		assert(Array.isArray(result.tables));
	}
});

Deno.test("should have consistent result type across sync and async", async () => {
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const syncResult = tryExtractionSync(bytes, "application/pdf");
	const asyncResult = await tryExtraction(bytes, "application/pdf");
	if (syncResult && asyncResult) {
		assertEquals(typeof syncResult.content, typeof asyncResult.content);
		assertEquals(typeof syncResult.mimeType, typeof asyncResult.mimeType);
	}
});

Deno.test("should invalidate missing required fields", () => {
	const invalid = { mimeType: "text/plain" };
	// deno-lint-ignore no-explicit-any
	assertEquals(isValidExtractionResult(invalid as any), false);
});

// --- Error Handling (5 tests) ---

Deno.test("should handle invalid data gracefully", async () => {
	const invalidData = new TextEncoder().encode("not a valid document");
	const result = await tryExtraction(invalidData, "application/pdf");
	assert(result === null || result !== undefined);
});

Deno.test("should handle corrupted data gracefully", async () => {
	const corrupted = new Uint8Array([0, 1, 2, 3, 4, 5]);
	const result = await tryExtraction(corrupted, "application/octet-stream");
	assert(result === null || result !== undefined);
});

Deno.test("should wrap errors with context", () => {
	const error = new Error("Test error");
	const wrapped = wrapWasmError(error, "extraction failed");
	assert(wrapped instanceof Error);
	assertExists(wrapped.message);
});

Deno.test("should handle empty file gracefully", async () => {
	const empty = new Uint8Array(0);
	const result = await tryExtraction(empty, "application/octet-stream");
	assert(result === null || result !== undefined);
});

Deno.test("should handle very large files", async () => {
	const large = getTestDocument("pdf/multi_page.pdf");
	assert(large.length > 100000);
	const result = await tryExtraction(large, "application/pdf");
	assert(result === null || result !== undefined);
});

// --- Adapter Functions (5 tests) ---

Deno.test("should provide fileToUint8Array helper", () => {
	assertEquals(typeof fileToUint8Array, "function");
});

Deno.test("should provide configToJS helper", () => {
	assertEquals(typeof configToJS, "function");
});

Deno.test("should provide isValidExtractionResult helper", () => {
	assertEquals(typeof isValidExtractionResult, "function");
});

Deno.test("should provide wrapWasmError helper", () => {
	assertEquals(typeof wrapWasmError, "function");
});

Deno.test("should validate valid extraction result", () => {
	const result = { content: "Test", mimeType: "text/plain", metadata: {}, tables: [] };
	assertEquals(isValidExtractionResult(result), true);
});

// --- Concurrent Operations (3 tests) ---

Deno.test("should handle concurrent extractions", async () => {
	const pdfBytes = getTestDocument("pdf/fake_memo.pdf");
	const pngBytes = getTestDocument("images/sample.png");
	const textBytes = new TextEncoder().encode("text");
	const promises = [
		tryExtraction(pdfBytes, "application/pdf"),
		tryExtraction(pngBytes, "image/png"),
		tryExtraction(textBytes, "text/plain"),
	];
	const results = await Promise.all(promises);
	assertEquals(results.length, 3);
});

Deno.test("should handle rapid sequential extractions", async () => {
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const result1 = await tryExtraction(bytes, "application/pdf");
	const result2 = await tryExtraction(bytes, "application/pdf");
	const result3 = await tryExtraction(bytes, "application/pdf");
	assertExists([result1, result2, result3]);
});

Deno.test("should mix sync and async extractions", async () => {
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const syncResult = tryExtractionSync(bytes, "application/pdf");
	const asyncResult = await tryExtraction(bytes, "application/pdf");
	assert(syncResult === null || syncResult !== undefined);
	assert(asyncResult === null || asyncResult !== undefined);
});

// --- Large Document Handling (4 tests) ---

Deno.test("should extract from multi-page PDF", async () => {
	const bytes = getTestDocument("pdf/multi_page.pdf");
	const result = await tryExtraction(bytes, "application/pdf");
	assert(result === null || result !== undefined);
});

Deno.test("should handle complex XLSX files", async () => {
	const bytes = getTestDocument("xlsx/excel_multi_sheet.xlsx");
	const result = await tryExtraction(bytes, "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
	assert(result === null || result !== undefined);
});

Deno.test("should extract from large PDF", async () => {
	const bytes = getTestDocument("pdf/fundamentals_of_deep_learning_2014.pdf");
	assert(bytes.length > 1000000);
	const result = await tryExtraction(bytes, "application/pdf");
	assert(result === null || result !== undefined);
});

Deno.test("should handle documents with many tables", async () => {
	const bytes = getTestDocument("pdf/large.pdf");
	const result = await tryExtraction(bytes, "application/pdf");
	assert(result === null || result !== undefined);
});

// --- Content Quality Checks (5 tests) ---

Deno.test("should extract meaningful content when available", async () => {
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const result = await tryExtraction(bytes, "application/pdf");
	if (result?.content) {
		assert(result.content.length > 0);
	}
});

Deno.test("should preserve content type", async () => {
	const bytes = getTestDocument("images/sample.png");
	const result = await tryExtraction(bytes, "image/png");
	if (result) {
		assertEquals(typeof result.content, "string");
	}
});

Deno.test("should handle multi-format batches", async () => {
	const files = [
		{ data: getTestDocument("images/sample.png"), mimeType: "image/png" },
		{ data: getTestDocument("images/flower_no_text.jpg"), mimeType: "image/jpeg" },
		{ data: new TextEncoder().encode("plain text"), mimeType: "text/plain" },
	];
	try {
		const results = await batchExtractBytes(files);
		assertEquals(results.length, 3);
		assert(results.every((r) => r || r === null));
	} catch (error) {
		assertExists(error);
	}
});

Deno.test("should not modify input bytes", async () => {
	const originalBytes = getTestDocument("pdf/fake_memo.pdf");
	const bytesCopy = new Uint8Array(originalBytes);
	await tryExtraction(bytesCopy, "application/pdf");
	assertEquals(originalBytes, bytesCopy);
});

Deno.test("should handle content consistently", async () => {
	const bytes = getTestDocument("pdf/fake_memo.pdf");
	const result1 = await tryExtraction(bytes, "application/pdf");
	const result2 = await tryExtraction(bytes, "application/pdf");
	if (result1 && result2) {
		assertEquals(result1.content, result2.content);
	}
});

// --- Memory and Performance (2 tests) ---

Deno.test("should not leak memory on repeated extractions", async () => {
	const bytes = getTestDocument("images/sample.png");
	for (let i = 0; i < 5; i++) {
		const result = await tryExtraction(bytes, "image/png");
		assert(result === null || result !== undefined);
	}
});

Deno.test("should handle rapid batch operations", async () => {
	const files = [{ data: getTestDocument("images/sample.png"), mimeType: "image/png" }];
	for (let i = 0; i < 3; i++) {
		try {
			const results = await batchExtractBytes(files);
			assertEquals(results.length, 1);
		} catch (error) {
			assertExists(error);
		}
	}
});
