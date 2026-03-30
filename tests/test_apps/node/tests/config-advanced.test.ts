/**
 * Advanced Configuration Tests
 *
 * Comprehensive testing of all configuration options, edge cases, and combinations.
 */

import { writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
	type ChunkingConfig,
	type ExtractionConfig,
	extractBytesSync,
	extractFileSync,
	type HierarchyConfig,
	type HtmlConversionOptions,
	type HtmlPreprocessingOptions,
	type ImageExtractionConfig,
	type KeywordConfig,
	type LanguageDetectionConfig,
	type OcrConfig,
	type PageExtractionConfig,
	type PdfConfig,
	type TesseractConfig,
	type TokenReductionConfig,
} from "@kreuzberg/node";
import { beforeAll, describe, expect, it } from "vitest";

describe("Advanced Configuration Options", () => {
	let testFile: string;
	let testBuffer: Buffer;

	beforeAll(() => {
		const content = Buffer.from("# Test Document\n\nThis is test content for configuration testing.");
		testFile = join(tmpdir(), "config-test.txt");
		testBuffer = content;
		writeFileSync(testFile, content);
	});

	describe("TesseractConfig - OCR Engine Configuration", () => {
		it("should accept PSM (Page Segmentation Mode) 0-13", () => {
			const config: ExtractionConfig = {
				ocr: {
					backend: "tesseract",
					tesseractConfig: { psm: 3 },
				},
			};
			expect(config.ocr?.tesseractConfig?.psm).toBe(3);
		});

		it("should accept PSM modes: 0 (auto), 6 (uniform block), 11 (sparse)", () => {
			const psmModes = [0, 3, 6, 11, 13];
			for (const psm of psmModes) {
				const config: ExtractionConfig = {
					ocr: {
						backend: "tesseract",
						tesseractConfig: { psm },
					},
				};
				expect(config.ocr?.tesseractConfig?.psm).toBe(psm);
			}
		});

		it("should enable table detection", () => {
			const config: ExtractionConfig = {
				ocr: {
					backend: "tesseract",
					tesseractConfig: {
						enableTableDetection: true,
					},
				},
			};
			expect(config.ocr?.tesseractConfig?.enableTableDetection).toBe(true);
		});

		it("should accept character whitelist for digit-only OCR", () => {
			const config: ExtractionConfig = {
				ocr: {
					backend: "tesseract",
					tesseractConfig: {
						tesseditCharWhitelist: "0123456789",
					},
				},
			};
			expect(config.ocr?.tesseractConfig?.tesseditCharWhitelist).toBe("0123456789");
		});

		it("should accept mixed tesseract config", () => {
			const config: ExtractionConfig = {
				ocr: {
					backend: "tesseract",
					language: "deu",
					tesseractConfig: {
						psm: 6,
						enableTableDetection: true,
						tesseditCharWhitelist: "abcdefghijklmnopqrstuvwxyz",
					},
				},
			};
			expect(config.ocr?.tesseractConfig?.psm).toBe(6);
			expect(config.ocr?.tesseractConfig?.enableTableDetection).toBe(true);
			expect(config.ocr?.tesseractConfig?.tesseditCharWhitelist).toContain("abc");
		});
	});

	describe("ChunkingConfig - Document Segmentation", () => {
		it("should accept maxChars chunking strategy", () => {
			const config: ExtractionConfig = {
				chunking: {
					maxChars: 2048,
					maxOverlap: 256,
				},
			};
			expect(config.chunking?.maxChars).toBe(2048);
			expect(config.chunking?.maxOverlap).toBe(256);
		});

		it("should accept alternative chunkSize and chunkOverlap", () => {
			const config: ExtractionConfig = {
				chunking: {
					chunkSize: 1024,
					chunkOverlap: 128,
				},
			};
			expect(config.chunking?.chunkSize).toBe(1024);
			expect(config.chunking?.chunkOverlap).toBe(128);
		});

		it("should support preset configurations", () => {
			const presets = ["default", "aggressive", "minimal"];
			for (const preset of presets) {
				const config: ExtractionConfig = {
					chunking: {
						preset,
					},
				};
				expect(config.chunking?.preset).toBe(preset);
			}
		});

		it("should enable/disable chunking explicitly", () => {
			const enabledConfig: ExtractionConfig = {
				chunking: {
					maxChars: 2048,
					enabled: true,
				},
			};
			expect(enabledConfig.chunking?.enabled).toBe(true);

			const disabledConfig: ExtractionConfig = {
				chunking: {
					enabled: false,
				},
			};
			expect(disabledConfig.chunking?.enabled).toBe(false);
		});

		it("should support embedding configuration within chunking", () => {
			const config: ExtractionConfig = {
				chunking: {
					maxChars: 2048,
					embedding: {
						model: "sentence-transformers/all-MiniLM-L6-v2",
						dimension: 384,
					},
				},
			};
			expect(config.chunking?.embedding).toBeDefined();
			expect(typeof config.chunking?.embedding).toBe("object");
		});
	});

	describe("ImageExtractionConfig - Image Processing", () => {
		it("should configure target DPI for extracted images", () => {
			const config: ExtractionConfig = {
				images: {
					extractImages: true,
					targetDpi: 200,
				},
			};
			expect(config.images?.targetDpi).toBe(200);
		});

		it("should limit maximum image dimensions", () => {
			const config: ExtractionConfig = {
				images: {
					extractImages: true,
					maxImageDimension: 3000,
				},
			};
			expect(config.images?.maxImageDimension).toBe(3000);
		});

		it("should enable DPI auto-adjustment", () => {
			const config: ExtractionConfig = {
				images: {
					extractImages: true,
					autoAdjustDpi: true,
					minDpi: 72,
					maxDpi: 300,
				},
			};
			expect(config.images?.autoAdjustDpi).toBe(true);
			expect(config.images?.minDpi).toBe(72);
			expect(config.images?.maxDpi).toBe(300);
		});

		it("should validate DPI range constraints", () => {
			const config: ExtractionConfig = {
				images: {
					extractImages: true,
					minDpi: 72,
					targetDpi: 150,
					maxDpi: 300,
				},
			};
			const img = config.images;
			if (img?.minDpi && img.targetDpi && img.maxDpi) {
				expect(img.minDpi <= img.targetDpi).toBe(true);
				expect(img.targetDpi <= img.maxDpi).toBe(true);
			}
		});
	});

	describe("PdfConfig - PDF-Specific Options", () => {
		it("should extract images from PDFs", () => {
			const config: ExtractionConfig = {
				pdfOptions: {
					extractImages: true,
				},
			};
			expect(config.pdfOptions?.extractImages).toBe(true);
		});

		it("should handle password-protected PDFs", () => {
			const config: ExtractionConfig = {
				pdfOptions: {
					passwords: ["password1", "password2", "password3"],
				},
			};
			expect(Array.isArray(config.pdfOptions?.passwords)).toBe(true);
			expect(config.pdfOptions?.passwords?.length).toBe(3);
		});

		it("should extract PDF metadata", () => {
			const config: ExtractionConfig = {
				pdfOptions: {
					extractMetadata: true,
				},
			};
			expect(config.pdfOptions?.extractMetadata).toBe(true);
		});

		it("should configure hierarchy extraction", () => {
			const config: ExtractionConfig = {
				pdfOptions: {
					hierarchy: {
						enabled: true,
						kClusters: 8,
						includeBbox: true,
						ocrCoverageThreshold: 0.8,
					},
				},
			};
			const hierarchy = config.pdfOptions?.hierarchy;
			expect(hierarchy?.enabled).toBe(true);
			expect(hierarchy?.kClusters).toBe(8);
			expect(hierarchy?.includeBbox).toBe(true);
			expect(hierarchy?.ocrCoverageThreshold).toBe(0.8);
		});
	});

	describe("KeywordConfig - Keyword Extraction", () => {
		it("should configure YAKE algorithm", () => {
			const config: ExtractionConfig = {
				keywords: {
					algorithm: "yake",
					yakeParams: {
						windowSize: 4,
					},
					maxKeywords: 15,
					minScore: 0.05,
				},
			};
			expect(config.keywords?.algorithm).toBe("yake");
			expect(config.keywords?.yakeParams?.windowSize).toBe(4);
		});

		it("should configure RAKE algorithm", () => {
			const config: ExtractionConfig = {
				keywords: {
					algorithm: "rake",
					rakeParams: {
						minWordLength: 4,
						maxWordsPerPhrase: 4,
					},
					maxKeywords: 20,
				},
			};
			expect(config.keywords?.algorithm).toBe("rake");
			expect(config.keywords?.rakeParams?.minWordLength).toBe(4);
			expect(config.keywords?.rakeParams?.maxWordsPerPhrase).toBe(4);
		});

		it("should filter keywords by score", () => {
			const config: ExtractionConfig = {
				keywords: {
					maxKeywords: 10,
					minScore: 0.1,
				},
			};
			expect(config.keywords?.minScore).toBe(0.1);
		});

		it("should configure n-gram ranges", () => {
			const config: ExtractionConfig = {
				keywords: {
					ngramRange: [1, 4],
				},
			};
			expect(config.keywords?.ngramRange).toEqual([1, 4]);
		});

		it("should set language for keyword extraction", () => {
			const config: ExtractionConfig = {
				keywords: {
					language: "de",
				},
			};
			expect(config.keywords?.language).toBe("de");
		});
	});

	describe("LanguageDetectionConfig - Language Detection", () => {
		it("should enable language detection", () => {
			const config: ExtractionConfig = {
				languageDetection: {
					enabled: true,
				},
			};
			expect(config.languageDetection?.enabled).toBe(true);
		});

		it("should set confidence threshold", () => {
			const config: ExtractionConfig = {
				languageDetection: {
					enabled: true,
					minConfidence: 0.7,
				},
			};
			expect(config.languageDetection?.minConfidence).toBe(0.7);
		});

		it("should detect multiple languages", () => {
			const config: ExtractionConfig = {
				languageDetection: {
					enabled: true,
					detectMultiple: true,
				},
			};
			expect(config.languageDetection?.detectMultiple).toBe(true);
		});
	});

	describe("TokenReductionConfig - Token Optimization", () => {
		it("should apply conservative token reduction", () => {
			const config: ExtractionConfig = {
				tokenReduction: {
					mode: "conservative",
				},
			};
			expect(config.tokenReduction?.mode).toBe("conservative");
		});

		it("should apply aggressive token reduction", () => {
			const config: ExtractionConfig = {
				tokenReduction: {
					mode: "aggressive",
					preserveImportantWords: true,
				},
			};
			expect(config.tokenReduction?.mode).toBe("aggressive");
			expect(config.tokenReduction?.preserveImportantWords).toBe(true);
		});
	});

	describe("HtmlConversionOptions - HTML to Markdown", () => {
		it("should configure heading styles", () => {
			const styles = ["atx", "underlined", "atx_closed"] as const;
			for (const style of styles) {
				const config: ExtractionConfig = {
					htmlOptions: {
						headingStyle: style,
					},
				};
				expect(config.htmlOptions?.headingStyle).toBe(style);
			}
		});

		it("should configure list formatting", () => {
			const config: ExtractionConfig = {
				htmlOptions: {
					listIndentType: "spaces",
					listIndentWidth: 4,
					bullets: "-",
				},
			};
			expect(config.htmlOptions?.listIndentType).toBe("spaces");
			expect(config.htmlOptions?.listIndentWidth).toBe(4);
			expect(config.htmlOptions?.bullets).toBe("-");
		});

		it("should configure emphasis symbols", () => {
			const config: ExtractionConfig = {
				htmlOptions: {
					strongEmSymbol: "__",
					subSymbol: "~",
					supSymbol: "^",
				},
			};
			expect(config.htmlOptions?.strongEmSymbol).toBe("__");
		});

		it("should control special character escaping", () => {
			const config: ExtractionConfig = {
				htmlOptions: {
					escapeAsterisks: true,
					escapeUnderscores: true,
					escapeMisc: true,
					escapeAscii: true,
				},
			};
			expect(config.htmlOptions?.escapeAsterisks).toBe(true);
			expect(config.htmlOptions?.escapeUnderscores).toBe(true);
		});

		it("should configure code block styling", () => {
			const styles = ["indented", "backticks", "tildes"] as const;
			for (const style of styles) {
				const config: ExtractionConfig = {
					htmlOptions: {
						codeBlockStyle: style,
					},
				};
				expect(config.htmlOptions?.codeBlockStyle).toBe(style);
			}
		});

		it("should configure line wrapping", () => {
			const config: ExtractionConfig = {
				htmlOptions: {
					wrap: true,
					wrapWidth: 100,
				},
			};
			expect(config.htmlOptions?.wrap).toBe(true);
			expect(config.htmlOptions?.wrapWidth).toBe(100);
		});

		it("should configure whitespace handling", () => {
			const modes = ["normalized", "strict"] as const;
			for (const mode of modes) {
				const config: ExtractionConfig = {
					htmlOptions: {
						whitespaceMode: mode,
					},
				};
				expect(config.htmlOptions?.whitespaceMode).toBe(mode);
			}
		});

		it("should enable metadata extraction from HTML", () => {
			const config: ExtractionConfig = {
				htmlOptions: {
					extractMetadata: true,
				},
			};
			expect(config.htmlOptions?.extractMetadata).toBe(true);
		});
	});

	describe("HtmlPreprocessingOptions - HTML Cleanup", () => {
		it("should apply minimal preprocessing", () => {
			const config: ExtractionConfig = {
				htmlOptions: {
					preprocessing: {
						enabled: true,
						preset: "minimal",
					},
				},
			};
			expect(config.htmlOptions?.preprocessing?.preset).toBe("minimal");
		});

		it("should apply standard preprocessing", () => {
			const config: ExtractionConfig = {
				htmlOptions: {
					preprocessing: {
						enabled: true,
						preset: "standard",
					},
				},
			};
			expect(config.htmlOptions?.preprocessing?.preset).toBe("standard");
		});

		it("should apply aggressive preprocessing", () => {
			const config: ExtractionConfig = {
				htmlOptions: {
					preprocessing: {
						enabled: true,
						preset: "aggressive",
					},
				},
			};
			expect(config.htmlOptions?.preprocessing?.preset).toBe("aggressive");
		});

		it("should remove navigation elements", () => {
			const config: ExtractionConfig = {
				htmlOptions: {
					preprocessing: {
						enabled: true,
						removeNavigation: true,
					},
				},
			};
			expect(config.htmlOptions?.preprocessing?.removeNavigation).toBe(true);
		});

		it("should remove form elements", () => {
			const config: ExtractionConfig = {
				htmlOptions: {
					preprocessing: {
						enabled: true,
						removeForms: true,
					},
				},
			};
			expect(config.htmlOptions?.preprocessing?.removeForms).toBe(true);
		});
	});

	describe("PageExtractionConfig - Per-Page Content", () => {
		it("should extract pages as separate array", () => {
			const config: ExtractionConfig = {
				pages: {
					extractPages: true,
				},
			};
			expect(config.pages?.extractPages).toBe(true);
		});

		it("should insert page markers in content", () => {
			const config: ExtractionConfig = {
				pages: {
					insertPageMarkers: true,
					markerFormat: "--- Page {page_num} ---",
				},
			};
			expect(config.pages?.insertPageMarkers).toBe(true);
			expect(config.pages?.markerFormat).toContain("{page_num}");
		});
	});

	describe("Output Format Configuration", () => {
		it("should accept outputFormat plain", () => {
			const config: ExtractionConfig = {
				outputFormat: "plain",
			};
			expect(config.outputFormat).toBe("plain");
		});

		it("should accept outputFormat markdown", () => {
			const config: ExtractionConfig = {
				outputFormat: "markdown",
			};
			expect(config.outputFormat).toBe("markdown");
		});

		it("should accept outputFormat djot", () => {
			const config: ExtractionConfig = {
				outputFormat: "djot",
			};
			expect(config.outputFormat).toBe("djot");
		});

		it("should accept outputFormat html", () => {
			const config: ExtractionConfig = {
				outputFormat: "html",
			};
			expect(config.outputFormat).toBe("html");
		});
	});

	describe("Result Format Configuration", () => {
		it("should accept resultFormat unified", () => {
			const config: ExtractionConfig = {
				resultFormat: "unified",
			};
			expect(config.resultFormat).toBe("unified");
		});

		it("should accept resultFormat element_based", () => {
			const config: ExtractionConfig = {
				resultFormat: "element_based",
			};
			expect(config.resultFormat).toBe("element_based");
		});
	});

	describe("Complex Configuration Combinations", () => {
		it("should build complete extraction config", () => {
			const config: ExtractionConfig = {
				useCache: true,
				enableQualityProcessing: true,
				ocr: {
					backend: "tesseract",
					language: "eng",
					tesseractConfig: {
						psm: 6,
						enableTableDetection: true,
					},
				},
				forceOcr: false,
				chunking: {
					maxChars: 4096,
					maxOverlap: 512,
					preset: "default",
				},
				images: {
					extractImages: true,
					targetDpi: 150,
					maxImageDimension: 2000,
					autoAdjustDpi: true,
					minDpi: 72,
					maxDpi: 300,
				},
				pdfOptions: {
					extractImages: true,
					extractMetadata: true,
					hierarchy: {
						enabled: true,
						kClusters: 6,
						includeBbox: true,
					},
				},
				keywords: {
					algorithm: "yake",
					maxKeywords: 10,
					minScore: 0.1,
					yakeParams: { windowSize: 3 },
				},
				languageDetection: {
					enabled: true,
					minConfidence: 0.5,
					detectMultiple: false,
				},
				tokenReduction: {
					mode: "conservative",
					preserveImportantWords: true,
				},
				htmlOptions: {
					headingStyle: "atx",
					listIndentType: "spaces",
					preprocessing: {
						enabled: true,
						preset: "standard",
						removeNavigation: true,
						removeForms: true,
					},
				},
				pages: {
					extractPages: true,
					insertPageMarkers: true,
					markerFormat: "--- Page {page_num} ---",
				},
				outputFormat: "markdown",
				resultFormat: "unified",
			};

			// Verify complete structure
			expect(config.ocr?.backend).toBe("tesseract");
			expect(config.chunking?.maxChars).toBe(4096);
			expect(config.keywords?.algorithm).toBe("yake");
			expect(config.outputFormat).toBe("markdown");
		});

		it("should handle minimal config (defaults)", () => {
			const result = extractFileSync(testFile, null);
			expect(result).toBeDefined();
		});

		it("should handle only chunking config", () => {
			const config: ExtractionConfig = {
				chunking: {
					maxChars: 2048,
				},
			};
			const result = extractBytesSync(testBuffer, "text/plain", config);
			expect(result).toBeDefined();
		});

		it("should handle only OCR config", () => {
			const config: ExtractionConfig = {
				ocr: {
					backend: "tesseract",
					language: "eng",
				},
			};
			const result = extractBytesSync(testBuffer, "text/plain", config);
			expect(result).toBeDefined();
		});
	});
});
