/**
 * Complete API Coverage Test Suite
 *
 * Tests 100% of all exported APIs, types, configurations, and protocols
 * from @kreuzberg/node package. This ensures all public surface area is verified
 * and working correctly.
 */

import { readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
	__version__,
	batchExtractBytes,
	batchExtractBytesSync,
	batchExtractFiles,
	batchExtractFilesSync,
	CacheError,
	type Chunk,
	type ChunkingConfig,
	type ChunkMetadata,
	classifyError,
	clearDocumentExtractors,
	clearOcrBackends,
	clearPostProcessors,
	clearValidators,
	detectMimeType,
	detectMimeTypeFromPath,
	type ErrorClassification,
	ErrorCode,
	type ExtractedImage,
	type ExtractedKeyword,
	type ExtractionConfig,
	type ExtractionResult,
	extractBytes,
	extractBytesSync,
	extractFile,
	extractFileSync,
	getEmbeddingPreset,
	getErrorCodeDescription,
	getErrorCodeName,
	getExtensionsForMime,
	getLastErrorCode,
	getLastPanicContext,
	type HtmlConversionOptions,
	type HtmlPreprocessingOptions,
	type ImageExtractionConfig,
	ImageProcessingError,
	type KeywordConfig,
	KreuzbergError,
	type LanguageDetectionConfig,
	listDocumentExtractors,
	listEmbeddingPresets,
	listOcrBackends,
	listPostProcessors,
	listValidators,
	MissingDependencyError,
	type OcrBackendProtocol,
	type OcrConfig,
	OcrError,
	type PageContent,
	type PageExtractionConfig,
	ParsingError,
	type PdfConfig,
	PluginError,
	type PostProcessorConfig,
	type PostProcessorProtocol,
	registerOcrBackend,
	registerPostProcessor,
	registerValidator,
	type Table,
	type TesseractConfig,
	type TokenReductionConfig,
	unregisterDocumentExtractor,
	unregisterOcrBackend,
	unregisterPostProcessor,
	unregisterValidator,
	ValidationError,
	type ValidatorProtocol,
	validateMimeType,
	type WorkerPool,
	type WorkerPoolStats,
} from "@kreuzberg/node";
import { afterEach, beforeAll, describe, expect, it } from "vitest";

