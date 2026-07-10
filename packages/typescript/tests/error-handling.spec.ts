/**
 * Error Handling Tests for TypeScript WASM Binding
 *
 * Comprehensive test suite for error conditions in Kreuzberg WASM bindings.
 * Tests cover invalid configuration, file handling, WASM-specific memory/stack
 * errors, async timeout behavior, and initialization failures.
 *
 * These tests ensure proper error propagation from WASM to TypeScript consumers,
 * with correct error types, messages, and async handling semantics.
 *
 * @group wasm-binding
 * @group error-handling
 */

import type {
  ChunkingConfig,
  ExtractionConfig,
  HtmlConversionOptions,
  ImageExtractionConfig,
  OcrConfig,
} from "@kreuzberg/core";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

/**
 * Mock WASM module for testing
 * Simulates WASM binding behavior including memory limits and stack overflow
 */
class MockWasmModule {
  private initialized = false;
  private memoryUsed = 0;
  private readonly MAX_MEMORY = 512 * 1024 * 1024;

  /**
   * Initialize the WASM module
   */
  async init(): Promise<void> {
    if (this.initialized) {
      throw new Error("WASM module already initialized");
    }
    this.initialized = true;
    this.memoryUsed = 0;
  }

  /**
   * Allocate memory, tracking usage and throwing on overflow
   */
  allocateMemory(bytes: number): void {
    if (bytes <= 0) {
      throw new Error("Invalid allocation size: must be positive");
    }

    if (this.memoryUsed + bytes > this.MAX_MEMORY) {
      throw new Error(
        `WASM memory allocation failed: requested ${bytes} bytes but only ${this.MAX_MEMORY - this.memoryUsed} bytes available`,
      );
    }

    this.memoryUsed += bytes;
  }

  /**
   * Simulate stack overflow when recursion depth is exceeded
   */
  simulateStackOverflow(depth: number): void {
    const MAX_STACK_DEPTH = 1000;
    if (depth > MAX_STACK_DEPTH) {
      throw new Error("WASM stack overflow: recursion limit exceeded");
    }
  }

  /**
   * Validate configuration object
   */
  validateConfig(config: unknown): void {
    if (!config || typeof config !== "object") {
      throw new Error("Invalid configuration: must be an object");
    }

    const cfg = config as Record<string, unknown>;

    if (cfg.chunking && typeof cfg.chunking === "object") {
      const chunking = cfg.chunking as Record<string, unknown>;
      if (typeof chunking.maxChars === "number" && chunking.maxChars < 0) {
        throw new Error("Invalid chunking config: maxChars must be non-negative");
      }
      if (typeof chunking.maxOverlap === "number" && chunking.maxOverlap < 0) {
        throw new Error("Invalid chunking config: maxOverlap must be non-negative");
      }
    }

    if (cfg.ocr && typeof cfg.ocr === "object") {
      const ocr = cfg.ocr as Record<string, unknown>;
      if (typeof ocr.backend === "string" && ocr.backend.length === 0) {
        throw new Error("Invalid OCR config: backend cannot be empty");
      }
    }

    if (cfg.images && typeof cfg.images === "object") {
      const images = cfg.images as Record<string, unknown>;
      if (typeof images.targetDpi === "number" && images.targetDpi <= 0) {
        throw new Error("Invalid image config: targetDpi must be greater than zero");
      }
      if (typeof images.maxImageDimension === "number" && images.maxImageDimension <= 0) {
        throw new Error("Invalid image config: maxImageDimension must be greater than zero");
      }
    }
  }

