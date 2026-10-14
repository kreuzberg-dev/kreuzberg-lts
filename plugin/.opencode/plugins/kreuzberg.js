// AI-RULEZ :: GENERATED FILE — DO NOT EDIT
// Content-Hash: blake3:a8a7aa35477a5f99890b713c11490fc4f3fe1d5e83b606cf1ec0792cd35a295b
// Source-Hash: blake3:901163c93cd1f09795c49bf6078d600e691112f66b88467ba8b237dff29330c4
// Schema-Version: v1

/**
 * OpenCode v2 adapter for kreuzberg.
 *
 * To add OpenCode-specific tools or hooks:
 *
 * 1. Create .ai-rulez/opencode/index.js.
 * 2. Default-export { id, setup(ctx) } from that source file. OpenCode v2 does not
 *    run v1 plugins (an exported function returning hooks).
 * 3. If the plugin bundles skills, commands, or agents, call
 *    registerBundledContent(ctx) from ../ai-rulez-content.js in setup.
 * 4. Run ai-rulez generate --plugin --dry-run, then ai-rulez generate --plugin.
 *
 * Keep shared skills, commands, agents, and MCP configuration in their normal
 * .ai-rulez sources. Validate all external input and never interpolate untrusted
 * values into shell commands.
 */
import { registerBundledContent } from "../ai-rulez-content.js"

/** @type {import("@opencode/plugin").Plugin.Plugin} */
export default {
  id: "kreuzberg",
  async setup(ctx) {
    await registerBundledContent(ctx)
  },
}
