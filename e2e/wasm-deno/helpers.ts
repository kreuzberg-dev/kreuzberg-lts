// @deno-types="../../crates/kreuzberg-wasm/dist/index.d.ts"
import type {
	ChunkingConfig,
	ExtractionConfig,
	ExtractionResult,
	ImageExtractionConfig,
	LanguageDetectionConfig,
	Metadata,
	OcrConfig,
	PdfConfig,
	PostProcessorConfig,
	Table,
	TesseractConfig,
	TokenReductionConfig,
} from "npm:@kreuzberg/wasm@^4.0.0";
// @deno-types="../../crates/kreuzberg-wasm/dist/index.d.ts"
import { extractBytes, initWasm } from "npm:@kreuzberg/wasm@^4.0.0";

// Re-exports from @kreuzberg/test-utils
import { buildConfig as buildConfigFromUtils } from "npm:@kreuzberg/test-utils@^0.1.0";
import { shouldSkipFixture as shouldSkipFixtureFromUtils } from "npm:@kreuzberg/test-utils@^0.1.0";
import {
	createAssertions,
	DenoAdapter,
	type ExtractionAssertions,
} from "npm:@kreuzberg/test-utils@^0.1.0";

export type {
	ChunkingConfig,
	ExtractionConfig,
	ExtractionResult,
	ImageExtractionConfig,
	LanguageDetectionConfig,
	Metadata,
	OcrConfig,
	PdfConfig,
	PostProcessorConfig,
	Table,
	TesseractConfig,
	TokenReductionConfig,
};

export { extractBytes, initWasm };

// Deno-specific document path resolver
const WORKSPACE_ROOT = new URL("../..", import.meta.url).pathname;
const TEST_DOCUMENTS = `${WORKSPACE_ROOT}/test_documents`;

export async function resolveDocument(relative: string): Promise<Uint8Array> {
	const path = `${TEST_DOCUMENTS}/${relative}`;
	return await Deno.readFile(path);
}

// Re-export config building from test-utils
export const buildConfig = buildConfigFromUtils;

// Re-export fixture skip handler from test-utils
export const shouldSkipFixture = shouldSkipFixtureFromUtils;

// Create assertions with Deno adapter
export const assertions: ExtractionAssertions<ExtractionResult> = createAssertions(
	new DenoAdapter(),
);
