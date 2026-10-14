#!/usr/bin/env bash
set -euo pipefail

fail() {
  printf 'kreuzberg-lts: %s\n' "$*" >&2
  exit 1
}

binary="${KREUZBERG_LTS_BIN:-}"
if [ -z "$binary" ]; then
  binary="$(command -v kreuzberg || true)"
fi
[ -n "$binary" ] && [ -x "$binary" ] || fail "Install Kreuzberg 4.x with MCP support; set KREUZBERG_LTS_BIN to its executable."
version="$("$binary" --version 2>/dev/null)" || fail "The selected executable is broken; install Kreuzberg 4.x with MCP support."
case "$version" in
"kreuzberg 4."*) ;;
*) fail "Expected Kreuzberg 4.x, received '$version'; do not use a v5 CLI with the LTS plugin." ;;
esac
"$binary" mcp --help >/dev/null 2>&1 || fail "The selected CLI lacks MCP support. Build kreuzberg-cli 4.10.4 with --features mcp."
exec "$binary" "$@"