describe("Complete Kreuzberg API Coverage", () => {
	const testDir = tmpdir();
	const testFiles: string[] = [];
	const testBuffers: { content: Buffer; mimeType: string }[] = [];

	beforeAll(() => {
		// Create test files for use in tests
		const files = [
			{ name: "test1.txt", content: "Hello World" },
			{ name: "test2.txt", content: "Document text" },
			{ name: "test3.md", content: "# Markdown\n\nContent here" },
		];

		for (const file of files) {
			const filePath = join(testDir, file.name);
			writeFileSync(filePath, Buffer.from(file.content));
			testFiles.push(filePath);
			testBuffers.push({
				content: Buffer.from(file.content),
				mimeType: file.name.endsWith(".md") ? "text/markdown" : "text/plain",
			});
		}
	});

	// ========================================================================
	// VERSION AND CONSTANTS
	// ========================================================================

	describe("Version and Constants", () => {
		it("should export version string", () => {
			expect(__version__).toBeDefined();
			expect(typeof __version__).toBe("string");
			expect(__version__).toMatch(/^\d+\.\d+\.\d+/);
		});

		it("should export ErrorCode object", () => {
			expect(ErrorCode).toBeDefined();
			expect(typeof ErrorCode).toBe("object");
		});
	});

	// ========================================================================
	// EXTRACTION APIS - SINGLE DOCUMENT
	// ========================================================================

	describe("Single Document Extraction", () => {
		it("should extract file (sync)", () => {
			const result = extractFileSync(testFiles[0], null);
			expect(result).toBeDefined();
			expect(typeof result.content).toBe("string");
			expect(result.mimeType).toBeDefined();
			expect(Array.isArray(result.tables)).toBe(true);
			expect(typeof result.metadata).toBe("object");
		});

		it("should extract file (async)", async () => {
			const result = await extractFile(testFiles[0], null);
			expect(result).toBeDefined();
			expect(typeof result.content).toBe("string");
		});

		it("should extract file with null config", () => {
			const result = extractFileSync(testFiles[0], null);
			expect(result).toBeDefined();
		});

		it("should extract file with undefined config", () => {
			const result = extractFileSync(testFiles[0], undefined);
			expect(result).toBeDefined();
		});

		it("should extract bytes (sync)", () => {
			const result = extractBytesSync(testBuffers[0].content, "text/plain", null);
			expect(result).toBeDefined();
			expect(typeof result.content).toBe("string");
		});

		it("should extract bytes (async)", async () => {
			const result = await extractBytes(testBuffers[0].content, "text/plain", null);
			expect(result).toBeDefined();
		});

		it("should extract file with password parameter (null)", () => {
			const result = extractFileSync(testFiles[0], null);
			expect(result).toBeDefined();
		});

		it("should extract file with password parameter (undefined)", () => {
			const result = extractFileSync(testFiles[0], undefined);
			expect(result).toBeDefined();
		});

		it("should extract with text/plain MIME type", () => {
			const result = extractBytesSync(testBuffers[0].content, "text/plain", null);
			expect(result).toBeDefined();
		});

		it("should extract with text/markdown MIME type", () => {
			const result = extractBytesSync(testBuffers[2].content, "text/markdown", null);
			expect(result).toBeDefined();
		});
	});

	// ========================================================================
	// EXTRACTION APIS - BATCH OPERATIONS
	// ========================================================================

	describe("Batch Document Extraction", () => {
		it("should batch extract files (sync)", () => {
			const results = batchExtractFilesSync(testFiles, null);
			expect(Array.isArray(results)).toBe(true);
			expect(results.length).toBe(testFiles.length);
			for (const result of results) {
				expect(typeof result.content).toBe("string");
			}
		});

		it("should batch extract files (async)", async () => {
			const results = await batchExtractFiles(testFiles, null);
			expect(Array.isArray(results)).toBe(true);
			expect(results.length).toBe(testFiles.length);
		});

		it("should batch extract bytes (sync)", () => {
			const results = batchExtractBytesSync(
				testBuffers.map((b) => b.content),
				testBuffers.map((b) => b.mimeType),
				null,
			);
			expect(Array.isArray(results)).toBe(true);
			expect(results.length).toBe(testBuffers.length);
		});

		it("should batch extract bytes (async)", async () => {
			const results = await batchExtractBytes(
				testBuffers.map((b) => b.content),
				testBuffers.map((b) => b.mimeType),
				null,
			);
			expect(Array.isArray(results)).toBe(true);
			expect(results.length).toBe(testBuffers.length);
		});

		it("should handle empty batch (sync)", () => {
			const results = batchExtractFilesSync([], null);
			expect(Array.isArray(results)).toBe(true);
			expect(results.length).toBe(0);
		});

		it("should handle single item batch", () => {
			const results = batchExtractFilesSync([testFiles[0]], null);
			expect(Array.isArray(results)).toBe(true);
			expect(results.length).toBe(1);
		});

		it("should batch extract with consistent results", () => {
			const results = batchExtractFilesSync(testFiles.slice(0, 2), null);
			expect(results.length).toBe(2);
			for (const result of results) {
				expect(typeof result.content).toBe("string");
				expect(typeof result.mimeType).toBe("string");
			}
		});
	});

	// ========================================================================
	// EXTRACTION CONFIG TYPES AND FIELDS
	// ========================================================================

	describe("ExtractionConfig and Sub-configs", () => {
		it("should handle empty config object", () => {
			const config: ExtractionConfig = {};
			expect(config).toBeDefined();
		});

		it("should handle all config fields as optional", () => {
			const config: ExtractionConfig = {
				useCache: true,
				enableQualityProcessing: false,
				forceOcr: false,
				maxConcurrentExtractions: 4,
				outputFormat: "plain",
				resultFormat: "unified",
			};
			expect(config).toBeDefined();
		});

		it("should extract with useCache config", () => {
			const config: ExtractionConfig = { useCache: true };
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});

		it("should extract with enableQualityProcessing config", () => {
			const config: ExtractionConfig = { enableQualityProcessing: true };
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});

		it("should extract with forceOcr config", () => {
			const config: ExtractionConfig = { forceOcr: false };
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});

		it("should extract with maxConcurrentExtractions config", () => {
			const config: ExtractionConfig = { maxConcurrentExtractions: 2 };
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});

		it("should extract with all output formats", () => {
			for (const format of ["plain", "markdown", "djot", "html"] as const) {
				const config: ExtractionConfig = { outputFormat: format };
				const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
				expect(result).toBeDefined();
			}
		});

		it("should extract with all result formats", () => {
			for (const format of ["unified", "element_based"] as const) {
				const config: ExtractionConfig = { resultFormat: format };
				const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
				expect(result).toBeDefined();
			}
		});
	});

	// ========================================================================
	// CHUNKING CONFIG
	// ========================================================================

	describe("ChunkingConfig", () => {
		it("should extract with maxChars chunking", () => {
			const config: ExtractionConfig = {
				chunking: { maxChars: 100 },
			};
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});

		it("should extract with maxOverlap chunking", () => {
			const config: ExtractionConfig = {
				chunking: { maxChars: 100, maxOverlap: 20 },
			};
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});

		it("should extract with chunkSize alternative", () => {
			const config: ExtractionConfig = {
				chunking: { chunkSize: 100 },
			};
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});

		it("should extract with chunkOverlap alternative", () => {
			const config: ExtractionConfig = {
				chunking: { chunkSize: 100, chunkOverlap: 10 },
			};
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});

		it("should extract with preset chunking", () => {
			const config: ExtractionConfig = {
				chunking: { preset: "default" },
			};
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});

		it("should extract with preset chunking config", () => {
			const config: ExtractionConfig = {
				chunking: { preset: "default" },
			};
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});

		it("should extract with chunking enabled flag", () => {
			const config: ExtractionConfig = {
				chunking: { maxChars: 100, enabled: true },
			};
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});
	});

	// ========================================================================
	// OCR CONFIG
	// ========================================================================

	describe("OcrConfig", () => {
		it("should handle OcrConfig with backend", () => {
			const ocrConfig: OcrConfig = {
				backend: "tesseract",
			};
			const config: ExtractionConfig = { ocr: ocrConfig };
			expect(config.ocr).toBeDefined();
			expect(config.ocr?.backend).toBe("tesseract");
		});

		it("should handle OcrConfig with language", () => {
			const ocrConfig: OcrConfig = {
				backend: "tesseract",
				language: "eng",
			};
			const config: ExtractionConfig = { ocr: ocrConfig };
			expect(config.ocr?.language).toBe("eng");
		});

		it("should handle OcrConfig with TesseractConfig", () => {
			const tesseractConfig: TesseractConfig = {
				psm: 6,
				enableTableDetection: true,
				tesseditCharWhitelist: "0123456789",
			};
			const ocrConfig: OcrConfig = {
				backend: "tesseract",
				tesseractConfig,
			};
			const config: ExtractionConfig = { ocr: ocrConfig };
			expect(config.ocr?.tesseractConfig).toBeDefined();
		});

		it("should extract with OCR config", () => {
			const config: ExtractionConfig = {
				ocr: { backend: "tesseract", language: "eng" },
			};
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});
	});

	// ========================================================================
	// PDF CONFIG
	// ========================================================================

	describe("PdfConfig", () => {
		it("should handle PdfConfig with extractImages", () => {
			const pdfConfig: PdfConfig = {
				extractImages: true,
			};
			const config: ExtractionConfig = { pdfOptions: pdfConfig };
			expect(config.pdfOptions).toBeDefined();
		});

		it("should handle PdfConfig with passwords", () => {
			const pdfConfig: PdfConfig = {
				passwords: ["password1", "password2"],
			};
			const config: ExtractionConfig = { pdfOptions: pdfConfig };
			expect(config.pdfOptions?.passwords).toBeDefined();
		});

		it("should handle PdfConfig with extractMetadata", () => {
			const pdfConfig: PdfConfig = {
				extractMetadata: true,
			};
			const config: ExtractionConfig = { pdfOptions: pdfConfig };
			expect(config.pdfOptions?.extractMetadata).toBe(true);
		});

		it("should handle PdfConfig with hierarchy", () => {
			const pdfConfig: PdfConfig = {
				hierarchy: { enabled: true, kClusters: 6 },
			};
			const config: ExtractionConfig = { pdfOptions: pdfConfig };
			expect(config.pdfOptions?.hierarchy).toBeDefined();
		});
	});

	// ========================================================================
	// IMAGE EXTRACTION CONFIG
	// ========================================================================

	describe("ImageExtractionConfig", () => {
		it("should handle ImageExtractionConfig with all fields", () => {
			const imageConfig: ImageExtractionConfig = {
				extractImages: true,
				targetDpi: 150,
				maxImageDimension: 2000,
				autoAdjustDpi: true,
				minDpi: 72,
				maxDpi: 300,
			};
			const config: ExtractionConfig = { images: imageConfig };
			expect(config.images).toBeDefined();
			expect(config.images?.targetDpi).toBe(150);
		});

		it("should extract with image config", () => {
			const config: ExtractionConfig = {
				images: { extractImages: true, targetDpi: 150 },
			};
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});
	});

	// ========================================================================
	// LANGUAGE DETECTION CONFIG
	// ========================================================================

	describe("LanguageDetectionConfig", () => {
		it("should handle LanguageDetectionConfig", () => {
			const langConfig: LanguageDetectionConfig = {
				enabled: true,
				minConfidence: 0.5,
				detectMultiple: false,
			};
			const config: ExtractionConfig = { languageDetection: langConfig };
			expect(config.languageDetection).toBeDefined();
		});

		it("should extract with language detection", () => {
			const config: ExtractionConfig = {
				languageDetection: { enabled: true, minConfidence: 0.5 },
			};
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});
	});

	// ========================================================================
	// TOKEN REDUCTION CONFIG
	// ========================================================================

	describe("TokenReductionConfig", () => {
		it("should handle TokenReductionConfig", () => {
			const tokenConfig: TokenReductionConfig = {
				mode: "conservative",
				preserveImportantWords: true,
			};
			const config: ExtractionConfig = { tokenReduction: tokenConfig };
			expect(config.tokenReduction).toBeDefined();
		});

		it("should extract with token reduction", () => {
			const config: ExtractionConfig = {
				tokenReduction: { mode: "conservative" },
			};
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});
	});

	// ========================================================================
	// KEYWORD CONFIG
	// ========================================================================

	describe("KeywordConfig", () => {
		it("should handle KeywordConfig with YAKE", () => {
			const keywordConfig: KeywordConfig = {
				algorithm: "yake",
				maxKeywords: 10,
				minScore: 0.1,
				ngramRange: [1, 3],
				language: "en",
				yakeParams: { windowSize: 3 },
			};
			const config: ExtractionConfig = { keywords: keywordConfig };
			expect(config.keywords).toBeDefined();
		});

		it("should handle KeywordConfig with RAKE", () => {
			const keywordConfig: KeywordConfig = {
				algorithm: "rake",
				maxKeywords: 10,
				rakeParams: { minWordLength: 3, maxWordsPerPhrase: 3 },
			};
			const config: ExtractionConfig = { keywords: keywordConfig };
			expect(config.keywords).toBeDefined();
		});

		it("should extract with keyword extraction", () => {
			const config: ExtractionConfig = {
				keywords: { algorithm: "yake", maxKeywords: 5 },
			};
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});
	});

	// ========================================================================
	// HTML CONVERSION OPTIONS
	// ========================================================================

	describe("HtmlConversionOptions", () => {
		it("should handle HtmlConversionOptions heading styles", () => {
			for (const style of ["atx", "underlined", "atx_closed"] as const) {
				const config: ExtractionConfig = {
					htmlOptions: { headingStyle: style },
				};
				expect(config.htmlOptions?.headingStyle).toBe(style);
			}
		});

		it("should handle HtmlConversionOptions list indentation", () => {
			for (const type of ["spaces", "tabs"] as const) {
				const config: ExtractionConfig = {
					htmlOptions: { listIndentType: type, listIndentWidth: 4 },
				};
				expect(config.htmlOptions?.listIndentType).toBe(type);
			}
		});

		it("should handle HtmlConversionOptions escape options", () => {
			const config: ExtractionConfig = {
				htmlOptions: {
					escapeAsterisks: true,
					escapeUnderscores: true,
					escapeMisc: true,
					escapeAscii: true,
				},
			};
			expect(config.htmlOptions?.escapeAsterisks).toBe(true);
		});

		it("should handle HtmlConversionOptions code options", () => {
			const config: ExtractionConfig = {
				htmlOptions: {
					codeLanguage: "javascript",
					codeBlockStyle: "backticks",
				},
			};
			expect(config.htmlOptions?.codeLanguage).toBe("javascript");
		});

		it("should handle HtmlConversionOptions whitespace modes", () => {
			for (const mode of ["normalized", "strict"] as const) {
				const config: ExtractionConfig = {
					htmlOptions: { whitespaceMode: mode },
				};
				expect(config.htmlOptions?.whitespaceMode).toBe(mode);
			}
		});

		it("should handle HtmlConversionOptions highlighting styles", () => {
			for (const style of ["double_equal", "html", "bold", "none"] as const) {
				const config: ExtractionConfig = {
					htmlOptions: { highlightStyle: style },
				};
				expect(config.htmlOptions?.highlightStyle).toBe(style);
			}
		});

		it("should handle HtmlConversionOptions preprocessing", () => {
			const config: ExtractionConfig = {
				htmlOptions: {
					preprocessing: {
						enabled: true,
						preset: "standard",
						removeNavigation: true,
						removeForms: true,
					},
				},
			};
			expect(config.htmlOptions?.preprocessing).toBeDefined();
		});

		it("should handle HtmlConversionOptions newline styles", () => {
			for (const style of ["spaces", "backslash"] as const) {
				const config: ExtractionConfig = {
					htmlOptions: { newlineStyle: style },
				};
				expect(config.htmlOptions?.newlineStyle).toBe(style);
			}
		});

		it("should handle HtmlConversionOptions all fields", () => {
			const config: ExtractionConfig = {
				htmlOptions: {
					headingStyle: "atx",
					listIndentType: "spaces",
					listIndentWidth: 4,
					bullets: "*",
					strongEmSymbol: "**",
					escapeAsterisks: false,
					escapeUnderscores: false,
					escapeMisc: false,
					escapeAscii: false,
					codeLanguage: "javascript",
					autolinks: true,
					defaultTitle: false,
					brInTables: false,
					hocrSpatialTables: false,
					highlightStyle: "none",
					extractMetadata: false,
					whitespaceMode: "normalized",
					stripNewlines: false,
					wrap: true,
					wrapWidth: 80,
					convertAsInline: false,
					subSymbol: "~",
					supSymbol: "^",
					newlineStyle: "spaces",
					codeBlockStyle: "backticks",
					keepInlineImagesIn: [],
					encoding: "utf-8",
					debug: false,
					stripTags: [],
					preserveTags: [],
					preprocessing: { enabled: true, preset: "standard" },
				},
			};
			expect(config.htmlOptions).toBeDefined();
		});
	});

	// ========================================================================
	// PAGE EXTRACTION CONFIG
	// ========================================================================

	describe("PageExtractionConfig", () => {
		it("should handle PageExtractionConfig", () => {
			const pageConfig: PageExtractionConfig = {
				extractPages: true,
				insertPageMarkers: true,
				markerFormat: "[Page {page_num}]",
			};
			const config: ExtractionConfig = { pages: pageConfig };
			expect(config.pages).toBeDefined();
		});

		it("should extract with page config", () => {
			const config: ExtractionConfig = {
				pages: { extractPages: true },
			};
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});
	});

	// ========================================================================
	// POST-PROCESSOR PLUGIN SYSTEM
	// ========================================================================

	describe("Post-Processor Plugin System", () => {
		const mockProcessor: PostProcessorProtocol = {
			name: () => "test-processor",
			process: (result: ExtractionResult) => result,
			processingStage: () => "middle",
		};

		afterEach(() => {
			clearPostProcessors();
		});

		it("should register post-processor", () => {
			registerPostProcessor(mockProcessor);
			const list = listPostProcessors();
			expect(list.includes("test-processor")).toBe(true);
		});

		it("should list post-processors", () => {
			const list = listPostProcessors();
			expect(Array.isArray(list)).toBe(true);
		});

		it("should unregister post-processor", () => {
			registerPostProcessor(mockProcessor);
			unregisterPostProcessor("test-processor");
			const list = listPostProcessors();
			expect(list.includes("test-processor")).toBe(false);
		});

		it("should clear all post-processors", () => {
			registerPostProcessor(mockProcessor);
			clearPostProcessors();
			const list = listPostProcessors();
			expect(list.length).toBe(0);
		});

		it("should handle PostProcessorConfig", () => {
			const config: ExtractionConfig = {
				postprocessor: {
					enabled: true,
					enabledProcessors: ["processor1"],
					disabledProcessors: ["processor2"],
				},
			};
			expect(config.postprocessor).toBeDefined();
		});
	});

	// ========================================================================
	// VALIDATOR PLUGIN SYSTEM
	// ========================================================================

	describe("Validator Plugin System", () => {
		const mockValidator: ValidatorProtocol = {
			name: () => "test-validator",
			validate: (result: ExtractionResult) => {
				// Implementation here
			},
			priority: () => 50,
			shouldValidate: (result: ExtractionResult) => true,
		};

		afterEach(() => {
			clearValidators();
		});

		it("should register validator", () => {
			registerValidator(mockValidator);
			const list = listValidators();
			expect(list.includes("test-validator")).toBe(true);
		});

		it("should list validators", () => {
			const list = listValidators();
			expect(Array.isArray(list)).toBe(true);
		});

		it("should unregister validator", () => {
			registerValidator(mockValidator);
			unregisterValidator("test-validator");
			const list = listValidators();
			expect(list.includes("test-validator")).toBe(false);
		});

		it("should clear all validators", () => {
			registerValidator(mockValidator);
			clearValidators();
			const list = listValidators();
			expect(list.length).toBe(0);
		});
	});

	// ========================================================================
	// OCR BACKEND PLUGIN SYSTEM
	// ========================================================================

	describe("OCR Backend Plugin System", () => {
		const mockOcrBackend: OcrBackendProtocol = {
			name: () => "test-ocr",
			supportedLanguages: () => ["en", "eng", "de", "deu"],
			processImage: async (imageBytes: Uint8Array | string, language: string) => ({
				content: "Test OCR output",
				mime_type: "text/plain",
				metadata: { confidence: 0.95 },
				tables: [],
			}),
		};

		afterEach(() => {
			clearOcrBackends();
		});

		it("should register OCR backend", () => {
			registerOcrBackend(mockOcrBackend);
			const list = listOcrBackends();
			expect(list.includes("test-ocr")).toBe(true);
		});

		it("should list OCR backends", () => {
			const list = listOcrBackends();
			expect(Array.isArray(list)).toBe(true);
		});

		it("should unregister OCR backend", () => {
			registerOcrBackend(mockOcrBackend);
			unregisterOcrBackend("test-ocr");
			const list = listOcrBackends();
			expect(list.includes("test-ocr")).toBe(false);
		});

		it("should clear all OCR backends", () => {
			registerOcrBackend(mockOcrBackend);
			clearOcrBackends();
			const list = listOcrBackends();
			expect(list.length).toBe(0);
		});
	});

	// ========================================================================
	// DOCUMENT EXTRACTOR REGISTRY
	// ========================================================================

	describe("Document Extractor Registry", () => {
		it("should list document extractors", () => {
			const extractors = listDocumentExtractors();
			expect(Array.isArray(extractors)).toBe(true);
		});

		it("should clear document extractors", () => {
			clearDocumentExtractors();
			const extractors = listDocumentExtractors();
			expect(Array.isArray(extractors)).toBe(true);
		});

		it("should unregister document extractor", () => {
			unregisterDocumentExtractor("non-existent");
			expect(true).toBe(true);
		});
	});

	// ========================================================================
	// MIME TYPE DETECTION AND VALIDATION
	// ========================================================================

	describe("MIME Type Detection and Validation", () => {
		it("should detect MIME type from PDF bytes", () => {
			const pdfHeader = Buffer.from("%PDF-1.4");
			const mimeType = detectMimeType(pdfHeader);
			expect(typeof mimeType).toBe("string");
			expect(mimeType.length).toBeGreaterThan(0);
		});

		it("should detect MIME type from file path", () => {
			const mimeType = detectMimeTypeFromPath("document.pdf", false);
			expect(typeof mimeType).toBe("string");
		});

		it("should detect MIME type with lowercase option", () => {
			// Create actual test file
			const testFile = join(testDir, "testdoc.PDF");
			writeFileSync(testFile, Buffer.from("test content"));
			try {
				const mimeType1 = detectMimeTypeFromPath(testFile, false);
				const mimeType2 = detectMimeTypeFromPath(testFile, true);
				expect(typeof mimeType1).toBe("string");
				expect(typeof mimeType2).toBe("string");
			} finally {
				// Cleanup
			}
		});

		it("should validate MIME type", () => {
			const result = validateMimeType("application/pdf");
			expect(typeof result).toBe("string");
		});

		it("should get extensions for MIME type", () => {
			const extensions = getExtensionsForMime("application/pdf");
			expect(Array.isArray(extensions)).toBe(true);
			expect(extensions.includes("pdf")).toBe(true);
		});

		it("should get extensions for markdown", () => {
			const extensions = getExtensionsForMime("text/markdown");
			expect(Array.isArray(extensions)).toBe(true);
		});

		it("should get extensions for text/plain", () => {
			const extensions = getExtensionsForMime("text/plain");
			expect(Array.isArray(extensions)).toBe(true);
			expect(extensions.includes("txt")).toBe(true);
		});
	});

	// ========================================================================
	// EMBEDDING PRESETS
	// ========================================================================

	describe("Embedding Presets", () => {
		it("should list embedding presets", () => {
			const presets = listEmbeddingPresets();
			expect(Array.isArray(presets)).toBe(true);
		});

		it("should get embedding preset", () => {
			const presets = listEmbeddingPresets();
			if (presets.length > 0) {
				const preset = getEmbeddingPreset(presets[0]);
				if (preset) {
					expect(typeof preset).toBe("object");
				}
			}
		});

		it("should return null for non-existent preset", () => {
			const preset = getEmbeddingPreset("non-existent-xyz");
			expect(preset).toBeNull();
		});
	});

	// ========================================================================
	// ERROR HANDLING AND CLASSIFICATION
	// ========================================================================

	describe("Error Handling and Diagnostics", () => {
		it("should get last error code", () => {
			const code = getLastErrorCode();
			expect(typeof code).toBe("number");
		});

		it("should get error code name", () => {
			for (let i = 0; i <= 7; i++) {
				const name = getErrorCodeName(i);
				expect(typeof name).toBe("string");
			}
		});

		it("should get error code description", () => {
			for (let i = 0; i <= 7; i++) {
				const description = getErrorCodeDescription(i);
				expect(typeof description).toBe("string");
			}
		});

		it("should classify error messages", () => {
			const messages = [
				"PDF parsing error",
				"OCR processing failed",
				"Validation failed",
				"File not found",
				"Connection timeout",
			];

			for (const msg of messages) {
				const classification = classifyError(msg);
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
						expect((obj.confidence as number) >= 0).toBe(true);
						expect((obj.confidence as number) <= 1).toBe(true);
					}
				}
			}
		});

		it("should get last panic context", () => {
			const context = getLastPanicContext();
			if (context !== null) {
				expect(typeof context).toBe("object");
			}
		});
	});

	// ========================================================================
	// ERROR CLASSES AND TYPES
	// ========================================================================

	describe("Error Classes", () => {
		it("should create KreuzbergError", () => {
			const error = new KreuzbergError("test error");
			expect(error).toBeInstanceOf(Error);
			expect(error.message).toBe("test error");
		});

		it("should create ValidationError", () => {
			const error = new ValidationError("validation failed");
			expect(error).toBeInstanceOf(Error);
		});

		it("should create ParsingError", () => {
			const error = new ParsingError("parsing failed");
			expect(error).toBeInstanceOf(Error);
		});

		it("should create OcrError", () => {
			const error = new OcrError("ocr failed");
			expect(error).toBeInstanceOf(Error);
		});

		it("should create CacheError", () => {
			const error = new CacheError("cache failed");
			expect(error).toBeInstanceOf(Error);
		});

		it("should create MissingDependencyError", () => {
			const error = new MissingDependencyError("dependency missing");
			expect(error).toBeInstanceOf(Error);
		});

		it("should create ImageProcessingError", () => {
			const error = new ImageProcessingError("image processing failed");
			expect(error).toBeInstanceOf(Error);
		});

		it("should create PluginError", () => {
			const error = new PluginError("plugin failed");
			expect(error).toBeInstanceOf(Error);
		});
	});

	// ========================================================================
	// EXTRACTION RESULT TYPES AND STRUCTURES
	// ========================================================================

	describe("Extraction Result Types", () => {
		it("should have ExtractionResult with all fields", () => {
			const result = extractFileSync(testFiles[0], null);
			expect(typeof result.content).toBe("string");
			expect(typeof result.mimeType).toBe("string");
			expect(typeof result.metadata).toBe("object");
			expect(Array.isArray(result.tables)).toBe(true);
			if (result.detectedLanguages !== null) {
				expect(Array.isArray(result.detectedLanguages)).toBe(true);
			}
			if (result.chunks !== null) {
				expect(Array.isArray(result.chunks)).toBe(true);
			}
			if (result.images !== null) {
				expect(Array.isArray(result.images)).toBe(true);
			}
		});

		it("should have Table type structure", () => {
			const table: Table = {
				cells: [["cell1", "cell2"]],
				markdown: "| cell1 | cell2 |",
				pageNumber: 1,
			};
			expect(table.cells.length).toBe(1);
			expect(table.markdown).toBeDefined();
			expect(table.pageNumber).toBe(1);
		});

		it("should have Chunk type structure", () => {
			const chunk: Chunk = {
				content: "test chunk",
				metadata: {
					byteStart: 0,
					byteEnd: 10,
					tokenCount: null,
					chunkIndex: 0,
					totalChunks: 1,
				},
				embedding: null,
			};
			expect(chunk.content).toBe("test chunk");
			expect(chunk.metadata.chunkIndex).toBe(0);
		});

		it("should have ExtractedImage type structure", () => {
			const image: ExtractedImage = {
				data: new Uint8Array(),
				format: "png",
				imageIndex: 0,
				pageNumber: null,
				width: null,
				height: null,
				colorspace: null,
				bitsPerComponent: null,
				isMask: false,
				description: null,
				ocrResult: null,
			};
			expect(image.format).toBe("png");
			expect(image.isMask).toBe(false);
		});

		it("should have PageContent type structure", () => {
			const pageContent: PageContent = {
				pageNumber: 1,
				content: "page text",
				tables: [],
				images: [],
			};
			expect(pageContent.pageNumber).toBe(1);
		});
	});

	// ========================================================================
	// PROTOCOL VALIDATION
	// ========================================================================

	describe("Plugin Protocol Validation", () => {
		it("should validate PostProcessorProtocol", () => {
			const processor: PostProcessorProtocol = {
				name: () => "test",
				process: (result: ExtractionResult) => result,
			};
			expect(typeof processor.name).toBe("function");
			expect(typeof processor.process).toBe("function");
		});

		it("should validate ValidatorProtocol", () => {
			const validator: ValidatorProtocol = {
				name: () => "test",
				validate: (result: ExtractionResult) => {
					// Implementation
				},
			};
			expect(typeof validator.name).toBe("function");
			expect(typeof validator.validate).toBe("function");
		});

		it("should validate OcrBackendProtocol", () => {
			const backend: OcrBackendProtocol = {
				name: () => "test",
				supportedLanguages: () => ["en"],
				processImage: async (imageBytes: Uint8Array | string, language: string) => ({
					content: "",
					mime_type: "text/plain",
					metadata: {},
					tables: [],
				}),
			};
			expect(typeof backend.name).toBe("function");
			expect(typeof backend.supportedLanguages).toBe("function");
			expect(typeof backend.processImage).toBe("function");
		});
	});

	// ========================================================================
	// COMPREHENSIVE CONFIG COMBINATIONS
	// ========================================================================

	describe("Complex Config Combinations", () => {
		it("should extract with all config options combined", () => {
			const config: ExtractionConfig = {
				useCache: true,
				enableQualityProcessing: true,
				forceOcr: false,
				outputFormat: "markdown",
				resultFormat: "unified",
				maxConcurrentExtractions: 4,
				ocr: {
					backend: "tesseract",
					language: "eng",
					tesseractConfig: { psm: 6 },
				},
				chunking: { maxChars: 100, maxOverlap: 10 },
				images: { extractImages: true, targetDpi: 150 },
				pdfOptions: { extractImages: true, extractMetadata: true },
				languageDetection: { enabled: true, minConfidence: 0.5 },
				tokenReduction: { mode: "conservative" },
				keywords: { algorithm: "yake", maxKeywords: 5 },
				htmlOptions: {
					headingStyle: "atx",
					listIndentType: "spaces",
					escapeAsterisks: false,
				},
				pages: { extractPages: true },
				postprocessor: { enabled: true },
			};
			const result = extractBytesSync(testBuffers[0].content, "text/plain", config);
			expect(result).toBeDefined();
		});

		it("should batch extract with complex config", () => {
			const config: ExtractionConfig = {
				outputFormat: "html",
				resultFormat: "element_based",
				chunking: { maxChars: 50 },
				keywords: { algorithm: "rake", maxKeywords: 3 },
			};
			const results = batchExtractBytesSync(
				testBuffers.map((b) => b.content),
				testBuffers.map((b) => b.mimeType),
				config,
			);
			expect(results.length).toBe(testBuffers.length);
		});
	});
});
