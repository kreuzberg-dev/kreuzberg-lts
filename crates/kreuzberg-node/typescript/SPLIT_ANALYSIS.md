# Index.ts Split Analysis

## Overview
- **File Size**: 2,361 lines
- **Current Architecture**: Monolithic index.ts serving as the main TypeScript SDK entry point
- **Purpose**: TypeScript/Node.js wrapper around Rust native bindings for document extraction

## File Structure

### Imports (Lines 48-76)
```
- "node:fs" → readFileSync (native file reading)
- "node:module" → createRequire (ESM/CJS compatibility)
- "./errors.js" → PanicContext type
- "./types.js" → All type definitions and interfaces
```

**Key Insight**: File depends on external types.js for all TypeScript interfaces.

---

## Exported Functions and Categories

### 1. CORE EXTRACTION (Lines 795-1148)
Primary user-facing extraction APIs

**Synchronous Functions:**
- `extractFileSync(filePath, mimeTypeOrConfig?, maybeConfig?)` → ExtractionResult
- `extractBytesSync(data, mimeType, config)` → ExtractionResult
- `batchExtractFilesSync(paths, config)` → ExtractionResult[]
- `batchExtractBytesSync(dataArray, mimeTypes, config)` → ExtractionResult[]

**Asynchronous Functions:**
- `extractFile(filePath, mimeTypeOrConfig?, maybeConfig?)` → Promise<ExtractionResult>
- `extractBytes(data, mimeType, config)` → Promise<ExtractionResult>
- `batchExtractFiles(paths, config)` → Promise<ExtractionResult[]>
- `batchExtractBytes(dataArray, mimeTypes, config)` → Promise<ExtractionResult[]>

**Helper Functions Used:**
- `normalizeExtractionConfig()` - converts config to native format
- `convertResult()` - transforms native result to TypeScript types

---

### 2. POST-PROCESSOR PLUGIN REGISTRATION (Lines 1206-1323)
Register and manage custom post-processors

**Exported Functions:**
- `registerPostProcessor(processor)` → void
- `unregisterPostProcessor(name)` → void
- `clearPostProcessors()` → void
- `listPostProcessors()` → string[]

**Helper Functions:**
- Internal processor wrapping logic (JSON serialization/deserialization)

---

### 3. VALIDATOR PLUGIN REGISTRATION (Lines 1360-1485)
Register and manage custom validators

**Exported Functions:**
- `registerValidator(validator)` → void
- `unregisterValidator(name)` → void (Line 1407)
- `clearValidators()` → void (Line 1425)
- `listValidators()` → string[] (Line 1445)

**Helper Functions:**
- Internal validator wrapping logic (JSON handling)

---

### 4. OCR BACKEND REGISTRATION (Lines 1546-1666)
Register and manage custom OCR backends

**Exported Functions:**
- `registerOcrBackend(backend)` → void
- `listOcrBackends()` → string[]
- `unregisterOcrBackend(name)` → void
- `clearOcrBackends()` → void

**Helper Functions:**
- `isOcrProcessTuple(value)` - type guard
- `isNestedOcrProcessTuple(value)` - type guard
- `describePayload(value)` - OCR payload debugging

**Internal Logic:**
- Complex tuple unpacking logic for OCR payload handling
- Base64 conversion for Buffer serialization
- Environment variable check: `KREUZBERG_DEBUG_GUTEN`

---

### 5. DOCUMENT EXTRACTOR REGISTRY (Lines 1684-1724)
Manage custom document extractors

**Exported Functions:**
- `listDocumentExtractors()` → string[]
- `unregisterDocumentExtractor(name)` → void
- `clearDocumentExtractors()` → void

**Note:** No `registerDocumentExtractor()` exported (intentional design - likely internal only)

---

### 6. CONFIGURATION LOADING (Lines 1754-1815)
ExtractionConfig object with static methods

**Exported Object:**
```typescript
export const ExtractionConfig = {
  fromFile(filePath): ExtractionConfigType
  discover(): ExtractionConfigType | null
}
```

