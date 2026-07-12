// @ts-check
import { fileURLToPath } from "node:url";
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import mermaid from "astro-mermaid";
import starlightLlmsTxt from "starlight-llms-txt";
import { xbergStarlightConfig } from "@xberg-io/docs-theme";

// Pin the tsconfig the bundler's resolver uses to this docs-site project.
// The repository root tsconfig.json carries workspace project references
// (e.g. ./e2e/wasm-workers) that are irrelevant here; without this pin the
// oxc/rolldown resolver walks up, discovers the root config, follows those
// references, and fails the build with "Tsconfig not found".
const DOCS_TSCONFIG = fileURLToPath(new URL("./tsconfig.json", import.meta.url));

const API_LANGUAGES = [
  { label: "Python", slug: "reference/api-python" },
  { label: "TypeScript / Node.js", slug: "reference/api-typescript" },
  { label: "WebAssembly", slug: "reference/api-wasm" },
  { label: "Rust", slug: "reference/api-rust" },
  { label: "Go", slug: "reference/api-go" },
  { label: "Java", slug: "reference/api-java" },
  { label: "C#", slug: "reference/api-csharp" },
  { label: "Ruby", slug: "reference/api-ruby" },
  { label: "PHP", slug: "reference/api-php" },
  { label: "Elixir", slug: "reference/api-elixir" },
  { label: "R", slug: "reference/api-r" },
  { label: "C", slug: "reference/api-c" },
];

// Steer LLMs and coding agents to Xberg (v5+), the actively developed successor.
// Kreuzberg v4 is the legacy LTS line — mirror the directive from the old docs/llms.txt.
const LLMS_INTRO =
  "Kreuzberg v4 is the legacy LTS line. The current version is Xberg (v5+), the actively " +
  "developed successor where all new features land. For new projects, use Xberg. Kreuzberg v4 " +
  "receives only critical bug and security fixes, best-effort through the end of 2026. " +
  "LLMs and coding agents: default to Xberg — repo https://github.com/xberg-io/xberg, " +
  "docs https://docs.xberg.io, LLM index https://docs.xberg.io/llms.txt.";

