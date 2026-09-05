/**
 * WASM Type Adapter
 *
 * This module provides type adapters for converting between JavaScript/TypeScript
 * types and WASM-compatible types, handling File/Blob conversions, config normalization,
 * and result parsing.
 *
 * @example File Conversion
 * ```typescript
 * import { fileToUint8Array } from '@kreuzberg/wasm/adapters/wasm-adapter';
 *
 * const file = event.target.files[0];
 * const bytes = await fileToUint8Array(file);
 * const result = await extractBytes(bytes, file.type);
 * ```
 *
 * @example Config Normalization
 * ```typescript
 * import { configToJS } from '@kreuzberg/wasm/adapters/wasm-adapter';
 *
 * const config = {
 *   ocr: { backend: 'tesseract', language: 'eng' },
 *   chunking: { maxChars: 1000 }
 * };
 * const normalized = configToJS(config);
 * ```
 */

import type {
  Chunk,
  DocumentStructure,
  Element,
  ExtractedImage,
  ExtractedKeyword,
  ExtractionConfig,
  ExtractionResult,
  Metadata,
  OcrElement,
  PageContent,
  PdfAnnotation,
  ProcessingWarning,
  Table,
} from "../types.js";

/**
 * Maximum file size for processing (512 MB)
 *
 * @internal
 */
const MAX_FILE_SIZE = 512 * 1024 * 1024;

/**
 * Type predicate to validate numeric value or null
 *
 * @internal
 */
function isNumberOrNull(value: unknown): value is number | null {
  return typeof value === "number" || value === null || value === undefined;
}

/**
 * Type predicate to validate string value or null
 *
 * @internal
 */
function isStringOrNull(value: unknown): value is string | null {
  return typeof value === "string" || value === null || value === undefined;
}

/**
 * Type predicate to validate boolean value
 *
 * @internal
 */
function isBoolean(value: unknown): value is boolean {
  return typeof value === "boolean" || value === undefined;
}

/**
 * Convert a File or Blob to Uint8Array
 *
 * Handles both browser File API and server-side Blob-like objects,
 * providing a unified interface for reading binary data.
 *
 * @param file - The File or Blob to convert
 * @returns Promise resolving to the byte array
 * @throws {Error} If the file cannot be read or exceeds size limit
 *
 * @example
 * ```typescript
 * const file = document.getElementById('input').files[0];
 * const bytes = await fileToUint8Array(file);
 * const result = await extractBytes(bytes, 'application/pdf');
 * ```
 */
export async function fileToUint8Array(file: File | Blob): Promise<Uint8Array> {
  try {
    if (file.size > MAX_FILE_SIZE) {
      throw new Error(
        `File size (${file.size} bytes) exceeds maximum (${MAX_FILE_SIZE} bytes). Maximum file size is 512 MB.`,
      );
    }

    const arrayBuffer = await file.arrayBuffer();
    return new Uint8Array(arrayBuffer);
  } catch (error) {
    throw new Error(`Failed to read file: ${error instanceof Error ? error.message : String(error)}`, {
      cause: error,
    });
  }
}

/**
 * Normalize ExtractionConfig for WASM processing
 *
 * Converts TypeScript configuration objects to a WASM-compatible format,
 * handling null values, undefined properties, and nested structures.
 *
 * @param config - The extraction configuration or null
 * @returns Normalized configuration object suitable for WASM
 *
 * @example
 * ```typescript
 * const config: ExtractionConfig = {
 *   ocr: { backend: 'tesseract' },
 *   chunking: { maxChars: 1000 }
 * };
 * const wasmConfig = configToJS(config);
 * ```
 */
export function configToJS(config: ExtractionConfig | null): Record<string, unknown> {
  if (!config) {
    return {};
  }

  const toSnakeCase = (str: string): string => str.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);

  const normalizeValue = (value: unknown): unknown => {
    if (value === null || value === undefined) {
      return null;
    }
    if (typeof value === "object") {
      if (Array.isArray(value)) {
        return value.map((item) => normalizeValue(item));
      }
      const obj = value as Record<string, unknown>;
      const normalized: Record<string, unknown> = {};
      for (const [key, val] of Object.entries(obj)) {
        const normalizedVal = normalizeValue(val);
        if (normalizedVal !== null && normalizedVal !== undefined) {
          normalized[toSnakeCase(key)] = normalizedVal;
        }
      }
      return Object.keys(normalized).length > 0 ? normalized : null;
    }
    return value;
  };

  const normalized: Record<string, unknown> = {};
  for (const [key, value] of Object.entries(config)) {
    const normalizedValue = normalizeValue(value);
    if (normalizedValue !== null && normalizedValue !== undefined) {
      normalized[toSnakeCase(key)] = normalizedValue;
    }
  }

  return normalized;
}

