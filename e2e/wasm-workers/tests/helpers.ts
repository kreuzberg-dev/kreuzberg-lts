/**
 * E2E test helpers for WASM workers - Thin adapter using @kreuzberg/test-utils
 * This file re-exports and adapts utilities from the shared test-utils package
 */

import type { ExtractionResult } from "@kreuzberg/wasm";
import {
	buildConfig,
	type ExtractionConfig,
} from "@kreuzberg/test-utils/config-mapping";
import {
	createAssertions,
	type ExtractionAssertions,
	VitestAdapter,
} from "@kreuzberg/test-utils/assertions";
import { shouldSkipFixture } from "@kreuzberg/test-utils/fixtures";

// Re-export core utilities
export { buildConfig, shouldSkipFixture };

// Create and export assertions instance using VitestAdapter
export const assertions: ExtractionAssertions<ExtractionResult> =
	createAssertions<ExtractionResult>(new VitestAdapter());

/**
 * Get fixture for WASM workers environment
 * Note: Cloudflare Workers cannot access the filesystem, so this always returns null
 */
export function getFixture(fixturePath: string): Uint8Array | null {
	console.warn(`[SKIP] Cloudflare Workers cannot load fixtures from disk. Fixture: ${fixturePath}`);
	console.warn("[SKIP] These tests require filesystem access which is not available in the Workers sandbox.");
	return null;
}
