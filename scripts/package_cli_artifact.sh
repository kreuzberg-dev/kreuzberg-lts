#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-${TARGET:-}}"
if [[ -z "$TARGET" ]]; then
  echo "Usage: $0 <target>" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STAGE="$ROOT/kreuzberg-cli-${TARGET}"
rm -rf "$STAGE"
mkdir -p "$STAGE"
cp "$ROOT/target/${TARGET}/release/kreuzberg" "$STAGE/"
cp "$ROOT/LICENSE" "$STAGE/"
cp "$ROOT/README.md" "$STAGE/"

if [[ -f "$ROOT/target/${TARGET}/release/libpdfium.dylib" ]]; then
  cp "$ROOT/target/${TARGET}/release/libpdfium.dylib" "$STAGE/"
fi
if [[ -f "$ROOT/target/${TARGET}/release/libpdfium.so" ]]; then
  cp "$ROOT/target/${TARGET}/release/libpdfium.so" "$STAGE/"
fi

# ~keep Bundle a CPU ONNX Runtime next to the binary. The CLI is built with
# ort-dynamic, so it dlopens ONNX Runtime at runtime and honors ORT_DYLIB_PATH
# (e.g. a GPU-enabled build). Without this bundled fallback the CPU-only default
# would fail to load when the environment variable is unset. RPATH (@loader_path
# on macOS, $ORIGIN/lib on Linux) resolves the fallback library relative to the
# executable. Download location is passed via ORT_BUNDLE_DIR (official ORT tgz).
if [[ -n "${ORT_BUNDLE_DIR:-}" && -d "$ORT_BUNDLE_DIR/lib" ]]; then
  mkdir -p "$STAGE/lib"
  case "$(uname -s)" in
  Darwin)
    cp -a "$ORT_BUNDLE_DIR"/lib/libonnxruntime*.dylib "$STAGE/lib/" 2>/dev/null || true
    # dylib load path must survive re-signing, so rpath first, then re-sign.
    install_name_tool -add_rpath '@loader_path/lib' "$STAGE/kreuzberg" 2>/dev/null || true
    codesign --force --sign - "$STAGE/kreuzberg" 2>/dev/null || true
    ;;
  Linux)
    cp -a "$ORT_BUNDLE_DIR"/lib/libonnxruntime.so* "$STAGE/lib/" 2>/dev/null || true
    command -v patchelf >/dev/null 2>&1 || {
      echo "error: patchelf is required on Linux to set the ORT RPATH" >&2
      exit 1
    }
    ORT_RPATH="\$ORIGIN/lib"
    patchelf --set-rpath "$ORT_RPATH" "$STAGE/kreuzberg"
    ;;
  *)
    cp -a "$ORT_BUNDLE_DIR"/lib/libonnxruntime.so* "$STAGE/lib/" 2>/dev/null || true
    ;;
  esac
else
  echo "skipping ONNX Runtime bundling (ORT_BUNDLE_DIR is unset or empty)" >&2
fi

tar -czf "kreuzberg-cli-${TARGET}.tar.gz" -C "$ROOT" "kreuzberg-cli-${TARGET}"
rm -rf "$STAGE"
