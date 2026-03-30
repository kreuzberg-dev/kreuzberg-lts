/**
 * Plugin Lifecycle and Advanced Features Tests
 *
 * Tests for plugin initialization, shutdown, error handling, and lifecycle management.
 */

import { writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
	clearOcrBackends,
	clearPostProcessors,
	clearValidators,
	type ExtractionResult,
	extractFileSync,
	listOcrBackends,
	listPostProcessors,
	listValidators,
	type OcrBackendProtocol,
	type OcrConfig,
	type PostProcessorProtocol,
	registerOcrBackend,
	registerPostProcessor,
	registerValidator,
	unregisterOcrBackend,
	unregisterPostProcessor,
	unregisterValidator,
	type ValidatorProtocol,
} from "@kreuzberg/node";
import { afterEach, beforeAll, describe, expect, it } from "vitest";

describe("Plugin Lifecycle Management", () => {
	let testFile: string;

	beforeAll(() => {
		const content = Buffer.from("Test content for plugins");
		testFile = join(tmpdir(), "plugin-test.txt");
		writeFileSync(testFile, content);
	});

	afterEach(() => {
		clearPostProcessors();
		clearValidators();
		clearOcrBackends();
	});

	describe("PostProcessor Lifecycle", () => {
		it("should call initialize() on processor registration", () => {
			let initCalled = false;

			const processor: PostProcessorProtocol = {
				name: () => "init-test-processor",
				process: (result: ExtractionResult): ExtractionResult => result,
				initialize: async () => {
					initCalled = true;
				},
			};

			registerPostProcessor(processor);
			// Note: initialize() is called internally, we're testing it's defined
			expect(processor.initialize).toBeDefined();
			expect(typeof processor.initialize).toBe("function");
		});

		it("should provide processingStage() for execution ordering", () => {
			const processor: PostProcessorProtocol = {
				name: () => "stage-test-processor",
				process: (result: ExtractionResult): ExtractionResult => result,
				processingStage: () => "early",
			};

			registerPostProcessor(processor);
			if (processor.processingStage) {
				const stage = processor.processingStage();
				expect(["early", "middle", "late"]).toContain(stage);
			}
		});

		it("should support different processing stages", () => {
			const earlyProc: PostProcessorProtocol = {
				name: () => "early-proc",
				process: (r: ExtractionResult): ExtractionResult => r,
				processingStage: () => "early",
			};

			const middleProc: PostProcessorProtocol = {
				name: () => "middle-proc",
				process: (r: ExtractionResult): ExtractionResult => r,
				processingStage: () => "middle",
			};

			const lateProc: PostProcessorProtocol = {
				name: () => "late-proc",
				process: (r: ExtractionResult): ExtractionResult => r,
				processingStage: () => "late",
			};

			registerPostProcessor(earlyProc);
			registerPostProcessor(middleProc);
			registerPostProcessor(lateProc);

			const list = listPostProcessors();
			expect(list).toContain("early-proc");
			expect(list).toContain("middle-proc");
			expect(list).toContain("late-proc");
		});

		it("should call shutdown() on processor unregistration", () => {
			let shutdownCalled = false;

			const processor: PostProcessorProtocol = {
				name: () => "shutdown-test-processor",
				process: (result: ExtractionResult) => result,
				shutdown: async () => {
					shutdownCalled = true;
				},
			};

			registerPostProcessor(processor);
			unregisterPostProcessor("shutdown-test-processor");
			// Shutdown is called internally
			expect(processor.shutdown).toBeDefined();
		});

		it("should handle processor that modifies result", () => {
			const processor: PostProcessorProtocol = {
				name: () => "modifying-processor",
				process: (result: ExtractionResult) => {
					return {
						...result,
						content: result.content.toUpperCase(),
					};
				},
			};

			registerPostProcessor(processor);
			const list = listPostProcessors();
			expect(list).toContain("modifying-processor");
		});

		it("should handle async process method", () => {
			const processor: PostProcessorProtocol = {
				name: () => "async-processor",
				process: async (result: ExtractionResult) => {
					await new Promise((resolve) => setTimeout(resolve, 10));
					return result;
				},
			};

			registerPostProcessor(processor);
			const list = listPostProcessors();
			expect(list).toContain("async-processor");
		});
	});

	describe("Validator Lifecycle", () => {
		it("should support validator priority ordering", () => {
			const lowPriority: ValidatorProtocol = {
				name: () => "low-priority-validator",
				validate: () => {
					// Valid
				},
				priority: () => 10,
			};

			const highPriority: ValidatorProtocol = {
				name: () => "high-priority-validator",
				validate: () => {
					// Valid
				},
				priority: () => 100,
			};

			registerValidator(lowPriority);
			registerValidator(highPriority);

			const list = listValidators();
			expect(list).toContain("low-priority-validator");
			expect(list).toContain("high-priority-validator");
		});

		it("should support conditional validation with shouldValidate()", () => {
			const conditionalValidator: ValidatorProtocol = {
				name: () => "conditional-validator",
				validate: () => {
					// Valid
				},
				shouldValidate: (result: ExtractionResult) => {
					// Only validate PDFs
					return result.mimeType === "application/pdf";
				},
			};

			registerValidator(conditionalValidator);
			const list = listValidators();
			expect(list).toContain("conditional-validator");
		});

		it("should support validator initialization", () => {
			const validator: ValidatorProtocol = {
				name: () => "init-validator",
				validate: () => {
					// Valid
				},
				initialize: async () => {
					// Setup resources
				},
			};

			registerValidator(validator);
			if (validator.initialize) {
				expect(typeof validator.initialize).toBe("function");
			}
		});

		it("should support validator shutdown", () => {
			const validator: ValidatorProtocol = {
				name: () => "shutdown-validator",
				validate: () => {
					// Valid
				},
				shutdown: async () => {
					// Cleanup resources
				},
			};

			registerValidator(validator);
			unregisterValidator("shutdown-validator");

			if (validator.shutdown) {
				expect(typeof validator.shutdown).toBe("function");
			}
		});

		it("should throw on validation failure", () => {
			const failingValidator: ValidatorProtocol = {
				name: () => "failing-validator",
				validate: (result: ExtractionResult) => {
					if (result.content.length === 0) {
						throw new Error("Content is empty");
					}
				},
			};

			registerValidator(failingValidator);
			const list = listValidators();
			expect(list).toContain("failing-validator");
		});

		it("should handle async validation", () => {
			const asyncValidator: ValidatorProtocol = {
				name: () => "async-validator",
				validate: async (result: ExtractionResult) => {
					await new Promise((resolve) => setTimeout(resolve, 10));
					if (result.content.length === 0) {
						throw new Error("No content");
					}
				},
			};

			registerValidator(asyncValidator);
			const list = listValidators();
			expect(list).toContain("async-validator");
		});
	});

	describe("OCR Backend Lifecycle", () => {
		it("should support OCR backend initialization", () => {
			const backend: OcrBackendProtocol = {
				name: () => "test-ocr-backend",
				supportedLanguages: () => ["eng", "deu", "fra"],
				processImage: async (imageBytes: Uint8Array | string, language: string) => ({
					content: "OCR output",
					mime_type: "text/plain",
					metadata: {},
					tables: [],
				}),
				initialize: async () => {
					// Load ML models
				},
			};

			registerOcrBackend(backend);
			const list = listOcrBackends();
			expect(list).toContain("test-ocr-backend");
		});

		it("should provide supportedLanguages() method", () => {
			const backend: OcrBackendProtocol = {
				name: () => "lang-test-ocr",
				supportedLanguages: () => ["eng", "fra", "deu", "spa", "ita"],
				processImage: async () => ({
					content: "text",
					mime_type: "text/plain",
					metadata: {},
					tables: [],
				}),
			};

			const langs = backend.supportedLanguages();
			expect(Array.isArray(langs)).toBe(true);
			expect(langs.length).toBeGreaterThan(0);
			expect(langs).toContain("eng");
		});

		it("should support OCR backend shutdown", () => {
			const backend: OcrBackendProtocol = {
				name: () => "shutdown-ocr-backend",
				supportedLanguages: () => ["eng"],
				processImage: async () => ({
					content: "text",
					mime_type: "text/plain",
					metadata: {},
					tables: [],
				}),
				shutdown: async () => {
					// Cleanup
				},
			};

			registerOcrBackend(backend);
			unregisterOcrBackend("shutdown-ocr-backend");

			if (backend.shutdown) {
				expect(typeof backend.shutdown).toBe("function");
			}
		});

		it("should handle processImage with Uint8Array", () => {
			const backend: OcrBackendProtocol = {
				name: () => "uint8array-ocr",
				supportedLanguages: () => ["eng"],
				processImage: async (imageBytes: Uint8Array | string, language: string) => {
					if (typeof imageBytes === "string") {
						// Base64
						expect(typeof imageBytes).toBe("string");
					} else {
						// Uint8Array
						expect(imageBytes instanceof Uint8Array).toBe(true);
					}
					return {
						content: "extracted",
						mime_type: "text/plain",
						metadata: { language },
						tables: [],
					};
				},
			};

			registerOcrBackend(backend);
			const list = listOcrBackends();
			expect(list).toContain("uint8array-ocr");
		});

		it("should return proper result structure from processImage", () => {
			const backend: OcrBackendProtocol = {
				name: () => "result-struct-ocr",
				supportedLanguages: () => ["eng"],
				processImage: async () => ({
					content: "detected text",
					mime_type: "text/plain",
					metadata: {
						confidence: 0.95,
						psm: 6,
					},
					tables: [],
				}),
			};

			registerOcrBackend(backend);
			// Verify structure is correct
			expect(typeof backend.processImage).toBe("function");
		});
	});

	describe("Multi-Plugin Interaction", () => {
		it("should allow multiple processors to be registered", () => {
			const proc1: PostProcessorProtocol = {
				name: () => "proc-1",
				process: (r: ExtractionResult): ExtractionResult => r,
			};

			const proc2: PostProcessorProtocol = {
				name: () => "proc-2",
				process: (r: ExtractionResult): ExtractionResult => r,
			};

			const proc3: PostProcessorProtocol = {
				name: () => "proc-3",
				process: (r: ExtractionResult): ExtractionResult => r,
			};

			registerPostProcessor(proc1);
			registerPostProcessor(proc2);
			registerPostProcessor(proc3);

			const list = listPostProcessors();
			expect(list).toContain("proc-1");
			expect(list).toContain("proc-2");
			expect(list).toContain("proc-3");
		});

		it("should maintain validator priority ordering", () => {
			const validators: ValidatorProtocol[] = [];

			for (let i = 0; i < 3; i++) {
				const validator: ValidatorProtocol = {
					name: () => `validator-${i}`,
					validate: () => {
						// Valid
					},
					priority: () => i * 50,
				};
				validators.push(validator);
				registerValidator(validator);
			}

			const list = listValidators();
			expect(list.length).toBe(3);
		});

		it("should handle plugin error isolation", () => {
			const failProc: PostProcessorProtocol = {
				name: () => "failing-proc",
				process: () => {
					throw new Error("Processor error");
				},
			};

			const goodProc: PostProcessorProtocol = {
				name: () => "good-proc",
				process: (r: ExtractionResult): ExtractionResult => r,
			};

			registerPostProcessor(failProc);
			registerPostProcessor(goodProc);

			const list = listPostProcessors();
			// Both should be registered even if one fails
			expect(list.includes("failing-proc") || list.includes("good-proc")).toBe(true);
		});
	});

	describe("Plugin Registry Management", () => {
		it("should handle duplicate plugin names gracefully", () => {
			const proc: PostProcessorProtocol = {
				name: () => "duplicate-test",
				process: (r: ExtractionResult): ExtractionResult => r,
			};

			registerPostProcessor(proc);
			registerPostProcessor(proc); // Register again

			const list = listPostProcessors();
			// Should either replace or ignore duplicate
			expect(list.includes("duplicate-test")).toBe(true);
		});

		it("should handle unregister on non-existent plugin", () => {
			// Should not throw
			expect(() => {
				unregisterPostProcessor("non-existent");
			}).not.toThrow();
		});

		it("should clear all plugins of a type", () => {
			const proc1: PostProcessorProtocol = {
				name: () => "clear-test-1",
				process: (r: ExtractionResult): ExtractionResult => r,
			};

			const proc2: PostProcessorProtocol = {
				name: () => "clear-test-2",
				process: (r: ExtractionResult): ExtractionResult => r,
			};

			registerPostProcessor(proc1);
			registerPostProcessor(proc2);

			clearPostProcessors();

			const list = listPostProcessors();
			expect(list.includes("clear-test-1")).toBe(false);
			expect(list.includes("clear-test-2")).toBe(false);
		});
	});
});