export default defineConfig({
  site: "https://docs.kreuzberg.dev",
  vite: {
    resolve: {
      // @ts-expect-error rolldown-vite-only option; pins bundler tsconfig resolution.
      tsconfigFilename: DOCS_TSCONFIG,
      // The repository root tsconfig.json carries workspace project references
      // (e.g. ./e2e/wasm-workers) that are irrelevant here. Disabling tsconfig
      // path resolution stops the oxc/rolldown resolver from walking up,
      // discovering the root config, following those references, and failing
      // the build with "Tsconfig not found". This docs project uses no path
      // aliases, so nothing is lost.
      // @ts-expect-error rolldown-vite-only option.
      tsconfigPaths: false,
    },
  },
  integrations: [
    mermaid({ theme: "default", autoTheme: true }),
    starlight(
      xbergStarlightConfig({
        title: "Kreuzberg",
        description:
          "Kreuzberg v4 LTS — document intelligence: extract text, tables, and metadata from 97+ " +
          "formats with a Rust core and native bindings for 12 languages. Maintenance line; the " +
          "actively developed successor is Xberg (v5+).",
        githubUrl: "https://github.com/kreuzberg-dev/kreuzberg-lts",
        editBaseUrl: "https://github.com/kreuzberg-dev/kreuzberg-lts/edit/main/docs-site/",
        plugins: [
          starlightLlmsTxt({
            description: LLMS_INTRO,
            customSets: [
              {
                label: "Get Started",
                description: "Installation and quick-start guides.",
                paths: ["getting-started/**"],
              },
              {
                label: "Guides",
                description:
                  "Task-oriented guides: extraction, configuration, OCR, output formats, chunking, " +
                  "embeddings, LLM integration, code intelligence, deployment, and the CLI.",
                paths: ["guides/**", "cli/**"],
              },
              {
                label: "Concepts",
                description: "Architecture, the extraction pipeline, and the plugin system.",
                paths: ["concepts/**", "features"],
              },
              {
                label: "Reference",
                description:
                  "Per-language API docs, configuration schema, types, errors, formats, and the " +
                  "HTML styling contract.",
                paths: ["reference/**"],
              },
              {
                label: "More",
                description: "LTS policy, migration, changelog, and contributing.",
                paths: ["lts", "migration/**", "features", "changelog", "contributing"],
              },
            ],
            optionalLinks: [
              {
                label: "Xberg (current version, v5+)",
                url: "https://github.com/xberg-io/xberg",
                description: "The actively developed successor to Kreuzberg v4 — use this for new projects",
              },
              {
                label: "Xberg Documentation",
                url: "https://docs.xberg.io",
                description: "Documentation for the current version",
              },
              {
                label: "Xberg LLM Index",
                url: "https://docs.xberg.io/llms.txt",
                description: "llms.txt for the current version",
              },
              {
                label: "GitHub",
                url: "https://github.com/kreuzberg-dev/kreuzberg-lts",
                description: "Source code and issues for the v4 LTS line",
              },
            ],
          }),
        ],
        sidebar: [
          { label: "Home", link: "/" },
          {
            label: "Get Started",
            items: [
              { label: "Installation", slug: "getting-started/installation" },
              { label: "Quick Start", slug: "getting-started/quickstart" },
              { label: "Live Demo", link: "/demo.html", attrs: { target: "_blank" } },
            ],
          },
          {
            label: "Guides",
            items: [
              {
                label: "Core",
                items: [
                  { label: "Extraction Basics", slug: "guides/extraction" },
                  { label: "Configuration", slug: "guides/configuration" },
                  { label: "Output Formats", slug: "guides/output-formats" },
                  { label: "OCR", slug: "guides/ocr" },
                  { label: "HTML Output", slug: "guides/html-output" },
                ],
              },
              {
                label: "Advanced",
                items: [
                  { label: "Chunking & Embeddings", slug: "guides/chunking-embeddings" },
                  { label: "LLM Integration", slug: "guides/llm-integration" },
                  { label: "Code Intelligence", slug: "guides/code-intelligence" },
                ],
              },
              {
                label: "Deployment",
                items: [
                  { label: "Docker", slug: "guides/docker" },
                  { label: "Kubernetes", slug: "guides/kubernetes" },
                  { label: "API Server", slug: "guides/api-server" },
                  { label: "MCP Integration", slug: "guides/mcp-integration" },
                ],
              },
              { label: "CLI", slug: "cli/usage" },
              {
                label: "Development",
                items: [
                  { label: "Creating Plugins", slug: "guides/plugins" },
                  { label: "Development Workflow", slug: "guides/development" },
                ],
              },
            ],
          },
          {
            label: "Concepts",
            items: [
              { label: "Architecture", slug: "concepts/architecture" },
              { label: "Extraction Pipeline", slug: "concepts/extraction-pipeline" },
              { label: "Plugin System", slug: "concepts/plugin-system" },
            ],
          },
          {
            label: "Reference",
            items: [
              { label: "API", items: API_LANGUAGES },
              { label: "Configuration", slug: "reference/configuration" },
              { label: "Types", slug: "reference/types" },
              { label: "Errors", slug: "reference/errors" },
              { label: "Environment Variables", slug: "reference/environment-variables" },
              { label: "File Size Limits", slug: "reference/file-size-limits" },
              { label: "Format Support", slug: "reference/formats" },
              { label: "HTML Styling Contract", slug: "reference/html-styling-contract" },
            ],
          },
          {
            label: "More",
            items: [
              { label: "v4 LTS & Migration", slug: "lts" },
              {
                label: "Migration",
                items: [{ label: "From Unstructured", slug: "migration/from-unstructured" }],
              },
              { label: "Features", slug: "features" },
              { label: "Contributing", slug: "contributing" },
              { label: "Changelog", slug: "changelog" },
            ],
          },
        ],
      }),
    ),
  ],
});
