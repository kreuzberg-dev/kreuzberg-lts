"""Verify the shipped plugin through native Hermes discovery and tool dispatch."""

import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PROBE = """
import json
from hermes_cli.plugins import PluginManager
from tools.registry import registry
manager = PluginManager()
manager.discover_and_load()
names = manager.list_plugin_skills("kreuzberg")
assert names == ["kreuzberg"], f"registered skill count/name mismatch: {names}"
skill = manager.find_plugin_skill("kreuzberg:kreuzberg")
assert skill is not None and skill.is_file(), "registered skill is unreadable"
references = sorted((skill.parent / "references").glob("*.md"))
assert len(references) == 8, f"expected eight references, received {len(references)}"
assert all(path.stat().st_size > 100 for path in references), "empty reference"
result = registry.dispatch("kreuzberg_call", {"tool": "get_version"})
assert isinstance(result, str), "Hermes result must use JSON text"
value = json.loads(result)
assert "Install Kreuzberg 4.x" in value.get("error", ""), result
assert "unsupported resulttype" not in result, result
print("Hermes native discovery: one skill, eight references, JSON-text dispatch")
"""


class HermesSkillTests(unittest.TestCase):
    def probe(self, remove_skill: bool) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory(prefix="kreuzberg-hermes-contract-") as temporary:
            home = Path(temporary)
            plugin = home / "plugins/kreuzberg"
            shutil.copytree(ROOT / "plugin/.hermes/plugins/kreuzberg", plugin)
            (home / "config.yaml").write_text("plugins:\n  enabled: [kreuzberg]\n")
            bundled = home / "empty-bundled"
            bundled.mkdir()
            if remove_skill:
                (plugin / "skills/kreuzberg/SKILL.md").unlink()
            environment = os.environ | {
                "HERMES_HOME": str(home),
                "HERMES_BUNDLED_PLUGINS": str(bundled),
                "HERMES_ENABLE_PROJECT_PLUGINS": "true",
                "KREUZBERG_LTS_BIN": str(home / "intentionally-missing-cli"),
            }
            return subprocess.run(
                [sys.executable, "-c", PROBE], cwd=home, env=environment, text=True, capture_output=True, check=False
            )

    def test_should_discover_shipped_skills_and_dispatch_json_text(self) -> None:
        result = self.probe(remove_skill=False)
        assert result.returncode == 0, result.stdout + result.stderr
        assert "one skill, eight references, JSON-text dispatch" in result.stdout

    def test_should_reject_missing_registered_skill_bytes(self) -> None:
        result = self.probe(remove_skill=True)
        assert result.returncode != 0
        assert "registered skill count/name mismatch" in result.stderr


if __name__ == "__main__":
    unittest.main()