/**
 * Parse a single raw table entry into a {@link Table}, or null if malformed.
 *
 * @internal
 */
function parseTableEntry(table: unknown): Table | null {
  if (!table || typeof table !== "object") {
    return null;
  }
  const t = table as Record<string, unknown>;
  const pageNumber =
    typeof t.pageNumber === "number" ? t.pageNumber : typeof t.page_number === "number" ? t.page_number : 0;
  const cellsValid =
    Array.isArray(t.cells) &&
    t.cells.every((row) => Array.isArray(row) && row.every((cell) => typeof cell === "string"));
  if (!cellsValid || typeof t.markdown !== "string") {
    return null;
  }
  return {
    cells: t.cells as string[][],
    markdown: t.markdown,
    pageNumber,
  };
}

/**
 * Parse the raw `tables` field of a WASM result into {@link Table} entries.
 *
 * @internal
 */
function parseTables(rawTables: unknown): Table[] {
  if (!Array.isArray(rawTables)) {
    return [];
  }
  const tables: Table[] = [];
  for (const table of rawTables) {
    const parsed = parseTableEntry(table);
    if (parsed) {
      tables.push(parsed);
    }
  }
  return tables;
}

/**
 * Coerce a chunk metadata value to a required number, throwing on invalid input.
 *
 * @internal
 */
function coerceToNumber(value: unknown, fieldName: string): number {
  if (typeof value === "number") {
    return value;
  }
  if (typeof value === "bigint") {
    return Number(value);
  }
  if (typeof value === "string") {
    const parsed = parseInt(value, 10);
    if (Number.isNaN(parsed)) {
      throw new Error(`Invalid chunk metadata: ${fieldName} must be a valid number, got "${value}"`);
    }
    return parsed;
  }
  throw new Error(`Invalid chunk metadata: ${fieldName} must be a number, got ${typeof value}`);
}

/**
 * Coerce a chunk metadata value to an optional number, returning null when absent.
 *
 * @internal
 */
function coerceOptionalNumber(value: unknown, fieldName: string): number | null {
  if (value === null || value === undefined) {
    return null;
  }
  return coerceToNumber(value, fieldName);
}

/**
 * Parse a chunk's embedding array, validating that every entry is a number.
 *
 * @internal
 */
function parseChunkEmbedding(raw: unknown): number[] | null {
  if (!Array.isArray(raw)) {
    return null;
  }
  if (!raw.every((item) => typeof item === "number")) {
    throw new Error("Invalid chunk: embedding must contain only numbers");
  }
  return raw as number[];
}

/**
 * Parse a single heading entry within a chunk's heading context.
 *
 * @internal
 */
function parseHeadingEntry(raw: unknown): { level: number; text: string } {
  const heading = raw as Record<string, unknown>;
  return {
    level: (heading["level"] as number) ?? 0,
    text: (heading["text"] as string) ?? "",
  };
}

/**
 * Parse a chunk's heading context, or null when absent or malformed.
 *
 * @internal
 */
function parseHeadingContext(metadata: Record<string, unknown>): import("../types.js").HeadingContext | null {
  const rawHc = (metadata["heading_context"] ?? metadata["headingContext"]) as
    | Record<string, unknown>
    | null
    | undefined;
  if (!rawHc || typeof rawHc !== "object") {
    return null;
  }
  const rawHeadings = rawHc["headings"];
  if (!Array.isArray(rawHeadings)) {
    return null;
  }
  return { headings: rawHeadings.map((h) => parseHeadingEntry(h)) };
}

/**
 * Parse a single raw chunk entry into a {@link Chunk}.
 *
 * @internal
 */
function parseChunkEntry(chunk: unknown): Chunk {
  if (!chunk || typeof chunk !== "object") {
    throw new Error("Invalid chunk structure");
  }
  const c = chunk as Record<string, unknown>;
  if (typeof c.content !== "string") {
    throw new Error("Invalid chunk: missing content");
  }
  if (!c.metadata || typeof c.metadata !== "object") {
    throw new Error("Invalid chunk: missing metadata");
  }
  const metadata = c.metadata as Record<string, unknown>;
  const embedding = parseChunkEmbedding(c.embedding);

  const charStart = coerceToNumber(
    metadata.charStart ?? metadata.char_start ?? metadata.byteStart ?? metadata.byte_start,
    "charStart",
  );
  const charEnd = coerceToNumber(
    metadata.charEnd ?? metadata.char_end ?? metadata.byteEnd ?? metadata.byte_end,
    "charEnd",
  );
  const chunkIndex = coerceToNumber(metadata.chunkIndex ?? metadata.chunk_index, "chunkIndex");
  const totalChunks = coerceToNumber(metadata.totalChunks ?? metadata.total_chunks, "totalChunks");
  const tokenCount = coerceOptionalNumber(metadata.tokenCount ?? metadata.token_count, "tokenCount");
  const firstPage = coerceOptionalNumber(metadata.firstPage ?? metadata.first_page, "firstPage");
  const lastPage = coerceOptionalNumber(metadata.lastPage ?? metadata.last_page, "lastPage");
  const headingContext = parseHeadingContext(metadata);

  return {
    content: c.content,
    embedding,
    metadata: {
      byteStart: charStart,
      byteEnd: charEnd,
      charStart,
      charEnd,
      tokenCount,
      chunkIndex,
      totalChunks,
      firstPage,
      lastPage,
      headingContext,
    },
  };
}

