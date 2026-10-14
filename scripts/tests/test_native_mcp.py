"""Verify actual Kreuzberg 4.x MCP startup and extraction; requires its published CLI."""

import base64
import importlib.util
import json
import os
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class NativeMcpTests(unittest.IsolatedAsyncioTestCase):
    def setUp(self) -> None:
        assert os.environ.get("KREUZBERG_LTS_BIN"), "Set KREUZBERG_LTS_BIN to the verified 4.x MCP executable"
        spec = importlib.util.spec_from_file_location(
            "native_adapter", ROOT / "plugin/.hermes/plugins/kreuzberg/hermes.py"
        )
        assert spec is not None and spec.loader is not None
        self.adapter = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(self.adapter)

    async def test_should_expose_all_thirteen_actual_tool_schemas(self) -> None:
        result = await self.adapter.call_tool({"tool": "list_tools"})
        assert sorted(tool["name"] for tool in result.get("tools", [])) == sorted(self.adapter.TOOL_NAMES), result
        assert len(result["tools"]) == 13
        extraction = next(tool for tool in result["tools"] if tool["name"] == "extract_bytes")
        assert "data" in extraction["inputSchema"]["required"]

    async def test_should_extract_real_bytes_and_return_a_flat_v4_result(self) -> None:
        result = await self.adapter.handle_tool(
            {
                "tool": "extract_bytes",
                "tool_arguments": {
                    "data": base64.b64encode(b"Kreuzberg LTS native contract").decode(),
                    "mime_type": "text/plain",
                    "config": {"use_cache": False},
                },
            }
        )
        envelope = json.loads(result)
        assert not envelope.get("isError", False), envelope
        extraction = json.loads(envelope["content"][0]["text"])
        assert extraction["content"].strip() == "Kreuzberg LTS native contract"
        assert "result" not in extraction


if __name__ == "__main__":
    unittest.main()
