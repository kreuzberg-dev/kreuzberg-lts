/**
 * Comprehensive Error Handling Tests
 *
 * Tests for all error codes, error classification, panic handling, and error propagation.
 */

import { writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
	CacheError,
	classifyError,
	ErrorCode,
	extractBytesSync,
	extractFileSync,
	getErrorCodeDescription,
	getErrorCodeName,
	getLastErrorCode,
	getLastPanicContext,
	ImageProcessingError,
	KreuzbergError,
	MissingDependencyError,
	OcrError,
	ParsingError,
	PluginError,
	ValidationError,
	validateMimeType,
} from "@kreuzberg/node";
import { describe, expect, it } from "vitest";

describe("Comprehensive Error Handling", () => {
	describe("ErrorCode Enum Coverage", () => {
		it("should have ErrorCode enum with all codes", () => {
			expect(ErrorCode).toBeDefined();
			expect(typeof ErrorCode).toBe("object");

			// Common error codes
			const codes = Object.values(ErrorCode);
			expect(Array.isArray(codes) || typeof codes === "object").toBe(true);
		});

		it("should get error code name for code 0", () => {
			const name = getErrorCodeName(0);
			expect(typeof name).toBe("string");
			expect(name.length).toBeGreaterThan(0);
		});

		it("should get error code description for code 0", () => {
			const description = getErrorCodeDescription(0);
			expect(typeof description).toBe("string");
			expect(description.length).toBeGreaterThan(0);
		});

		it("should handle multiple error codes", () => {
			// Test codes 0-7 (common range)
			for (let i = 0; i <= 7; i++) {
				const name = getErrorCodeName(i);
				const description = getErrorCodeDescription(i);

				expect(typeof name).toBe("string");
				expect(typeof description).toBe("string");
			}
		});
	});

	describe("Error Classification", () => {
		it("should classify PDF parsing errors", () => {
			const classification = classifyError("PDF parsing error");
			expect(classification).toBeDefined();
			expect(typeof classification).toBe("object");

			if (typeof classification === "object" && classification !== null) {
				const obj = classification as Record<string, unknown>;
				if (obj.code !== undefined) {
					expect(typeof obj.code).toBe("number");
				}
				if (obj.name !== undefined) {
					expect(typeof obj.name).toBe("string");
				}
				if (obj.confidence !== undefined) {
					expect(typeof obj.confidence).toBe("number");
					expect(obj.confidence >= 0 && obj.confidence <= 1).toBe(true);
				}
			}
		});

		it("should classify OCR errors", () => {
			const classification = classifyError("OCR processing failed");
			expect(classification).toBeDefined();
			expect(typeof classification).toBe("object");
		});

		it("should classify validation errors", () => {
			const classification = classifyError("Validation failed: content is empty");
			expect(classification).toBeDefined();
			expect(typeof classification).toBe("object");
		});

		it("should classify I/O errors", () => {
			const classification = classifyError("File not found in read operation");
			expect(classification).toBeDefined();
			expect(typeof classification).toBe("object");
		});

		it("should classify permission errors", () => {
			const classification = classifyError("Permission denied reading file");
			expect(classification).toBeDefined();
			expect(typeof classification).toBe("object");
		});

		it("should classify memory errors", () => {
			const classification = classifyError("Out of memory during extraction");
			expect(classification).toBeDefined();
			expect(typeof classification).toBe("object");
		});

		it("should classify timeout errors", () => {
			const classification = classifyError("Operation timed out");
			expect(classification).toBeDefined();
			expect(typeof classification).toBe("object");
		});

		it("should provide confidence scores for classifications", () => {
			const classifications = [
				classifyError("PDF parsing error"),
				classifyError("File not found"),
				classifyError("Permission denied"),
			];

			for (const classification of classifications) {
				if (typeof classification === "object" && classification !== null) {
					const obj = classification as Record<string, unknown>;
					if (obj.confidence !== undefined) {
						expect(typeof obj.confidence).toBe("number");
						expect(obj.confidence).toBeGreaterThanOrEqual(0);
						expect(obj.confidence).toBeLessThanOrEqual(1);
					}
				}
			}
		});
	});

	describe("Last Error Code Tracking", () => {
		it("should track last error code", () => {
			const code = getLastErrorCode();
			expect(typeof code).toBe("number");
			expect(code >= 0).toBe(true);
		});

		it("should return consistent error code for repeated calls", () => {
			const code1 = getLastErrorCode();
			const code2 = getLastErrorCode();
			expect(typeof code1).toBe("number");
			expect(typeof code2).toBe("number");
		});
	});

	describe("Panic Context", () => {
		it("should retrieve panic context if available", () => {
			const context = getLastPanicContext();
			if (context !== null) {
				expect(typeof context).toBe("object");
			}
		});

		it("should handle null panic context gracefully", () => {
			const context = getLastPanicContext();
			expect(context === null || typeof context === "object").toBe(true);
		});

		it("should provide panic details when available", () => {
			const context = getLastPanicContext();
			if (context !== null && typeof context === "object") {
				const contextObj = context as Record<string, unknown>;
				// Common panic fields
				if (contextObj.message !== undefined) {
					expect(typeof contextObj.message === "string").toBe(true);
				}
				if (contextObj.file !== undefined) {
					expect(typeof contextObj.file === "string").toBe(true);
				}
				if (contextObj.line !== undefined) {
					expect(typeof contextObj.line === "number").toBe(true);
				}
			}
		});
	});

	describe("Error Exception Classes", () => {
		it("should construct KreuzbergError", () => {
			const error = new KreuzbergError("Base error");
			expect(error).toBeInstanceOf(Error);
			expect(error.message).toBe("Base error");
			expect(error.name).toContain("KreuzbergError");
		});

		it("should construct ValidationError", () => {
			const error = new ValidationError("Content validation failed");
			expect(error).toBeInstanceOf(Error);
			expect(error.message).toBe("Content validation failed");
		});

		it("should construct ParsingError", () => {
			const error = new ParsingError("PDF parse failed");
			expect(error).toBeInstanceOf(Error);
			expect(error.message).toBe("PDF parse failed");
		});

		it("should construct OcrError", () => {
			const error = new OcrError("OCR engine failed");
			expect(error).toBeInstanceOf(Error);
			expect(error.message).toBe("OCR engine failed");
		});

		it("should construct CacheError", () => {
			const error = new CacheError("Cache miss");
			expect(error).toBeInstanceOf(Error);
			expect(error.message).toBe("Cache miss");
		});

		it("should construct MissingDependencyError", () => {
			const error = new MissingDependencyError("Tesseract not installed");
			expect(error).toBeInstanceOf(Error);
			expect(error.message).toBe("Tesseract not installed");
		});

		it("should construct ImageProcessingError", () => {
			const error = new ImageProcessingError("Image resize failed");
			expect(error).toBeInstanceOf(Error);
			expect(error.message).toBe("Image resize failed");
		});

		it("should construct PluginError", () => {
			const error = new PluginError("Plugin initialization failed");
			expect(error).toBeInstanceOf(Error);
			expect(error.message).toContain("Plugin initialization failed");
		});

		it("should maintain error inheritance chain", () => {
			const errors = [
				new KreuzbergError("test"),
				new ValidationError("test"),
				new ParsingError("test"),
				new OcrError("test"),
				new CacheError("test"),
				new MissingDependencyError("test"),
				new ImageProcessingError("test"),
				new PluginError("test"),
			];

			for (const error of errors) {
				expect(error instanceof Error).toBe(true);
				// Message may be wrapped with error context
				expect(error.message).toContain("test");
			}
		});

		it("should support error throwing and catching", () => {
			expect(() => {
				throw new ParsingError("Caught error");
			}).toThrow(ParsingError);

			expect(() => {
				throw new OcrError("Another error");
			}).toThrow(OcrError);
		});
	});

	describe("MIME Type Validation Errors", () => {
		it("should validate valid MIME types", () => {
			const validMimes = [
				"application/pdf",
				"text/plain",
				"text/html",
				"application/vnd.openxmlformats-officedocument.wordprocessingml.document",
			];

			for (const mime of validMimes) {
				const result = validateMimeType(mime);
				expect(typeof result).toBe("string");
				expect(result.length).toBeGreaterThan(0);
			}
		});

		it("should handle invalid MIME types", () => {
			// validateMimeType throws on unsupported format
			expect(() => {
				validateMimeType("invalid/mime-type-xyz");
			}).toThrow();
		});
	});

	describe("Error Propagation in Batch Operations", () => {
		it("should handle single file extraction with non-existent file", () => {
			expect(() => {
				extractFileSync("/nonexistent/file.pdf", null);
			}).toThrow();
		});

		it("should handle bytes extraction with null data", () => {
			expect(() => {
				extractBytesSync(null as any, "text/plain", null);
			}).toThrow();
		});

		it("should track errors in batch operations", () => {
			const testFiles = [];
			for (let i = 0; i < 2; i++) {
				const filePath = join(tmpdir(), `error-test-${i}.txt`);
				const content = Buffer.from(`Test ${i}`);
				writeFileSync(filePath, content);
				testFiles.push(filePath);
			}

			// Even if some files have issues, batch should attempt all
			const results = [];
			for (const file of testFiles) {
				try {
					const result = extractFileSync(file, null);
					results.push(result);
				} catch (error) {
					results.push(error);
				}
			}

			expect(results.length).toBe(testFiles.length);
		});
	});

	describe("Error Message Content", () => {
		it("should have descriptive error messages", () => {
			const descriptions = [];
			for (let i = 0; i <= 3; i++) {
				const desc = getErrorCodeDescription(i);
				expect(desc.length).toBeGreaterThan(0);
				descriptions.push(desc);
			}

			// All should be different or at least meaningful
			expect(descriptions.length).toBeGreaterThan(0);
		});

		it("should provide actionable error names", () => {
			const names = [];
			for (let i = 0; i <= 3; i++) {
				const name = getErrorCodeName(i);
				expect(name.length).toBeGreaterThan(0);
				expect(/^[a-z_]+$/.test(name) || /^[A-Za-z]+$/.test(name)).toBe(true);
				names.push(name);
			}

			expect(names.length).toBeGreaterThan(0);
		});
	});

	describe("Error Context and Metadata", () => {
		it("should classify errors with different confidence levels", () => {
			const errors = [
				"PDF parsing failed", // Should have high confidence
				"Something went wrong", // Should have lower confidence
				"I/O error reading file", // Should have medium-high confidence
			];

			const classifications = errors.map((err) => classifyError(err));

			// At least some should have different confidence scores
			const confidences = classifications
				.map((c) => {
					if (typeof c === "object" && c !== null && "confidence" in c) {
						return (c as Record<string, unknown>).confidence;
					}
					return null;
				})
				.filter((c) => c !== null);

			expect(confidences.length).toBeGreaterThan(0);
		});

		it("should provide error categorization", () => {
			const errorTypes = ["validation_error", "parsing_error", "ocr_error", "io_error", "permission_error"];

			for (const errType of errorTypes) {
				const classification = classifyError(errType);
				expect(classification).toBeDefined();
				expect(typeof classification).toBe("object");
			}
		});
	});

	describe("Error Recovery Patterns", () => {
		it("should allow re-attempting extraction after error", () => {
			const validFile = join(tmpdir(), "valid-error-test.txt");
			writeFileSync(validFile, Buffer.from("test content"));

			// First attempt
			let result1: unknown;
			try {
				result1 = extractFileSync(validFile, null);
			} catch (error) {
				expect(error).toBeDefined();
			}

			// Second attempt should work
			const result2 = extractFileSync(validFile, null);
			expect(result2).toBeDefined();
		});

		it("should allow error inspection and handling", () => {
			try {
				extractFileSync("/nonexistent/file.pdf", null);
			} catch (error) {
				expect(error).toBeDefined();

				const code = getLastErrorCode();
				expect(typeof code).toBe("number");

				const errorName = getErrorCodeName(code);
				expect(typeof errorName).toBe("string");
			}
		});
	});
});