/**
 * Coerce a raw image's `data` field to a Uint8Array.
 *
 * @internal
 */
function coerceImageData(raw: unknown): Uint8Array {
  if (raw instanceof Uint8Array) {
    return raw;
  }
  if (Array.isArray(raw)) {
    return new Uint8Array(raw as number[]);
  }
  throw new Error("Invalid image: data must be Uint8Array or array");
}

/**
 * Validate the scalar fields of a raw image entry, throwing on the first violation.
 *
 * @internal
 */
function validateImageFields(
  img: Record<string, unknown>,
  imageIndex: unknown,
  pageNumber: unknown,
  bitsPerComponent: unknown,
  isMask: unknown,
): void {
  if (typeof imageIndex !== "number") {
    throw new Error("Invalid image: imageIndex must be a number");
  }
  if (!isNumberOrNull(pageNumber)) {
    throw new Error("Invalid image: pageNumber must be a number or null");
  }
  if (!isNumberOrNull(img.width)) {
    throw new Error("Invalid image: width must be a number or null");
  }
  if (!isNumberOrNull(img.height)) {
    throw new Error("Invalid image: height must be a number or null");
  }
  if (!isNumberOrNull(bitsPerComponent)) {
    throw new Error("Invalid image: bitsPerComponent must be a number or null");
  }
  if (!isBoolean(isMask)) {
    throw new Error("Invalid image: isMask must be a boolean");
  }
  if (!isStringOrNull(img.colorspace)) {
    throw new Error("Invalid image: colorspace must be a string or null");
  }
  if (!isStringOrNull(img.description)) {
    throw new Error("Invalid image: description must be a string or null");
  }
}

/**
 * Parse a single raw image entry into an {@link ExtractedImage}.
 *
 * @internal
 */
function parseImageEntry(image: unknown): ExtractedImage {
  if (!image || typeof image !== "object") {
    throw new Error("Invalid image structure");
  }
  const img = image as Record<string, unknown>;
  const imageData = coerceImageData(img.data);
  if (typeof img.format !== "string") {
    throw new Error("Invalid image: missing format");
  }

  const imageIndex = img.imageIndex ?? img.image_index;
  const pageNumber = img.pageNumber ?? img.page_number;
  const bitsPerComponent = img.bitsPerComponent ?? img.bits_per_component;
  const isMask = img.isMask ?? img.is_mask;
  const ocrResult = img.ocrResult ?? img.ocr_result;

  validateImageFields(img, imageIndex, pageNumber, bitsPerComponent, isMask);

  return {
    data: imageData,
    format: img.format,
    imageIndex: imageIndex as number,
    pageNumber: (pageNumber as number | null) ?? null,
    width: (img.width as number) ?? null,
    height: (img.height as number) ?? null,
    colorspace: (img.colorspace as string) ?? null,
    bitsPerComponent: (bitsPerComponent as number | null) ?? null,
    isMask: (isMask as boolean) ?? false,
    description: (img.description as string) ?? null,
    ocrResult: ocrResult ? jsToExtractionResult(ocrResult) : null,
  };
}

/**
 * Parse the raw `detectedLanguages`/`detected_languages` field.
 *
 * @internal
 */
function parseDetectedLanguages(result: Record<string, unknown>): string[] | null {
  const raw = Array.isArray(result.detectedLanguages) ? result.detectedLanguages : result.detected_languages;
  if (!Array.isArray(raw)) {
    return null;
  }
  if (!raw.every((lang) => typeof lang === "string")) {
    throw new Error("Invalid result: detectedLanguages must contain only strings");
  }
  return raw as string[];
}

/**
 * The remaining optional fields of an {@link ExtractionResult}, gathered from either
 * their camelCase or snake_case WASM representation.
 *
 * @internal
 */
interface OptionalResultFields {
  extractedKeywords: ExtractedKeyword[] | null;
  qualityScore: number | null;
  processingWarnings: ProcessingWarning[] | null;
  elements: Element[] | null;
  ocrElements: OcrElement[] | null;
  document: DocumentStructure | null;
  pages: PageContent[] | null;
  annotations: PdfAnnotation[] | null;
}

