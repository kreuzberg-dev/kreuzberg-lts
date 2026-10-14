"""Exercise the LTS plugin launcher version and MCP boundary."""

import os
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
LAUNCHER = ROOT / "plugin/.ai-rulez/scripts/mcp-launch.sh"


class LauncherTests(unittest.TestCase):
    def launch(self, version: str, mcp: bool) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory(prefix="lts-launcher-") as temporary:
            binary = Path(temporary) / "kreuzberg"
            binary.write_text(
                '#!/bin/sh\ncase "$1" in\n--version) echo \'kreuzberg '
                + version
                + "';;\nmcp) "
                + ("echo 'MCP invoked'" if mcp else "exit 2")
                + ";;\nesac\n"
            )
            binary.chmod(0o755)
            return subprocess.run(
                ["bash", str(LAUNCHER), "mcp", "--transport", "stdio"],
                env=os.environ | {"KREUZBERG_LTS_BIN": str(binary)},
                text=True,
                capture_output=True,
                check=False,
            )

    def test_should_launch_a_v4_cli_with_mcp_support(self) -> None:
        result = self.launch("4.10.4", True)
        assert result.returncode == 0, result.stderr
        assert "MCP invoked" in result.stdout

    def test_should_reject_a_v5_cli_before_protocol_startup(self) -> None:
        result = self.launch("5.0.0", True)
        assert result.returncode == 1
        assert "Kreuzberg 4.x" in result.stderr

    def test_should_reject_a_v4_cli_without_mcp_support(self) -> None:
        result = self.launch("4.10.4", False)
        assert result.returncode == 1
        assert "MCP support" in result.stderr


if __name__ == "__main__":
    unittest.main()