**Supported File Formats:** .toml, .yaml, .json

---

### 7. MIME TYPE UTILITIES (Lines 1843-1942)
MIME type detection and validation

**Exported Functions:**
- `detectMimeType(bytes)` → string (from raw bytes via magic bytes)
- `detectMimeTypeFromPath(filePath, checkExists?)` → string (from file extension)
- `validateMimeType(mimeType)` → string
- `getExtensionsForMime(mimeType)` → string[]

**Rules:**
- Any `image/*` MIME type is valid
- MIME types are normalized by the native binding

---

### 8. EMBEDDING UTILITIES (Lines 1949-2007)
Embedding model preset management

**Exported Functions:**
- `listEmbeddingPresets()` → string[]
- `getEmbeddingPreset(name)` → EmbeddingPreset | null

**Related Type:**
```typescript
export interface EmbeddingPreset {
  // Contains preset configuration
}
```

---

### 9. ERROR HANDLING AND DIAGNOSTICS (Lines 2041-2156)
Error code management and classification

**Exported Functions:**
- `getLastErrorCode()` → number (error codes 0-7)
- `getLastPanicContext()` → PanicContext | null
- `getErrorCodeName(code)` → string
- `getErrorCodeDescription(code)` → string
- `classifyError(errorMessage)` → ErrorClassification

**Error Codes:**
- 0: Success (no error)
- 1: GenericError
- 2: Panic
- 3: InvalidArgument
- 4: IoError
- 5: ParsingError
- 6: OcrError
- 7: MissingDependency

---

### 10. WORKER POOL OPERATIONS (Lines 2185-2359)
Concurrent extraction with worker threads

**Exported Functions:**
- `createWorkerPool(size?)` → WorkerPool
- `getWorkerPoolStats(pool)` → WorkerPoolStats
- `extractFileInWorker(pool, filePath, mimeTypeOrConfig?, maybeConfig?)` → Promise<ExtractionResult>
- `batchExtractFilesInWorker(pool, paths, config)` → Promise<ExtractionResult[]>
- `closeWorkerPool(pool)` → Promise<void>

**Related Types:**
- `WorkerPool` - opaque handle type
- `WorkerPoolStats` - pool statistics

---

### 11. TESTING AND VERSION (Lines 233-244, 2361)
Internal testing utilities and version info

**Exported Functions:**
- `__setBindingForTests(mock)` → void (internal)
- `__resetBindingForTests()` → void (internal)
- `__version__` → "4.0.8" (constant)

**Re-Exports:**
- All types from "./types.js"
- Error classes from "./errors.js"
- `GutenOcrBackend` from "./ocr/guten-ocr.js"

---

## Internal Helper Functions (Not Exported)

### Native Binding Management
- `createNativeBindingError(error)` - error formatting for binding load failures
- `loadNativeBinding()` - loads the native .node module
- `getBinding()` - singleton getter with lazy loading

### Type Conversion Functions
- `parseMetadata(metadataStr)` - JSON parsing for metadata
- `ensureUint8Array(value)` - validation helper
- `convertChunk(rawChunk)` - Chunk type conversion
- `convertImage(rawImage)` - ExtractedImage type conversion
- `convertPageContent(rawPage)` - PageContent type conversion
- `convertResult(rawResult)` - ExtractionResult type conversion

### Config Normalization Functions
These convert TypeScript config types to native binding format:
- `setIfDefined<T>()` - utility for selective field assignment
- `normalizeTesseractConfig(config?)`
- `normalizeOcrConfig(ocr?)`
- `normalizeChunkingConfig(chunking?)`
- `normalizeImageExtractionConfig(images?)`
- `normalizePdfConfig(pdf?)`
- `normalizeTokenReductionConfig(tokenReduction?)`
- `normalizeLanguageDetectionConfig(languageDetection?)`
- `normalizePostProcessorConfig(postprocessor?)`
- `normalizeHtmlPreprocessing(options?)`
- `normalizeHtmlOptions(options?)`
- `normalizeKeywordConfig(config?)`
- `normalizePageConfig(pages?)`
- `normalizeExtractionConfig(config)` - master orchestrator

