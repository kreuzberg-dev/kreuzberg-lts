# Kreuzberg LTS coding-agent plugin

This plugin targets Kreuzberg 4.x, not its actively developed Xberg successor. It supplies one product skill, eight references and MCP integration where the host supports it. Private repository-maintenance skills are not published.

Install a Kreuzberg 4.x CLI separately. Use the v4.10.4 binaries from <https://github.com/kreuzberg-dev/kreuzberg-lts/releases/tag/v4.10.4>, or `cargo install kreuzberg-cli --version 4.10.4 --features mcp`. Set `KREUZBERG_LTS_BIN` to its executable when PATH selects another version. The launcher rejects v5 or builds without MCP support. Optional OCR/model/provider features need their actual dependencies and credentials.

| Agent | Installation |
| --- | --- |
| Claude | Add this repository as marketplace `kreuzberg-lts`, install `kreuzberg@kreuzberg-lts` |
| Codex | `codex plugin marketplace add <repository-checkout>` then `codex plugin add kreuzberg@kreuzberg-lts` |
| Cursor | Install the generated `.cursor-plugin` bundle or import the published Cursor team marketplace |
| Agent Plugins | Install `plugin/` using its portable `plugin.json` manifest |
| Copilot | Install the portable `plugin/plugin.json` bundle with the host's plugin installer |
| Gemini | Link `plugin/` as an extension; `gemini-extension.json` refers to bundled context |
| Kimi | Use `plugin/kimi.plugin.json` and the bundled MCP configuration |
| OpenCode v2 | Load `plugin/.opencode/plugins/kreuzberg.js`; its setup hook registers the installed skill |
| Factory | Install the generated `.factory-plugin` manifest and common bundle |
| Pi | Install `plugin/` as a package; Pi gets skills and prompts, not an MCP server |
| Hermes | Install `.hermes/plugins/kreuzberg` or build its wheel; enable `kreuzberg` and install declared Python dependencies |

Read each generated host manifest before choosing its installer. Configure the launcher environment in the host; no plugin artifact bundles the document extractor or API credentials. Hermes registers a `kreuzberg_call` bridge whose `list_tools` choice exposes the actual schemas and whose handler returns the host's JSON-text result contract.

Maintainers: install ai-rulez 5.3.0 with the reviewed source revision pinned in CI. Run `task plugin:generate`, `task plugin:verify`, and the plugin contract tests. Public `skills/` exports are derived byte-for-byte from native generation with a hash ownership manifest. The manual plugin-artifacts workflow verifies the selected checkout against an existing release tag and runs three distribution jobs: a deterministic GitHub archive, the npm OpenCode/Pi package and the PyPI Hermes wheel. It does not create a new product release. Configure the required registry trusted publishers before dispatching; see [plugin publishing](https://github.com/kreuzberg-dev/kreuzberg-lts/blob/main/.github/PLUGIN-PUBLISHING.md). Use the verified local generated bundles until the first registry publication is complete.
