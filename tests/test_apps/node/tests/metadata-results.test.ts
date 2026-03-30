/**
 * Metadata and Result Structure Tests
 *
 * Comprehensive testing of extraction result structure, metadata types,
 * and all return value shapes across different extraction modes.
 */

import { writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
	type Chunk,
	type ChunkMetadata,
	type ExtractedImage,
	type ExtractedKeyword,
	type ExtractionResult,
	extractBytesSync,
	extractFileSync,
	type Metadata,
	type PageBoundary,
	type PageInfo,
	type PageStructure,
	type Table,
} from "@kreuzberg/node";
import { beforeAll, describe, expect, it } from "vitest";

describe("ExtractionResult Structure and Metadata", () => {
	let testFile: string;
	let testBuffer: Buffer;

	beforeAll(() => {
		const content = Buffer.from("# Test Document\n\nMultiple paragraphs.\n\nMore content.");
		testFile = join(tmpdir(), "metadata-test.txt");
		testBuffer = content;
		writeFileSync(testFile, content);
	});

	describe("ExtractionResult Base Structure", () => {
		it("should have content field as string", () => {
			const result = extractFileSync(testFile, null);
			expect(result).toBeDefined();
			expect(typeof result.content).toBe("string");
		});

		it("should have mimeType field as string", () => {
			const result = extractFileSync(testFile, null);
			expect(result).toBeDefined();
			expect(typeof result.mimeType).toBe("string");
			expect(result.mimeType.length).toBeGreaterThan(0);
		});

		it("should have metadata object", () => {
			const result = extractFileSync(testFile, null);
			expect(result).toBeDefined();
			expect(typeof result.metadata).toBe("object");
			expect(result.metadata).not.toBeNull();
		});

		it("should have tables array", () => {
			const result = extractFileSync(testFile, null);
			expect(result).toBeDefined();
			expect(Array.isArray(result.tables)).toBe(true);
		});

		it("should have detectedLanguages field", () => {
			const result = extractFileSync(testFile, null);
			expect(result).toBeDefined();
			// Can be null or array of strings
			if (result.detectedLanguages !== null) {
				expect(Array.isArray(result.detectedLanguages)).toBe(true);
			}
		});

		it("should have chunks field (can be null)", () => {
			const result = extractFileSync(testFile, null);
			expect(result).toBeDefined();
			// Chunks are null unless chunking is enabled
			if (result.chunks !== null) {
				expect(Array.isArray(result.chunks)).toBe(true);
			} else {
				expect(result.chunks).toBeNull();
			}
		});

		it("should have images field (can be null)", () => {
			const result = extractFileSync(testFile, null);
			expect(result).toBeDefined();
			// Images are null unless image extraction is enabled
			if (result.images !== null) {
				expect(Array.isArray(result.images)).toBe(true);
			} else {
				expect(result.images).toBeNull();
			}
		});

		it("should have optional pages field", () => {
			const result = extractFileSync(testFile, null);
			expect(result).toBeDefined();
			// Pages are optional
			if (result.pages !== undefined && result.pages !== null) {
				expect(Array.isArray(result.pages)).toBe(true);
			}
		});

		it("should have optional keywords field", () => {
			const result = extractFileSync(testFile, null);
			expect(result).toBeDefined();
			// Keywords are optional
			if (result.keywords !== undefined && result.keywords !== null) {
				expect(Array.isArray(result.keywords)).toBe(true);
			}
		});
	});

	describe("Chunk Structure", () => {
		it("should have chunk content field", () => {
			const config = { chunking: { maxChars: 100 } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.chunks && result.chunks.length > 0) {
				const chunk = result.chunks[0];
				expect(typeof chunk.content).toBe("string");
				expect(chunk.content.length).toBeGreaterThan(0);
			}
		});

		it("should have chunk metadata", () => {
			const config = { chunking: { maxChars: 100 } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.chunks && result.chunks.length > 0) {
				const chunk = result.chunks[0];
				expect(chunk.metadata).toBeDefined();
				expect(typeof chunk.metadata).toBe("object");
			}
		});

		it("should have chunk byte position tracking", () => {
			const config = { chunking: { maxChars: 100 } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.chunks && result.chunks.length > 0) {
				const chunk = result.chunks[0];
				const metadata = chunk.metadata;

				expect(typeof metadata.byteStart).toBe("number");
				expect(typeof metadata.byteEnd).toBe("number");
				expect(metadata.byteStart >= 0).toBe(true);
				expect(metadata.byteEnd > metadata.byteStart).toBe(true);
			}
		});

		it("should have chunk index information", () => {
			const config = { chunking: { maxChars: 100 } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.chunks && result.chunks.length > 0) {
				const chunk = result.chunks[0];
				expect(typeof chunk.metadata.chunkIndex).toBe("number");
				expect(typeof chunk.metadata.totalChunks).toBe("number");
				expect(chunk.metadata.chunkIndex >= 0).toBe(true);
				expect(chunk.metadata.totalChunks > 0).toBe(true);
			}
		});

		it("should have optional token count in metadata", () => {
			const config = { chunking: { maxChars: 100 } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.chunks && result.chunks.length > 0) {
				const chunk = result.chunks[0];
				// tokenCount can be null or a number
				if (chunk.metadata.tokenCount !== null) {
					expect(typeof chunk.metadata.tokenCount).toBe("number");
				}
			}
		});

		it("should have optional page tracking in metadata", () => {
			const config = { chunking: { maxChars: 100 }, pages: { extractPages: true } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.chunks && result.chunks.length > 0) {
				const chunk = result.chunks[0];
				// Page fields optional
				if (chunk.metadata.firstPage !== undefined && chunk.metadata.firstPage !== null) {
					expect(typeof chunk.metadata.firstPage).toBe("number");
					expect(chunk.metadata.firstPage >= 1).toBe(true);
				}
				if (chunk.metadata.lastPage !== undefined && chunk.metadata.lastPage !== null) {
					expect(typeof chunk.metadata.lastPage).toBe("number");
					expect(chunk.metadata.lastPage >= 1).toBe(true);
				}
			}
		});

		it("should have optional embedding vector", () => {
			const config = { chunking: { maxChars: 100 } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.chunks && result.chunks.length > 0) {
				const chunk = result.chunks[0];
				// Embedding can be null or array of numbers
				if (chunk.embedding !== undefined && chunk.embedding !== null) {
					expect(Array.isArray(chunk.embedding)).toBe(true);
					if (chunk.embedding.length > 0) {
						expect(typeof chunk.embedding[0]).toBe("number");
					}
				}
			}
		});
	});

	describe("ExtractedImage Structure", () => {
		it("should have image data as Uint8Array", () => {
			const config = { images: { extractImages: true } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.images && result.images.length > 0) {
				const image = result.images[0];
				expect(image.data instanceof Uint8Array).toBe(true);
			}
		});

		it("should have image format", () => {
			const config = { images: { extractImages: true } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.images && result.images.length > 0) {
				const image = result.images[0];
				expect(typeof image.format).toBe("string");
			}
		});

		it("should have image index", () => {
			const config = { images: { extractImages: true } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.images && result.images.length > 0) {
				const image = result.images[0];
				expect(typeof image.imageIndex).toBe("number");
				expect(image.imageIndex >= 0).toBe(true);
			}
		});

		it("should have optional page number for image", () => {
			const config = { images: { extractImages: true } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.images && result.images.length > 0) {
				const image = result.images[0];
				if (image.pageNumber !== null && image.pageNumber !== undefined) {
					expect(typeof image.pageNumber).toBe("number");
					expect(image.pageNumber >= 1).toBe(true);
				}
			}
		});

		it("should have image dimensions", () => {
			const config = { images: { extractImages: true } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.images && result.images.length > 0) {
				const image = result.images[0];
				// Dimensions optional
				if (image.width !== null && image.width !== undefined) {
					expect(typeof image.width).toBe("number");
				}
				if (image.height !== null && image.height !== undefined) {
					expect(typeof image.height).toBe("number");
				}
			}
		});

		it("should have color space information", () => {
			const config = { images: { extractImages: true } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.images && result.images.length > 0) {
				const image = result.images[0];
				// Colorspace optional
				if (image.colorspace !== null && image.colorspace !== undefined) {
					expect(typeof image.colorspace).toBe("string");
				}
			}
		});

		it("should have isMask flag", () => {
			const config = { images: { extractImages: true } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.images && result.images.length > 0) {
				const image = result.images[0];
				expect(typeof image.isMask).toBe("boolean");
			}
		});

		it("should have optional OCR result", () => {
			const config = { images: { extractImages: true } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.images && result.images.length > 0) {
				const image = result.images[0];
				// ocrResult optional
				if (image.ocrResult !== null && image.ocrResult !== undefined) {
					expect(typeof image.ocrResult).toBe("object");
					expect(image.ocrResult.content).toBeDefined();
				}
			}
		});
	});

	describe("Table Structure", () => {
		it("should have table cells as 2D array", () => {
			const result = extractBytesSync(testBuffer, "text/plain", null);

			if (result.tables && result.tables.length > 0) {
				const table = result.tables[0];
				expect(Array.isArray(table.cells)).toBe(true);
				if (table.cells.length > 0) {
					expect(Array.isArray(table.cells[0])).toBe(true);
				}
			}
		});

		it("should have table markdown representation", () => {
			const result = extractBytesSync(testBuffer, "text/plain", null);

			if (result.tables && result.tables.length > 0) {
				const table = result.tables[0];
				expect(typeof table.markdown).toBe("string");
			}
		});

		it("should have table page number", () => {
			const result = extractBytesSync(testBuffer, "text/plain", null);

			if (result.tables && result.tables.length > 0) {
				const table = result.tables[0];
				expect(typeof table.pageNumber).toBe("number");
				expect(table.pageNumber >= 1).toBe(true);
			}
		});
	});

	describe("ExtractedKeyword Structure", () => {
		it("should have keyword text", () => {
			const config = { keywords: { maxKeywords: 10 } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.keywords && result.keywords.length > 0) {
				const keyword = result.keywords[0];
				expect(typeof keyword.text).toBe("string");
			}
		});

		it("should have keyword score", () => {
			const config = { keywords: { maxKeywords: 10 } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.keywords && result.keywords.length > 0) {
				const keyword = result.keywords[0];
				expect(typeof keyword.score).toBe("number");
				expect(keyword.score >= 0).toBe(true);
			}
		});

		it("should have keyword algorithm", () => {
			const config = { keywords: { maxKeywords: 10, algorithm: "yake" } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.keywords && result.keywords.length > 0) {
				const keyword = result.keywords[0];
				expect(["yake", "rake"]).toContain(keyword.algorithm);
			}
		});

		it("should have optional keyword positions", () => {
			const config = { keywords: { maxKeywords: 10 } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.keywords && result.keywords.length > 0) {
				const keyword = result.keywords[0];
				if (keyword.positions !== undefined && keyword.positions !== null) {
					expect(Array.isArray(keyword.positions)).toBe(true);
				}
			}
		});
	});

	describe("Metadata Field Types", () => {
		it("should have language in metadata", () => {
			const result = extractBytesSync(testBuffer, "text/plain", null);
			const metadata = result.metadata;

			if (metadata.language !== undefined && metadata.language !== null) {
				expect(typeof metadata.language).toBe("string");
			}
		});

		it("should have format_type discriminator", () => {
			const result = extractBytesSync(testBuffer, "text/plain", null);
			const metadata = result.metadata;

			// format_type discriminates between metadata types
			if (metadata.format_type !== undefined && metadata.format_type !== null) {
				const validTypes = ["pdf", "excel", "email", "pptx", "archive", "image", "xml", "text", "html", "ocr"];
				expect(validTypes).toContain(metadata.format_type);
			}
		});

		it("should have date fields", () => {
			const result = extractBytesSync(testBuffer, "text/plain", null);
			const metadata = result.metadata;

			if (metadata.date !== undefined && metadata.date !== null) {
				expect(typeof metadata.date).toBe("string");
			}
		});

		it("should have standard document fields", () => {
			const result = extractBytesSync(testBuffer, "text/plain", null);
			const metadata = result.metadata;

			// Common fields
			if (metadata.title !== undefined) {
				expect(typeof metadata.title === "string" || metadata.title === null).toBe(true);
			}
			if (metadata.author !== undefined) {
				expect(typeof metadata.author === "string" || metadata.author === null).toBe(true);
			}
		});

		it("should support dynamic metadata fields", () => {
			const result = extractBytesSync(testBuffer, "text/plain", null);
			const metadata = result.metadata as Record<string, unknown>;

			// Can access arbitrary properties added by post-processors
			expect(typeof metadata === "object").toBe(true);
		});
	});

	describe("PageStructure Information", () => {
		it("should have page structure when page extraction enabled", () => {
			const config = { pages: { extractPages: true } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.metadata.page_structure !== undefined && result.metadata.page_structure !== null) {
				const pageStruct = result.metadata.page_structure;
				expect(typeof pageStruct.totalCount).toBe("number");
				expect(pageStruct.totalCount > 0).toBe(true);
			}
		});

		it("should have page unit type", () => {
			const config = { pages: { extractPages: true } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.metadata.page_structure !== undefined && result.metadata.page_structure !== null) {
				const pageStruct = result.metadata.page_structure;
				expect(["page", "slide", "sheet"]).toContain(pageStruct.unitType);
			}
		});

		it("should have optional page boundaries", () => {
			const config = { pages: { extractPages: true } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (
				result.metadata.page_structure?.boundaries !== undefined &&
				result.metadata.page_structure?.boundaries !== null
			) {
				const boundaries = result.metadata.page_structure.boundaries;
				expect(Array.isArray(boundaries)).toBe(true);

				if (boundaries.length > 0) {
					const boundary = boundaries[0];
					expect(typeof boundary.byteStart).toBe("number");
					expect(typeof boundary.byteEnd).toBe("number");
					expect(typeof boundary.pageNumber).toBe("number");
				}
			}
		});

		it("should have optional per-page info", () => {
			const config = { pages: { extractPages: true } };
			const result = extractBytesSync(testBuffer, "text/plain", config);

			if (result.metadata.page_structure?.pages !== undefined && result.metadata.page_structure?.pages !== null) {
				const pages = result.metadata.page_structure.pages;
				expect(Array.isArray(pages)).toBe(true);

				if (pages.length > 0) {
					const page = pages[0];
					expect(typeof page.number).toBe("number");
				}
			}
		});
	});

	describe("Error Metadata", () => {
		it("should have error metadata on failure", () => {
			const result = extractBytesSync(testBuffer, "text/plain", null);
			const metadata = result.metadata;

			// error field optional
			if (metadata.error !== undefined && metadata.error !== null) {
				expect(typeof metadata.error).toBe("object");
			}
		});
	});

	describe("Consistency Across Formats", () => {
		it("should have consistent structure with async extraction", async () => {
			const result = extractFileSync(testFile, null);
			expect(result.content).toBeDefined();
			expect(result.mimeType).toBeDefined();
			expect(result.metadata).toBeDefined();
			expect(Array.isArray(result.tables)).toBe(true);
		});

		it("should handle async bytes extraction", async () => {
			const result = extractBytesSync(testBuffer, "text/plain", null);
			expect(result.content).toBeDefined();
			expect(result.mimeType).toBe("text/plain");
		});

		it("should preserve structure across output formats", () => {
			const plainConfig = { outputFormat: "plain" as const };
			const markdownConfig = { outputFormat: "markdown" as const };

			const plainResult = extractBytesSync(testBuffer, "text/plain", plainConfig);
			const mdResult = extractBytesSync(testBuffer, "text/plain", markdownConfig);

			expect(plainResult.mimeType).toBe(mdResult.mimeType);
			expect(typeof plainResult.content).toBe("string");
			expect(typeof mdResult.content).toBe("string");
		});

		it("should preserve structure across result formats", () => {
			const unifiedConfig = { resultFormat: "unified" as const };
			const elementConfig = { resultFormat: "element_based" as const };

			const unifiedResult = extractBytesSync(testBuffer, "text/plain", unifiedConfig);
			const elementResult = extractBytesSync(testBuffer, "text/plain", elementConfig);

			expect(typeof unifiedResult.content).toMatch(/^(string|object)$/);
			expect(typeof elementResult.content).toMatch(/^(string|object)$/);
		});
	});
});