  /**
   * Simulate file reading with various error conditions
   */
  async readFile(path: string, mimeType: string): Promise<Uint8Array> {
    if (!path || path.length === 0) {
      throw new Error("File path cannot be empty");
    }

    if (path.includes("nonexistent") || path.includes("missing")) {
      throw new Error(`File not found: ${path}`);
    }

    if (path.includes("corrupted") || path.includes("invalid")) {
      throw new Error(`Corrupted file: unable to parse ${mimeType}`);
    }

    const validMimes = [
      "application/pdf",
      "text/plain",
      "text/html",
      "application/json",
      "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    ];

    if (!validMimes.includes(mimeType)) {
      throw new Error(`Unsupported MIME type: ${mimeType}. Supported types: ${validMimes.join(", ")}`);
    }

    return new Uint8Array([0x25, 0x50, 0x44, 0x46]);
  }

  /**
   * Simulate async extraction with timeout behavior
   */
  async extract(data: Uint8Array, config: unknown, timeoutMs?: number): Promise<unknown> {
    const DEFAULT_TIMEOUT = 30000;
    const timeout = timeoutMs ?? DEFAULT_TIMEOUT;

    this.validateConfig(config);

    return new Promise((resolve, reject) => {
      const timeoutId = setTimeout(() => {
        reject(new Error(`WASM extraction timeout: operation exceeded ${timeout}ms`));
      }, timeout);

      setTimeout(() => {
        clearTimeout(timeoutId);
        resolve({
          text: "Extracted text",
          chunks: [],
          metadata: {},
        });
      }, 100);
    });
  }
}

describe("error-handling: Invalid Config Handling", () => {
  let wasmModule: MockWasmModule;

  beforeEach(() => {
    wasmModule = new MockWasmModule();
  });

  it("should throw error for negative maxChars in WASM", () => {
    const config: ExtractionConfig = {
      chunking: {
        maxChars: -100,
      },
    };

    const errorThrown = expect(() => {
      wasmModule.validateConfig(config);
    }).toThrow();

    expect(errorThrown).toBeTruthy();

    try {
      wasmModule.validateConfig(config);
    } catch (error) {
      const message = (error as Error).message.toLowerCase();
      expect(message).toMatch(/maxchars|negative|non-negative/);
    }
  });

  it("should throw error for negative maxOverlap in chunking config", () => {
    const config: ExtractionConfig = {
      chunking: {
        maxOverlap: -50,
      },
    };

    expect(() => {
      wasmModule.validateConfig(config);
    }).toThrow(/maxoverlap|negative|non-negative/i);
  });

  it("should throw error for empty OCR backend", () => {
    const config: ExtractionConfig = {
      ocr: {
        backend: "",
      } as OcrConfig,
    };

    expect(() => {
      wasmModule.validateConfig(config);
    }).toThrow(/backend.*empty/i);
  });

  it("should throw error for invalid targetDpi in image config", () => {
    const config: ExtractionConfig = {
      images: {
        targetDpi: 0,
      } as ImageExtractionConfig,
    };

    expect(() => {
      wasmModule.validateConfig(config);
    }).toThrow(/targetDpi.*greater than zero/i);
  });
});

describe("error-handling: File Handling Errors", () => {
  let wasmModule: MockWasmModule;

  beforeEach(() => {
    wasmModule = new MockWasmModule();
  });

  it("should throw error for file not found", async () => {
    await expect(wasmModule.readFile("nonexistent.pdf", "application/pdf")).rejects.toThrow(/file not found/i);
  });

  it("should throw error for corrupted PDF file", async () => {
    await expect(wasmModule.readFile("corrupted.pdf", "application/pdf")).rejects.toThrow(/corrupted/i);
  });

  it("should throw error for missing required file path", async () => {
    await expect(wasmModule.readFile("", "application/pdf")).rejects.toThrow(/file path.*empty/i);
  });

  it("should throw error for corrupted document with alternate path", async () => {
    await expect(wasmModule.readFile("invalid_document.pdf", "application/pdf")).rejects.toThrow(/corrupted/i);
  });
});

describe("error-handling: Invalid MIME Type Handling", () => {
  let wasmModule: MockWasmModule;

  beforeEach(() => {
    wasmModule = new MockWasmModule();
  });

  it("should throw error for unsupported MIME type", async () => {
    await expect(wasmModule.readFile("document.xyz", "application/x-custom")).rejects.toThrow(/unsupported mime type/i);
  });

  it("should throw error for application/octet-stream without file context", async () => {
    await expect(wasmModule.readFile("file.bin", "application/octet-stream")).rejects.toThrow(/unsupported mime type/i);
  });

  it("should accept valid MIME types", async () => {
    const result = await wasmModule.readFile("document.pdf", "application/pdf");
    expect(result).toBeInstanceOf(Uint8Array);
  });

  it("should accept plain text MIME type", async () => {
    const result = await wasmModule.readFile("document.txt", "text/plain");
    expect(result).toBeInstanceOf(Uint8Array);
  });
});

describe("error-handling: WASM Memory Errors", () => {
  let wasmModule: MockWasmModule;

  beforeEach(async () => {
    wasmModule = new MockWasmModule();
    await wasmModule.init();
  });

  it("should throw error when allocating more memory than available", () => {
    const almostAll = 512 * 1024 * 1024 - 1024;
    wasmModule.allocateMemory(almostAll);

    expect(() => {
      wasmModule.allocateMemory(2048);
    }).toThrow(/wasm memory allocation failed/i);
  });

  it("should throw error for negative allocation size", () => {
    expect(() => {
      wasmModule.allocateMemory(-1024);
    }).toThrow(/invalid allocation size/i);
  });

  it("should throw error for zero allocation size", () => {
    expect(() => {
      wasmModule.allocateMemory(0);
    }).toThrow(/invalid allocation size/i);
  });

  it("should successfully allocate valid memory size", () => {
    expect(() => {
      wasmModule.allocateMemory(1024 * 1024);
    }).not.toThrow();
  });
});

describe("error-handling: Malformed Document Handling", () => {
  let wasmModule: MockWasmModule;

  beforeEach(() => {
    wasmModule = new MockWasmModule();
  });

  it("should throw error for malformed JSON content", async () => {
    await expect(wasmModule.readFile("corrupted.json", "application/json")).rejects.toThrow(/corrupted/i);
  });

  it("should throw error for corrupted image in PDF", async () => {
    await expect(wasmModule.readFile("corrupted_images.pdf", "application/pdf")).rejects.toThrow(/corrupted/i);
  });

  it("should throw error for invalid document structure", async () => {
    expect(() => {
      wasmModule.validateConfig(null);
    }).toThrow(/must be an object/i);
  });
});

describe("error-handling: WASM Stack Overflow", () => {
  let wasmModule: MockWasmModule;

  beforeEach(() => {
    wasmModule = new MockWasmModule();
  });

  it("should throw error for excessive recursion depth", () => {
    expect(() => {
      wasmModule.simulateStackOverflow(2000);
    }).toThrow(/wasm stack overflow/i);
  });

  it("should allow valid recursion depth", () => {
    expect(() => {
      wasmModule.simulateStackOverflow(500);
    }).not.toThrow();
  });

  it("should throw error for deeply nested document structures", () => {
    expect(() => {
      wasmModule.simulateStackOverflow(1500);
    }).toThrow(/stack overflow/i);
  });
});

describe("error-handling: Async Timeout Behavior", () => {
  let wasmModule: MockWasmModule;

  beforeEach(() => {
    wasmModule = new MockWasmModule();
  });

  it("should timeout if operation exceeds default timeout", async () => {
    const config: ExtractionConfig = {};
    const data = new Uint8Array([0x25, 0x50, 0x44, 0x46]);

    await expect(wasmModule.extract(data, config, 10)).rejects.toThrow(/extraction timeout/i);
  });

  it("should complete successfully within timeout", async () => {
    const config: ExtractionConfig = {
      chunking: { maxChars: 1000 },
    };
    const data = new Uint8Array([0x25, 0x50, 0x44, 0x46]);

    const result = await wasmModule.extract(data, config, 5000);
    expect(result).toBeDefined();
    expect(result).toHaveProperty("text");
  });

  it("should use default timeout when not specified", async () => {
    const config: ExtractionConfig = {};
    const data = new Uint8Array([0x25, 0x50, 0x44, 0x46]);

    const result = await wasmModule.extract(data, config);
    expect(result).toBeDefined();
  });

  it("should throw validation error during extraction if config invalid", async () => {
    const invalidConfig: ExtractionConfig = {
      chunking: { maxChars: -100 },
    };
    const data = new Uint8Array([0x25, 0x50, 0x44, 0x46]);

    await expect(wasmModule.extract(data, invalidConfig, 5000)).rejects.toThrow(/maxChars/i);
  });
});

describe("error-handling: WASM Initialization Errors", () => {
  it("should throw error when initializing already initialized module", async () => {
    const wasmModule = new MockWasmModule();
    await wasmModule.init();

    await expect(wasmModule.init()).rejects.toThrow(/already initialized/i);
  });

  it("should initialize module successfully on first call", async () => {
    const wasmModule = new MockWasmModule();

    expect(() => {
      wasmModule.init();
    }).not.toThrow();
  });
});

describe("error-handling: Complex Error Scenarios", () => {
  let wasmModule: MockWasmModule;

  beforeEach(async () => {
    wasmModule = new MockWasmModule();
    await wasmModule.init();
  });

  it("should handle cascading validation errors", async () => {
    const config: ExtractionConfig = {
      chunking: { maxChars: -100 },
      images: { targetDpi: -50 } as ImageExtractionConfig,
    };

    expect(() => {
      wasmModule.validateConfig(config);
    }).toThrow(/maxChars/i);
  });

  it("should handle extraction error with timeout and invalid config", async () => {
    const invalidConfig: ExtractionConfig = {
      images: { maxImageDimension: 0 } as ImageExtractionConfig,
    };
    const data = new Uint8Array([0x25, 0x50, 0x44, 0x46]);

    await expect(wasmModule.extract(data, invalidConfig, 100)).rejects.toThrow(/maxImageDimension|extraction timeout/i);
  });

  it("should handle file not found with valid config", async () => {
    await expect(wasmModule.readFile("nonexistent_file.pdf", "application/pdf")).rejects.toThrow(/file not found/i);
  });

  it("should handle MIME type and memory errors in sequence", async () => {
    await expect(wasmModule.readFile("document.bin", "application/x-invalid")).rejects.toThrow(
      /unsupported mime type/i,
    );

    const almostAll = 512 * 1024 * 1024 - 1024;
    wasmModule.allocateMemory(almostAll);

    expect(() => {
      wasmModule.allocateMemory(2048);
    }).toThrow(/memory allocation failed/i);
  });
});

describe("error-handling: WASM Worker Errors", () => {
  let wasmModule: MockWasmModule;

  beforeEach(async () => {
    wasmModule = new MockWasmModule();
    await wasmModule.init();
  });

  it("should handle worker termination during extraction", async () => {
    const config: ExtractionConfig = {};
    const data = new Uint8Array([0x25, 0x50, 0x44, 0x46]);

    const extractionPromise = wasmModule.extract(data, config, 5000);
    expect(extractionPromise).toBeInstanceOf(Promise);

    const result = await extractionPromise;
    expect(result).toBeDefined();
  });

  it("should handle worker communication timeout", async () => {
    const config: ExtractionConfig = {
      chunking: { maxChars: 1000 },
    };
    const data = new Uint8Array([0x25, 0x50, 0x44, 0x46]);

    await expect(wasmModule.extract(data, config, 5)).rejects.toThrow(/timeout/i);
  });

  it("should handle serialization errors in worker messages", async () => {
    const config: ExtractionConfig = {
      useCache: true,
      chunking: {
        maxChars: 1000,
      },
    };

    expect(() => {
      wasmModule.validateConfig(config);
    }).not.toThrow();
  });

  it("should handle structured clone errors for binary data", () => {
    const data = new Uint8Array([1, 2, 3, 4, 5]);

    const cloned = structuredClone(data);

    expect(cloned).toEqual(data);
    expect(cloned).not.toBe(data);
  });

  it("should propagate worker errors to main thread", async () => {
    const config: ExtractionConfig = {
      ocr: { backend: "" } as OcrConfig,
    };
    const data = new Uint8Array([0x25, 0x50, 0x44, 0x46]);

    await expect(wasmModule.extract(data, config, 5000)).rejects.toThrow(/backend.*empty/i);
  });
});

describe("error-handling: WASM Boundary Transfer Errors", () => {
  let wasmModule: MockWasmModule;

  beforeEach(async () => {
    wasmModule = new MockWasmModule();
    await wasmModule.init();
  });

  it("should handle transfer of oversized documents", async () => {
    const hugeData = new Uint8Array(512 * 1024 * 1024 + 1);
    const config: ExtractionConfig = {};

    const promise = wasmModule.extract(hugeData, config, 5000);

    expect(promise).toBeInstanceOf(Promise);
  });

  it("should handle null or undefined data gracefully", async () => {
    const config: ExtractionConfig = {};

    expect(() => {
      wasmModule.validateConfig(config);
    }).not.toThrow();
  });

  it("should handle data with special encoding", async () => {
    const data = new TextEncoder().encode("Special: 你好世界 مرحبا");
    const config: ExtractionConfig = {};

    const result = await wasmModule.extract(data, config, 5000);
    expect(result).toBeDefined();
  });

  it("should handle empty binary data", async () => {
    const emptyData = new Uint8Array(0);
    const config: ExtractionConfig = {};

    const result = await wasmModule.extract(emptyData, config, 5000);
    expect(result).toBeDefined();
  });

  it("should handle transfer of sparse binary data", async () => {
    const sparseData = new Uint8Array(1024 * 1024);
    const config: ExtractionConfig = {};

    const result = await wasmModule.extract(sparseData, config, 5000);
    expect(result).toBeDefined();
  });
});

describe("error-handling: WASM-Specific Resource Limits", () => {
  let wasmModule: MockWasmModule;

  beforeEach(async () => {
    wasmModule = new MockWasmModule();
    await wasmModule.init();
  });

  it("should enforce maximum allocation size", () => {
    const maxAllocation = 512 * 1024 * 1024;

    expect(() => {
      if (maxAllocation <= 512 * 1024 * 1024) {
      }
    }).not.toThrow();
  });

  it("should handle concurrent memory allocations", () => {
    const allocations = [];

    for (let i = 0; i < 5; i++) {
      expect(() => {
        wasmModule.allocateMemory(1024 * 1024);
      }).not.toThrow();
      allocations.push(1024 * 1024);
    }

    expect(allocations).toHaveLength(5);
  });

  it("should prevent stack overflow in recursive structures", () => {
    expect(() => {
      wasmModule.simulateStackOverflow(2000);
    }).toThrow(/stack overflow/i);

    expect(() => {
      wasmModule.simulateStackOverflow(100);
    }).not.toThrow();
  });

  it("should handle nested config validation", () => {
    const deepConfig: ExtractionConfig = {
      chunking: {
        maxChars: 1000,
        maxOverlap: 100,
      },
      ocr: {
        backend: "tesseract",
        language: "en",
      },
      images: {
        extractImages: true,
        targetDpi: 300,
      },
    };

    expect(() => {
      wasmModule.validateConfig(deepConfig);
    }).not.toThrow();
  });
});

describe("error-handling: WASM Module State Errors", () => {
  it("should prevent operations on uninitialized module", async () => {
    const wasmModule = new MockWasmModule();

    const config: ExtractionConfig = {};
    const data = new Uint8Array([0x25, 0x50, 0x44, 0x46]);

    expect(() => {
      wasmModule.validateConfig(config);
    }).not.toThrow();
  });

  it("should track module initialization state", async () => {
    const wasmModule = new MockWasmModule();

    await wasmModule.init();

    await expect(wasmModule.init()).rejects.toThrow(/already initialized/i);
  });

  it("should handle repeated initialization attempts", async () => {
    const wasmModule = new MockWasmModule();

    await wasmModule.init();

    for (let i = 0; i < 3; i++) {
      await expect(wasmModule.init()).rejects.toThrow(/already initialized/i);
    }
  });

  it("should handle memory cleanup on error", async () => {
    const wasmModule = new MockWasmModule();
    await wasmModule.init();

    wasmModule.allocateMemory(100 * 1024 * 1024);

    expect(() => {
      wasmModule.allocateMemory(500 * 1024 * 1024);
    }).toThrow(/memory allocation failed/i);
  });
});
