import { defineConfig } from "tsup";

export default defineConfig({
	entry: [
		"typescript/index.ts",
		"typescript/cli.ts",
		"typescript/errors.ts",
		"typescript/errors/diagnostics.ts",
		"typescript/types.ts",
		"typescript/ocr/guten-ocr.ts",
		"typescript/extraction/batch.ts",
		"typescript/extraction/single.ts",
		"typescript/extraction/worker-pool.ts",
		"typescript/plugins/post-processors.ts",
		"typescript/plugins/validators.ts",
		"typescript/plugins/ocr-backends.ts",
		"typescript/registry/document-extractors.ts",
		"typescript/config/loader.ts",
		"typescript/mime/utilities.ts",
		"typescript/embeddings/presets.ts",
		"typescript/core/binding.ts",
	],
	format: ["esm", "cjs"],
	bundle: false,
	dts: {
		compilerOptions: {
			skipLibCheck: true,
			skipDefaultLibCheck: true,
		},
	},
	splitting: false,
	sourcemap: true,
	clean: true,
	shims: false,
	platform: "node",
	target: "node22",
	external: ["sharp", "@gutenye/ocr-node", /\.node$/, /@kreuzberg\/node-.*/, "./index.js", "../index.js"],
});