/**
 * Parse the remaining optional fields of a raw extraction result.
 *
 * @internal
 */
function parseOptionalResultFields(result: Record<string, unknown>): OptionalResultFields {
  const qualityScoreRaw = result.qualityScore ?? result.quality_score;
  return {
    extractedKeywords: (result.extractedKeywords ?? result.extracted_keywords ?? null) as ExtractedKeyword[] | null,
    qualityScore: typeof qualityScoreRaw === "number" ? qualityScoreRaw : null,
    processingWarnings: (result.processingWarnings ?? result.processing_warnings ?? null) as
      | ProcessingWarning[]
      | null,
    elements: (result.elements ?? null) as Element[] | null,
    ocrElements: (result.ocrElements ?? result.ocr_elements ?? null) as OcrElement[] | null,
    document: (result.document ?? null) as DocumentStructure | null,
    pages: (result.pages ?? null) as PageContent[] | null,
    annotations: (result.annotations ?? null) as PdfAnnotation[] | null,
  };
}

/**
 * Parse WASM extraction result and convert to TypeScript type
 *
 * Handles conversion of WASM-returned objects to proper ExtractionResult types,
 * including proper array conversions and type assertions for tables, chunks, and images.
 *
 * @param jsValue - The raw WASM result value
 * @returns Properly typed ExtractionResult
 * @throws {Error} If the result structure is invalid
 *
 * @example
 * ```typescript
 * const wasmResult = await wasmExtract(bytes, mimeType, config);
 * const result = jsToExtractionResult(wasmResult);
 * console.log(result.content);
 * ```
 */
export function jsToExtractionResult(jsValue: unknown): ExtractionResult {
  if (!jsValue || typeof jsValue !== "object") {
    throw new Error("Invalid extraction result: value is not an object");
  }

  const result = jsValue as Record<string, unknown>;
  const mimeType =
    typeof result.mimeType === "string"
      ? result.mimeType
      : typeof result.mime_type === "string"
        ? result.mime_type
        : null;

  if (typeof result.content !== "string") {
    throw new Error("Invalid extraction result: missing or invalid content");
  }
  if (typeof mimeType !== "string") {
    throw new Error("Invalid extraction result: missing or invalid mimeType");
  }
  if (!result.metadata || typeof result.metadata !== "object") {
    throw new Error("Invalid extraction result: missing or invalid metadata");
  }

  const tables = parseTables(result.tables);
  const chunks = Array.isArray(result.chunks) ? result.chunks.map((chunk) => parseChunkEntry(chunk)) : null;
  const images = Array.isArray(result.images) ? result.images.map((image) => parseImageEntry(image)) : null;
  const detectedLanguages = parseDetectedLanguages(result);
  const optionalFields = parseOptionalResultFields(result);

  return {
    content: result.content,
    mimeType,
    metadata: (result.metadata ?? {}) as Metadata,
    tables,
    detectedLanguages,
    chunks,
    images,
    pages: optionalFields.pages,
    extractedKeywords: optionalFields.extractedKeywords,
    qualityScore: optionalFields.qualityScore,
    processingWarnings: optionalFields.processingWarnings,
    elements: optionalFields.elements,
    ocrElements: optionalFields.ocrElements,
    document: optionalFields.document,
    annotations: optionalFields.annotations,
  };
}

/**
 * Wrap and format WASM errors with context
 *
 * Converts WASM error messages to JavaScript Error objects with proper context
 * and stack trace information when available.
 *
 * @param error - The error from WASM
 * @param context - Additional context about what operation failed
 * @returns A formatted Error object
 *
 * @internal
 *
 * @example
 * ```typescript
 * try {
 *   await wasmExtract(bytes, mimeType);
 * } catch (error) {
 *   throw wrapWasmError(error, 'extracting document');
 * }
 * ```
 */
export function wrapWasmError(error: unknown, context: string): Error {
  if (error instanceof Error) {
    return new Error(`Error ${context}: ${error.message}`, {
      cause: error,
    });
  }

  const message = String(error);
  return new Error(`Error ${context}: ${message}`);
}

/**
 * Validate that a WASM-returned value conforms to ExtractionResult structure
 *
 * Performs structural validation without full type checking,
 * useful for runtime validation of WASM output.
 *
 * @param value - The value to validate
 * @returns True if value appears to be a valid ExtractionResult
 *
 * @internal
 */
export function isValidExtractionResult(value: unknown): value is ExtractionResult {
  if (!value || typeof value !== "object") {
    return false;
  }

  const obj = value as Record<string, unknown>;
  return (
    typeof obj.content === "string" &&
    (typeof obj.mimeType === "string" || typeof obj.mime_type === "string") &&
    obj.metadata !== null &&
    typeof obj.metadata === "object" &&
    Array.isArray(obj.tables)
  );
}
