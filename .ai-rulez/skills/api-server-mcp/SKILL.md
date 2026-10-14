---
type: Playbook
title: Api Server Mcp
description: Maintain the Kreuzberg 4.x REST router, MCP tools, transports and error contracts
x-ai-rulez:
  kind: skill
  id: api-server-mcp
  metadata:
    priority: critical
    name: api-server-mcp
---

# API Server and MCP Protocol

Use the 4.x implementation in this repository. REST and MCP have separate transports; do not invent REST subroutes for MCP tools.

## REST routes

`crates/kreuzberg/src/api/router.rs` registers:

- POST `/extract`, `/extract-structured`, `/detect`, `/embed`, `/chunk`, `/cache/warm`.
- GET `/formats`, `/health`, `/info`, `/version`, `/cache/stats`, `/cache/manifest`.
- DELETE `/cache/clear`.
- PUT `/process` and POST `/v1/convert/file` for Open WebUI compatibility.
- GET `/openapi.json` when the `api` feature is enabled.

Read the individual handler request types before constructing requests. The router applies configurable body limits, CORS, tracing, request IDs, compression and panic handling. Restrict production CORS with `KREUZBERG_CORS_ORIGINS`; the permissive default is not a security boundary.

`api/error.rs` maps validation and unsupported formats to HTTP 400, parsing/OCR failures to 422, and other extraction errors to 500. JSON responses carry `error_type`, `message`, optional `traceback`, and `status_code`. JSON syntax, content type and multipart rejection handling have their own statuses. Preserve those contracts when changing handlers.

## MCP tools and transport

`crates/kreuzberg/src/mcp/server.rs` implements thirteen tools:

`extract_file`, `extract_bytes`, `batch_extract_files`, `detect_mime_type`, `cache_stats`, `list_formats`, `cache_clear`, `get_version`, `cache_manifest`, `cache_warm`, `embed_text`, `extract_structured`, `chunk_text`.

Read schemas in `mcp/params.rs`. File extraction uses `path`, bytes extraction uses base64 `data`, and batch extraction uses `paths`. Extraction tools accept an optional `config` JSON object and `response_format` (`json` or `toon`); batches additionally accept `file_configs`, a parallel array of optional per-file configurations. Do not substitute `file_path`, `file_paths`, or arbitrary flattened extraction flags.

MCP results are `CallToolResult` values: decode the text content using the requested wire format and inspect the MCP error flag. JSON extraction payloads use the 4.x flat `ExtractionResult`; a batch payload is an array. Do not apply a newer product's result envelope.

Run `kreuzberg mcp --transport stdio` for local JSON-RPC. The CLI must be built with its `mcp` feature; HTTP transport additionally requires `mcp-http` and serves the streamable HTTP service at `/mcp`. There are no implemented MCP resource or prompt catalogs to advertise.

```json
{
  "mcpServers": {
    "kreuzberg": {
      "command": "kreuzberg",
      "args": ["mcp", "--transport", "stdio"]
    }
  }
}
```

Keep diagnostics off stdio protocol stdout. Check optional dependencies before promising OCR, embeddings or provider-backed structured extraction. Provider credentials belong in the environment. Exercise actual `tools/list` and a tool call after modifying registration or adapters; listing tool names alone does not prove dispatch works.
