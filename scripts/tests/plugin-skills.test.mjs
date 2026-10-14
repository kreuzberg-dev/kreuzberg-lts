import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import path from "node:path";
import test from "node:test";
import plugin from "../../plugin/.opencode/plugins/kreuzberg.js";

const root = path.resolve(import.meta.dirname, "../..");

test("OpenCode registers the installed Kreuzberg LTS skill once with readable references", async () => {
  const registered = new Map();
  const servers = new Map();
  const context = {
    mcp: {
      transform(callback) {
        callback({ set: (name, server) => servers.set(name, server) });
      },
    },
    skill: {
      transform(callback) {
        callback({ get: (id) => registered.get(id), add: (skill) => registered.set(skill.id, skill) });
      },
    },
  };
  await plugin.setup(context);
  await plugin.setup(context);
  assert.equal(registered.size, 1);
  assert.equal(servers.size, 1);
  assert.equal(servers.get("kreuzberg").type, "local");
  assert.ok(servers.get("kreuzberg").command[0].endsWith("/scripts/mcp-launch.sh"));
  const skill = registered.get("kreuzberg/kreuzberg");
  assert.equal(skill.name, "kreuzberg");
  assert.match(skill.content, /kreuzberg/u);
  for (const resource of [
    "advanced-features",
    "cli-reference",
    "configuration",
    "nodejs-api",
    "other-bindings",
    "python-api",
    "rust-api",
    "supported-formats",
  ]) {
    const content = await readFile(path.join(path.dirname(skill.path), `references/${resource}.md`), "utf8");
    assert.ok(content.length > 100, `resource missing content: ${resource}`);
  }
});

test("Pi package ships every declared skill and its eight references", async () => {
  const manifest = JSON.parse(await readFile(path.join(root, "plugin/package.json"), "utf8"));
  assert.deepEqual(manifest.pi.skills, ["./.pi/skills/kreuzberg"]);
  assert.deepEqual(manifest.pi.prompts, []);
  for (const item of [
    "SKILL.md",
    "index.md",
    ...[
      "advanced-features",
      "cli-reference",
      "configuration",
      "nodejs-api",
      "other-bindings",
      "python-api",
      "rust-api",
      "supported-formats",
    ].map((name) => `references/${name}.md`),
  ]) {
    const file = `.pi/skills/kreuzberg/${item}`;
    assert.ok(manifest.files.includes(file), `npm file list omitted ${file}`);
    assert.ok((await readFile(path.join(root, "plugin", file))).length > 0);
  }
});