### Type Assertions
- `assertUint8Array(value, name)` - validates and casts to Uint8Array
- `assertUint8ArrayList(values, name)` - validates array of Uint8Arrays

### OCR-Specific Helpers
- `isOcrProcessTuple(value)` - type guard for OCR payload tuples
- `isNestedOcrProcessTuple(value)` - type guard for nested OCR tuples
- `describePayload(value)` - debugging helper for OCR payloads

---

## Native Binding Interface (Line 82-155)

The `NativeBinding` interface defines all methods available from the compiled Rust addon:

**Extraction Methods:**
- extractFileSync, extractFile
- extractBytesSync, extractBytes
- batchExtractFilesSync, batchExtractFiles
- batchExtractBytesSync, batchExtractBytes

**Plugin Registration:**
- registerPostProcessor, unregisterPostProcessor, clearPostProcessors, listPostProcessors
- registerValidator, unregisterValidator, clearValidators, listValidators
- registerOcrBackend, unregisterOcrBackend, clearOcrBackends, listOcrBackends
- registerDocumentExtractor, unregisterDocumentExtractor, clearDocumentExtractors, listDocumentExtractors

**MIME Type Methods:**
- detectMimeType, detectMimeTypeFromBytes, detectMimeTypeFromPath
- validateMimeType, getExtensionsForMime

**Embedding Methods:**
- listEmbeddingPresets, getEmbeddingPreset

**Error Methods:**
- getErrorCodeName, getErrorCodeDescription, classifyError
- getLastErrorCode, getLastPanicContext

**Config Methods:**
- loadExtractionConfigFromFile, discoverExtractionConfig

**Worker Pool Methods:**
- createWorkerPool, getWorkerPoolStats
- extractFileInWorker, batchExtractFilesInWorker
- closeWorkerPool

---

## Module Organization Plan

### 1. `core/binding.ts` (120-150 lines)
**Contains:**
- `NativeBinding` interface
- `loadNativeBinding()` function
- `getBinding()` function (with caching)
- `createNativeBindingError()` function
- Global `binding` and `bindingInitialized` variables

**Circular Dependency Risk:** None (lowest-level module)

---

### 2. `core/type-converters.ts` (150-200 lines)
**Contains:**
- `parseMetadata(metadataStr)`
- `ensureUint8Array(value)`
- `convertChunk(rawChunk)`
- `convertImage(rawImage)`
- `convertPageContent(rawPage)`
- `convertResult(rawResult)`

**Dependencies:** types.js, binding.ts

**Circular Dependency Risk:** None (utility module)

---

### 3. `core/config-normalizer.ts` (200-250 lines)
**Contains:**
- `setIfDefined<T>()`
- `normalizeTesseractConfig()`
- `normalizeOcrConfig()`
- `normalizeChunkingConfig()`
- `normalizeImageExtractionConfig()`
- `normalizePdfConfig()`
- `normalizeTokenReductionConfig()`
- `normalizeLanguageDetectionConfig()`
- `normalizePostProcessorConfig()`
- `normalizeHtmlPreprocessing()`
- `normalizeHtmlOptions()`
- `normalizeKeywordConfig()`
- `normalizePageConfig()`
- `normalizeExtractionConfig()` - main orchestrator

**Dependencies:** types.js

**Circular Dependency Risk:** None (utility module)

---

### 4. `extraction/single.ts` (100-120 lines)
**Contains:**
- `extractFileSync(filePath, mimeTypeOrConfig?, maybeConfig?)`
- `extractFile(filePath, mimeTypeOrConfig?, maybeConfig?)`

**Dependencies:** binding.ts, type-converters.ts, config-normalizer.ts, types.js

**Circular Dependency Risk:** None (depends only on utilities)

---

### 5. `extraction/batch.ts` (80-100 lines)
**Contains:**
- `batchExtractFilesSync(paths, config)`
- `batchExtractFiles(paths, config)`
- `batchExtractBytesSync(dataArray, mimeTypes, config)`
- `batchExtractBytes(dataArray, mimeTypes, config)`

