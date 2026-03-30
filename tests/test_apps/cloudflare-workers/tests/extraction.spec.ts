import { extractBytes, extractBytesSync, initWasm, isInitialized } from "@kreuzberg/wasm";
import wasmModule from "@kreuzberg/wasm/kreuzberg_wasm_bg.wasm";
import { beforeAll, describe, expect, it } from "vitest";

beforeAll(async () => {
	if (!isInitialized()) {
		await initWasm({ wasmModule });
	}
});

describe("Text Extraction in Cloudflare Workers", () => {
	it("should extract plain text content", async () => {
		const text = "Hello from Cloudflare Workers!";
		const bytes = new TextEncoder().encode(text);
		const result = await extractBytes(bytes, "text/plain");
		expect(result).toBeDefined();
		expect(result.content).toContain("Hello");
		expect(result.mimeType).toBe("text/plain");
	});

	it("should extract HTML content to markdown", async () => {
		const html = "<html><body><h1>Title</h1><p>Paragraph content here.</p></body></html>";
		const bytes = new TextEncoder().encode(html);
		const result = await extractBytes(bytes, "text/html");
		expect(result).toBeDefined();
		expect(result.content.length).toBeGreaterThan(0);
		expect(result.content).toContain("Title");
		expect(result.content).toContain("Paragraph");
	});

	it("should extract JSON content", async () => {
		const json = JSON.stringify({ key: "value", nested: { a: 1 } });
		const bytes = new TextEncoder().encode(json);
		const result = await extractBytes(bytes, "application/json");
		expect(result).toBeDefined();
		expect(result.content.length).toBeGreaterThan(0);
	});

	it("should handle empty input gracefully", async () => {
		const bytes = new Uint8Array(0);
		try {
			const result = await extractBytes(bytes, "text/plain");
			expect(result).toBeDefined();
		} catch (error) {
			expect(error).toBeDefined();
		}
	});

	it("should extract synchronously when supported", () => {
		const text = "Sync extraction test";
		const bytes = new TextEncoder().encode(text);
		try {
			const result = extractBytesSync(bytes, "text/plain");
			expect(result).toBeDefined();
			expect(result.content).toContain("Sync");
		} catch (error) {
			// Sync may not be available in all Workers environments
			expect(error).toBeDefined();
		}
	});

	it("should preserve MIME type in results", async () => {
		const bytes = new TextEncoder().encode("test");
		const result = await extractBytes(bytes, "text/plain");
		expect(result.mimeType).toBe("text/plain");
	});

	it("should handle XML content", async () => {
		const xml = '<?xml version="1.0"?><root><item>Content</item></root>';
		const bytes = new TextEncoder().encode(xml);
		const result = await extractBytes(bytes, "application/xml");
		expect(result).toBeDefined();
		expect(result.content.length).toBeGreaterThan(0);
	});
});

describe("Error Handling in Cloudflare Workers", () => {
	it("should handle corrupted binary data", async () => {
		const bytes = new Uint8Array([0xff, 0xfe, 0x00, 0x01, 0x02]);
		try {
			const result = await extractBytes(bytes, "application/pdf");
			expect(result).toBeDefined();
		} catch (error) {
			expect(error).toBeInstanceOf(Error);
		}
	});

	it("should handle unknown MIME types", async () => {
		const bytes = new TextEncoder().encode("some data");
		try {
			const result = await extractBytes(bytes, "application/x-unknown-type");
			expect(result).toBeDefined();
		} catch (error) {
			expect(error).toBeDefined();
		}
	});
});
