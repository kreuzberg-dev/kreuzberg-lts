import { readFileSync, realpathSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { ExtractionConfig, extractBytesSync, extractFileSync } from "../../dist/index.js";
import type { ExtractionConfig as ExtractionConfigType } from "../../src/types.js";
import { getTestDocumentPath } from "../helpers/index.js";

describe("Configuration Options", () => {
  const pdfPath = getTestDocumentPath("pdf/simple.pdf");
  const pdfBytes = new Uint8Array(readFileSync(realpathSync(pdfPath)));

  describe("Basic configuration", () => {
    it("should handle useCache: true", () => {
      const config: ExtractionConfigType = { useCache: true };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });

    it("should handle useCache: false", () => {
      const config: ExtractionConfigType = { useCache: false };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });

    it("should handle enableQualityProcessing: true", () => {
      const config: ExtractionConfigType = { enableQualityProcessing: true };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });

    it("should handle enableQualityProcessing: false", () => {
      const config: ExtractionConfigType = { enableQualityProcessing: false };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });
  });

  describe("OCR configuration", () => {
    it("should handle OCR with tesseract backend", () => {
      const config: ExtractionConfigType = {
        ocr: {
          backend: "tesseract",
          language: "eng",
        },
      };
      try {
        const result = extractFileSync(pdfPath, null, config);
        expect(result.content).toBeTruthy();
      } catch (e) {
        const errorMessage = e instanceof Error ? e.message : String(e);
        if (errorMessage.includes("Tesseract") && errorMessage.includes("Failed to initialize")) {
          return;
        }
        throw e;
      }
    });

    it("should handle OCR with tesseract config options", () => {
      const config: ExtractionConfigType = {
        ocr: {
          backend: "tesseract",
          language: "eng",
          tesseractConfig: {
            psm: 6,
            enableTableDetection: true,
            tesseditCharWhitelist: "0123456789",
          },
        },
      };
      try {
        const result = extractFileSync(pdfPath, null, config);
        expect(result.content).toBeTruthy();
      } catch (e) {
        const errorMessage = e instanceof Error ? e.message : String(e);
        if (errorMessage.includes("Tesseract") && errorMessage.includes("Failed to initialize")) {
          return;
        }
        throw e;
      }
    });

    it("should handle forceOcr: true", () => {
      const config: ExtractionConfigType = {
        forceOcr: true,
        ocr: {
          backend: "tesseract",
          language: "eng",
        },
      };
      try {
        const result = extractFileSync(pdfPath, null, config);
        expect(result.content).toBeTruthy();
      } catch (e) {
        const errorMessage = e instanceof Error ? e.message : String(e);
        if (errorMessage.includes("Tesseract") && errorMessage.includes("Failed to initialize")) {
          return;
        }
        throw e;
      }
    });

    it("should handle forceOcr: false", () => {
      const config: ExtractionConfigType = {
        forceOcr: false,
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });
  });

  describe("Chunking configuration", () => {
    it("should handle chunking with maxChars", () => {
      const config: ExtractionConfigType = {
        chunking: {
          maxChars: 1000,
          maxOverlap: 100,
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });

    it("should handle chunking with different settings", () => {
      const config: ExtractionConfigType = {
        chunking: {
          maxChars: 500,
          maxOverlap: 50,
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });
  });

  describe("HTML conversion configuration", () => {
    it("should handle htmlOptions with preprocessing", () => {
      const config: ExtractionConfigType = {
        htmlOptions: {
          headingStyle: "atx",
          wrap: true,
          wrapWidth: 120,
          preprocessing: {
            enabled: true,
            preset: "standard",
          },
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });
  });

  describe("Keyword configuration", () => {
    it("should handle keyword extraction settings", () => {
      const config: ExtractionConfigType = {
        keywords: {
          algorithm: "yake",
          maxKeywords: 5,
          minScore: 0.2,
          ngramRange: [1, 3],
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });
  });

  describe("PDF options", () => {
    it("should handle PDF extractImages: true", () => {
      const config: ExtractionConfigType = {
        pdfOptions: {
          extractImages: true,
          extractMetadata: true,
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });

    it("should handle PDF extractImages: false", () => {
      const config: ExtractionConfigType = {
        pdfOptions: {
          extractImages: false,
          extractMetadata: true,
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });

    it("should handle PDF password configuration", () => {
      const config: ExtractionConfigType = {
        pdfOptions: {
          passwords: ["test123", "password"],
          extractMetadata: true,
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });
  });

  describe("Image configuration", () => {
    it("should handle image extraction config", () => {
      const config: ExtractionConfigType = {
        images: {
          extractImages: true,
          targetDpi: 300,
          maxImageDimension: 4096,
          autoAdjustDpi: true,
          minDpi: 72,
          maxDpi: 600,
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });

    it("should handle custom DPI settings", () => {
      const config: ExtractionConfigType = {
        images: {
          targetDpi: 150,
          minDpi: 100,
          maxDpi: 300,
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });
  });

  describe("Token reduction", () => {
    it("should handle token reduction: off", () => {
      const config: ExtractionConfigType = {
        tokenReduction: {
          mode: "off",
          preserveImportantWords: true,
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });

    it("should handle token reduction: aggressive", () => {
      const config: ExtractionConfigType = {
        tokenReduction: {
          mode: "aggressive",
          preserveImportantWords: false,
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });
  });

  describe("Language detection", () => {
    it("should handle language detection enabled", () => {
      const config: ExtractionConfigType = {
        languageDetection: {
          enabled: true,
          minConfidence: 0.8,
          detectMultiple: false,
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });

    it("should handle language detection with multiple languages", () => {
      const config: ExtractionConfigType = {
        languageDetection: {
          enabled: true,
          minConfidence: 0.7,
          detectMultiple: true,
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });
  });

  describe("Postprocessor configuration", () => {
    it("should handle postprocessor enabled: true", () => {
      const config: ExtractionConfigType = {
        postprocessor: {
          enabled: true,
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });

    it("should handle postprocessor enabled: false", () => {
      const config: ExtractionConfigType = {
        postprocessor: {
          enabled: false,
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });

    it("should handle enabled processors list", () => {
      const config: ExtractionConfigType = {
        postprocessor: {
          enabled: true,
          enabledProcessors: ["processor1", "processor2"],
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });

    it("should handle disabled processors list", () => {
      const config: ExtractionConfigType = {
        postprocessor: {
          enabled: true,
          disabledProcessors: ["processor3", "processor4"],
        },
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });
  });

  describe("Max concurrent extractions", () => {
    it("should handle maxConcurrentExtractions setting", () => {
      const config: ExtractionConfigType = {
        maxConcurrentExtractions: 4,
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });

    it("should handle maxConcurrentExtractions: 1", () => {
      const config: ExtractionConfigType = {
        maxConcurrentExtractions: 1,
      };
      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });
  });

  describe("Combined configurations", () => {
    it("should handle complex configuration with multiple options", () => {
      const config: ExtractionConfigType = {
        useCache: false,
        enableQualityProcessing: true,
        ocr: {
          backend: "tesseract",
          language: "eng",
        },
        chunking: {
          maxChars: 1000,
          maxOverlap: 200,
        },
        images: {
          targetDpi: 300,
        },
        tokenReduction: {
          mode: "off",
        },
        languageDetection: {
          enabled: true,
        },
        maxConcurrentExtractions: 2,
      };
      try {
        const result = extractFileSync(pdfPath, null, config);
        expect(result.content).toBeTruthy();
      } catch (e) {
        const errorMessage = e instanceof Error ? e.message : String(e);
        if (errorMessage.includes("Tesseract") && errorMessage.includes("Failed to initialize")) {
          return;
        }
        throw e;
      }
    });

    it("should handle configuration with bytes extraction", () => {
      const config: ExtractionConfigType = {
        useCache: false,
        enableQualityProcessing: true,
        pdfOptions: {
          extractImages: true,
          extractMetadata: true,
        },
      };
      const result = extractBytesSync(pdfBytes, "application/pdf", config);
      expect(result.content).toBeTruthy();
    });
  });

  describe("Config file loading", () => {
    const fixturesDir = join(process.cwd(), "tests", "fixtures");

    function expectOcrSection(
      config: ExtractionConfigType,
      expected: { backend: string; language: string; psm: number; enableTableDetection: boolean; whitelist: string },
    ) {
      expect(config.ocr).toBeDefined();
      expect(config.ocr?.backend).toBe(expected.backend);
      expect(config.ocr?.language).toBe(expected.language);
      expect(config.ocr?.tesseractConfig?.psm).toBe(expected.psm);
      expect(config.ocr?.tesseractConfig?.enableTableDetection).toBe(expected.enableTableDetection);
      expect(config.ocr?.tesseractConfig?.tesseditCharWhitelist).toBe(expected.whitelist);
    }

    function expectChunkingSection(config: ExtractionConfigType, expected: { maxChars: number; maxOverlap: number }) {
      expect(config.chunking).toBeDefined();
      expect(config.chunking?.maxChars).toBe(expected.maxChars);
      expect(config.chunking?.maxOverlap).toBe(expected.maxOverlap);
    }

    function expectImagesSection(
      config: ExtractionConfigType,
      expected: {
        extractImages: boolean;
        targetDpi: number;
        maxImageDimension: number;
        autoAdjustDpi: boolean;
        minDpi: number;
        maxDpi: number;
      },
    ) {
      expect(config.images).toBeDefined();
      expect(config.images?.extractImages).toBe(expected.extractImages);
      expect(config.images?.targetDpi).toBe(expected.targetDpi);
      expect(config.images?.maxImageDimension).toBe(expected.maxImageDimension);
      expect(config.images?.autoAdjustDpi).toBe(expected.autoAdjustDpi);
      expect(config.images?.minDpi).toBe(expected.minDpi);
      expect(config.images?.maxDpi).toBe(expected.maxDpi);
    }

    function expectPdfOptionsSection(
      config: ExtractionConfigType,
      expected: { extractImages: boolean; extractMetadata: boolean; passwords: string[] },
    ) {
      expect(config.pdfOptions).toBeDefined();
      expect(config.pdfOptions?.extractImages).toBe(expected.extractImages);
      expect(config.pdfOptions?.extractMetadata).toBe(expected.extractMetadata);
      expect(config.pdfOptions?.passwords).toEqual(expected.passwords);
    }

    function expectTokenReductionSection(
      config: ExtractionConfigType,
      expected: { mode: string; preserveImportantWords: boolean },
    ) {
      expect(config.tokenReduction).toBeDefined();
      expect(config.tokenReduction?.mode).toBe(expected.mode);
      expect(config.tokenReduction?.preserveImportantWords).toBe(expected.preserveImportantWords);
    }

    function expectLanguageDetectionSection(
      config: ExtractionConfigType,
      expected: { enabled: boolean; minConfidence: number; detectMultiple: boolean },
    ) {
      expect(config.languageDetection).toBeDefined();
      expect(config.languageDetection?.enabled).toBe(expected.enabled);
      expect(config.languageDetection?.minConfidence).toBe(expected.minConfidence);
      expect(config.languageDetection?.detectMultiple).toBe(expected.detectMultiple);
    }

    function expectPostprocessorSection(
      config: ExtractionConfigType,
      expected: { enabled: boolean; enabledProcessors: string[]; disabledProcessors: string[] },
    ) {
      expect(config.postprocessor).toBeDefined();
      expect(config.postprocessor?.enabled).toBe(expected.enabled);
      expect(config.postprocessor?.enabledProcessors).toEqual(expected.enabledProcessors);
      expect(config.postprocessor?.disabledProcessors).toEqual(expected.disabledProcessors);
    }

    it("should load config from TOML file", () => {
      const configPath = join(fixturesDir, "config.toml");
      const config = ExtractionConfig.fromFile(configPath);

      expect(config).toBeDefined();
      expect(config.useCache).toBe(false);
      expect(config.enableQualityProcessing).toBe(true);
      expect(config.forceOcr).toBe(false);
      expect(config.maxConcurrentExtractions).toBe(4);

      expectOcrSection(config, {
        backend: "tesseract",
        language: "eng",
        psm: 6,
        enableTableDetection: true,
        whitelist: "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ",
      });
      expectChunkingSection(config, { maxChars: 1000, maxOverlap: 200 });
      expectImagesSection(config, {
        extractImages: true,
        targetDpi: 300,
        maxImageDimension: 4096,
        autoAdjustDpi: true,
        minDpi: 72,
        maxDpi: 600,
      });
      expectPdfOptionsSection(config, {
        extractImages: true,
        extractMetadata: true,
        passwords: ["password1", "password2"],
      });
      expectTokenReductionSection(config, { mode: "moderate", preserveImportantWords: true });
      expectLanguageDetectionSection(config, { enabled: true, minConfidence: 0.85, detectMultiple: false });
      expectPostprocessorSection(config, {
        enabled: true,
        enabledProcessors: ["processor1", "processor2"],
        disabledProcessors: ["processor3"],
      });
    });

    it("should load config from YAML file", () => {
      const configPath = join(fixturesDir, "config.yaml");
      const config = ExtractionConfig.fromFile(configPath);

      expect(config).toBeDefined();
      expect(config.useCache).toBe(true);
      expect(config.enableQualityProcessing).toBe(false);
      expect(config.forceOcr).toBe(true);
      expect(config.maxConcurrentExtractions).toBe(8);

      expectOcrSection(config, {
        backend: "tesseract",
        language: "deu",
        psm: 3,
        enableTableDetection: false,
        whitelist: "0123456789",
      });
      expectChunkingSection(config, { maxChars: 500, maxOverlap: 100 });
      expectImagesSection(config, {
        extractImages: false,
        targetDpi: 150,
        maxImageDimension: 2048,
        autoAdjustDpi: false,
        minDpi: 100,
        maxDpi: 300,
      });
      expectPdfOptionsSection(config, {
        extractImages: false,
        extractMetadata: false,
        passwords: ["test123", "secret456"],
      });
      expectTokenReductionSection(config, { mode: "aggressive", preserveImportantWords: false });
      expectLanguageDetectionSection(config, { enabled: false, minConfidence: 0.7, detectMultiple: true });
      expectPostprocessorSection(config, {
        enabled: false,
        enabledProcessors: [],
        disabledProcessors: ["processor1", "processor2"],
      });
    });

    it("should use loaded config for extraction", () => {
      const configPath = join(fixturesDir, "config.toml");
      const config = ExtractionConfig.fromFile(configPath);

      const result = extractFileSync(pdfPath, null, config);
      expect(result.content).toBeTruthy();
    });

    it("should throw error for non-existent file", () => {
      const configPath = join(fixturesDir, "nonexistent.toml");

      expect(() => {
        ExtractionConfig.fromFile(configPath);
      }).toThrow();
    });

    it("should throw error for invalid TOML file", () => {
      const configPath = join(fixturesDir, "invalid-config.toml");

      expect(() => {
        ExtractionConfig.fromFile(configPath);
      }).toThrow(/Invalid TOML|TOML/i);
    });

    it("should support both relative and absolute paths", () => {
      const absolutePath = join(fixturesDir, "config.toml");
      const config1 = ExtractionConfig.fromFile(absolutePath);
      expect(config1).toBeDefined();

      const relativePath = join("..", "..", "test_documents", "test_fixtures", "config.yaml");
      const config2 = ExtractionConfig.fromFile(relativePath);
      expect(config2).toBeDefined();
    });

    it("should auto-detect file format based on extension", () => {
      const tomlPath = join(fixturesDir, "config.toml");
      const yamlPath = join(fixturesDir, "config.yaml");

      const tomlConfig = ExtractionConfig.fromFile(tomlPath);
      const yamlConfig = ExtractionConfig.fromFile(yamlPath);

      expect(tomlConfig).toBeDefined();
      expect(yamlConfig).toBeDefined();
      expect(tomlConfig.useCache).toBe(false);
      expect(yamlConfig.useCache).toBe(true);
    });
  });
});