**Dependencies:** binding.ts, type-converters.ts, config-normalizer.ts, types.js

**Helper Functions Needed:**
- `assertUint8ArrayList()` - move from index.ts

**Circular Dependency Risk:** None

---

### 6. `extraction/worker-pool.ts` (150-180 lines)
**Contains:**
- `createWorkerPool(size?)`
- `getWorkerPoolStats(pool)`
- `extractFileInWorker(pool, filePath, mimeTypeOrConfig?, maybeConfig?)`
- `batchExtractFilesInWorker(pool, paths, config)`
- `closeWorkerPool(pool)`

**Dependencies:** binding.ts, type-converters.ts, config-normalizer.ts, types.js

**Circular Dependency Risk:** None

---

### 7. `plugins/post-processor.ts` (100-120 lines)
**Contains:**
- `registerPostProcessor(processor)`
- `unregisterPostProcessor(name)`
- `clearPostProcessors()`
- `listPostProcessors()`

**Internal Logic:**
- Complex wrapping logic that converts ExtractionResult to/from JSON
- Handling of both function and method-based property access

**Dependencies:** binding.ts, types.js

**Circular Dependency Risk:** None

---

### 8. `plugins/validator.ts` (80-100 lines)
**Contains:**
- `registerValidator(validator)`
- `unregisterValidator(name)`
- `clearValidators()`
- `listValidators()`

**Internal Logic:**
- Similar wrapping logic for validators
- JSON serialization/deserialization

**Dependencies:** binding.ts, types.js

**Circular Dependency Risk:** None

---

### 9. `plugins/ocr.ts` (120-150 lines)
**Contains:**
- `registerOcrBackend(backend)`
- `listOcrBackends()`
- `unregisterOcrBackend(name)`
- `clearOcrBackends()`
- `isOcrProcessTuple()` helper
- `isNestedOcrProcessTuple()` helper
- `describePayload()` helper

**Internal Logic:**
- Complex tuple unpacking for OCR payload handling
- Base64 conversion logic
- Environment-variable-based debugging (`KREUZBERG_DEBUG_GUTEN`)

**Dependencies:** binding.ts, types.js

**Circular Dependency Risk:** None

---

### 10. `registry/document-extractor.ts` (40-50 lines)
**Contains:**
- `listDocumentExtractors()`
- `unregisterDocumentExtractor(name)`
- `clearDocumentExtractors()`

**Note:** No register function exported (intentional)

**Dependencies:** binding.ts

**Circular Dependency Risk:** None

---

### 11. `config/extraction-config.ts` (30-50 lines)
**Contains:**
- `ExtractionConfig` object with static methods:
  - `fromFile(filePath)`
  - `discover()`

**Dependencies:** binding.ts

**Circular Dependency Risk:** None

---

### 12. `mime/detection.ts` (60-80 lines)
**Contains:**
- `detectMimeType(bytes)`
- `detectMimeTypeFromPath(filePath, checkExists?)`
- `validateMimeType(mimeType)`
- `getExtensionsForMime(mimeType)`

**Dependencies:** binding.ts

**Circular Dependency Risk:** None

---

### 13. `embeddings/presets.ts` (40-50 lines)
**Contains:**
- `EmbeddingPreset` interface (export from types.js)
- `listEmbeddingPresets()`
- `getEmbeddingPreset(name)`

**Dependencies:** binding.ts, types.js

**Circular Dependency Risk:** None

---

### 14. `errors/diagnostics.ts` (60-80 lines)
**Contains:**
- `getLastErrorCode()`
- `getLastPanicContext()`
- `getErrorCodeName(code)`
- `getErrorCodeDescription(code)`
- `classifyError(errorMessage)`

**Dependencies:** binding.ts, types.js

**Circular Dependency Risk:** None

---

### 15. `testing/binding-mock.ts` (30-40 lines)
**Contains:**
- `__setBindingForTests(mock)` - internal
- `__resetBindingForTests()` - internal

