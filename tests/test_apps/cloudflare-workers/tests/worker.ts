import { extractBytes, getVersion, initWasm, isInitialized } from "@kreuzberg/wasm";
import wasmModule from "@kreuzberg/wasm/kreuzberg_wasm_bg.wasm";

export default {
	async fetch(request: Request): Promise<Response> {
		try {
			if (!isInitialized()) {
				await initWasm({ wasmModule });
			}

			const version = getVersion();

			if (request.method === "POST") {
				const contentType = request.headers.get("content-type") || "application/octet-stream";
				const arrayBuffer = await request.arrayBuffer();
				const bytes = new Uint8Array(arrayBuffer);
				const result = await extractBytes(bytes, contentType);
				return new Response(JSON.stringify({ version, result }), {
					headers: { "content-type": "application/json" },
				});
			}

			return new Response(JSON.stringify({ version, initialized: isInitialized() }), {
				headers: { "content-type": "application/json" },
			});
		} catch (error) {
			const message = error instanceof Error ? error.message : String(error);
			return new Response(JSON.stringify({ error: message }), {
				status: 500,
				headers: { "content-type": "application/json" },
			});
		}
	},
};