**Dependencies:** core/binding.ts

**Circular Dependency Risk:** Potential (if test module imports from main index)
**Mitigation:** Use type-only imports where possible

---

### 16. `index.ts` (NEW - Re-export module) (50-100 lines)
**Contains:**
- Re-exports from all module subdirectories
- Re-exports from "./types.js"
- Re-exports from "./errors.js"
- Re-exports from "./ocr/guten-ocr.js"
- `__version__` constant

**Structure:**
```typescript
export {
  // Core
  extractFile, extractFileSync,
  extractBytes, extractBytesSync,
  batchExtractFiles, batchExtractFilesSync,
  batchExtractBytes, batchExtractBytesSync,

  // Extraction
  extractFileInWorker,
  batchExtractFilesInWorker,
  createWorkerPool,
  closeWorkerPool,
  getWorkerPoolStats,

  // Plugins
  registerPostProcessor, unregisterPostProcessor, clearPostProcessors, listPostProcessors,
  registerValidator, unregisterValidator, clearValidators, listValidators,
  registerOcrBackend, unregisterOcrBackend, clearOcrBackends, listOcrBackends,

  // Registry
  listDocumentExtractors, unregisterDocumentExtractor, clearDocumentExtractors,

  // Config
  ExtractionConfig,

  // MIME
  detectMimeType, detectMimeTypeFromPath, validateMimeType, getExtensionsForMime,

  // Embeddings
  listEmbeddingPresets, getEmbeddingPreset,

  // Error Handling
  getLastErrorCode, getLastPanicContext, getErrorCodeName, getErrorCodeDescription, classifyError,

  // Testing (internal)
  __setBindingForTests, __resetBindingForTests,

  // Version
  __version__,
} from './modules/index.js';

// Re-exports from external files
export * from './types.js';
export * from './errors.js';
export { GutenOcrBackend } from './ocr/guten-ocr.js';
```

**Circular Dependency Risk:** None (aggregation only)

---

## Internal Helper Functions to Move

### To `core/assertions.ts` (30-50 lines)
- `assertUint8Array(value, name)`
- `assertUint8ArrayList(values, name)`

**Can be used by:**
- extraction/batch.ts

---

## Summary of Line Distribution

| Module | Estimated Lines | Purpose |
|--------|-----------------|---------|
| core/binding.ts | 140 | Native module loading |
| core/type-converters.ts | 180 | Result type conversion |
| core/config-normalizer.ts | 240 | Config normalization |
| core/assertions.ts | 40 | Type assertions |
| extraction/single.ts | 110 | Single file extraction |
| extraction/batch.ts | 100 | Batch extraction |
| extraction/worker-pool.ts | 170 | Worker pool operations |
| plugins/post-processor.ts | 110 | Post-processor plugin registry |
| plugins/validator.ts | 90 | Validator plugin registry |
| plugins/ocr.ts | 140 | OCR backend registry |
| registry/document-extractor.ts | 50 | Document extractor registry |
| config/extraction-config.ts | 45 | Config loading |
| mime/detection.ts | 75 | MIME utilities |
| embeddings/presets.ts | 50 | Embedding presets |
| errors/diagnostics.ts | 75 | Error diagnostics |
| testing/binding-mock.ts | 40 | Test utilities |
| **Total** | **~1,580** | (remaining code to be split into submodules) |

Original: 2,361 lines → After split: ~1,580 lines in modules + index.ts re-exports

---

## Circular Dependency Analysis

### No Circular Dependencies Expected

**Dependency Graph (DAG):**
```
types.js (top-level type definitions - no dependencies)
  ↓
errors.js (error classes - minimal dependencies)
  ↓
core/binding.ts (native binding - no module dependencies except types)
  ↓
├→ core/type-converters.ts (depends on binding.ts, types.js)
├→ core/config-normalizer.ts (depends on types.js only)
├→ core/assertions.ts (depends on nothing - pure utility)
  ↓
├→ extraction/single.ts (depends on converters, binding, config-normalizer)
├→ extraction/batch.ts (depends on converters, binding, config-normalizer, assertions)
├→ extraction/worker-pool.ts (depends on converters, binding, config-normalizer)
  ↓
├→ plugins/post-processor.ts (depends on binding.ts)
├→ plugins/validator.ts (depends on binding.ts)
├→ plugins/ocr.ts (depends on binding.ts)
├→ registry/document-extractor.ts (depends on binding.ts)
├→ config/extraction-config.ts (depends on binding.ts)
├→ mime/detection.ts (depends on binding.ts)
├→ embeddings/presets.ts (depends on binding.ts, types.js)
├→ errors/diagnostics.ts (depends on binding.ts, types.js)
├→ testing/binding-mock.ts (depends on binding.ts)
  ↓
index.ts (aggregates all above - no dependencies except module exports)
```

**Validation:** All dependencies flow downward (acyclic). No module depends on modules that depend on it.

---

## Special Considerations

### 1. Global State Management
- `binding` and `bindingInitialized` variables in core/binding.ts
- Must remain in a single location to ensure singleton pattern
- Testing utilities can reset this state

### 2. Native Binding Lazy Loading
- First call to `getBinding()` loads the native module
- Error handling wrapped in `createNativeBindingError()`
- Subsequent calls return cached binding

### 3. Environment Variable Usage
- `KREUZBERG_DEBUG_GUTEN` for OCR debugging
- Must be preserved in plugins/ocr.ts

### 4. Plugin Registration Wrapping
- Post-processors, Validators, and OCR backends need complex wrapping
- Conversion between TypeScript types and JSON
- Original processor/validator/backend stored in non-enumerable properties

### 5. Configuration Normalization
- Extensive conditional field mapping
- Nested config structures (e.g., htmlOptions.preprocessing)
- Must preserve order of operations in master normalizer

### 6. Type Conversion Complexity
- Metadata parsed from JSON strings
- Chunks, images, pages converted from raw objects
- Results merged from multiple native binding calls

---

## Implementation Strategy

### Phase 1: Core Infrastructure
1. Extract core/binding.ts
2. Extract core/type-converters.ts
3. Extract core/config-normalizer.ts
4. Extract core/assertions.ts

### Phase 2: Extraction APIs
5. Extract extraction/single.ts
6. Extract extraction/batch.ts
7. Extract extraction/worker-pool.ts

### Phase 3: Plugin System
8. Extract plugins/post-processor.ts
9. Extract plugins/validator.ts
10. Extract plugins/ocr.ts

### Phase 4: Utilities and Testing
11. Extract registry/document-extractor.ts
12. Extract config/extraction-config.ts
13. Extract mime/detection.ts
14. Extract embeddings/presets.ts
15. Extract errors/diagnostics.ts
16. Extract testing/binding-mock.ts

### Phase 5: Final
17. Create new index.ts with re-exports
18. Update import paths in all consumers
19. Verify tests pass

---

## Gotchas and Warnings

### 1. Import Path Changes
- All current imports from '@kreuzberg/node' must continue working
- Use consistent re-export pattern in new index.ts

### 2. Native Binding Method Validation
- loadNativeBinding() validates required methods exist
- Must update validation list if new methods added to NativeBinding

### 3. Config Normalization Ordering
- normalizeExtractionConfig() calls multiple specific normalizers
- Order doesn't matter for most configs, but document for clarity

### 4. OCR Tuple Unpacking
- Complex nested tuple handling with multiple guards
- Debug output controlled by KREUZBERG_DEBUG_GUTEN env var
- Must be preserved exactly for compatibility

### 5. Plugin Wrapping Non-Enumerable Properties
- Uses Object.defineProperty() for `__original` and `__stage`
- Must preserve this pattern for proper unwrapping

### 6. Error Handling Context
- getLastErrorCode() and getLastPanicContext() read from native state
- State is global - only valid immediately after error occurs

### 7. Worker Pool Opaque Handle
- WorkerPool type is opaque (passed as Record<string, unknown>)
- Cast only when calling native binding methods
