#!/bin/sh
# Vendor a .node's non-system shared-library closure next to it so the published
# npm platform package is self-contained.
#
# Usage: vendor-node-libs.sh <bin-dir> <out-dir>
#
# Resolves the recursive shared-library dependencies of every *.node in
# <bin-dir> and copies each non-base library into <out-dir>. The published npm
# platform package places these beside the .node, whose RUNPATH is $ORIGIN, so
# the dynamic loader finds them at require() time — without this, a dynamically
# linked ORT build fails on the consumer with "cannot find libonnxruntime"
# (ERR_DLOPEN_FAILED).
#
# Base libraries — the dynamic loader, libc, the compiler runtime
# (libgcc_s/libstdc++) and system TLS (libssl/libcrypto) — are intentionally NOT
# vendored: they are guaranteed present at the manylinux_2_28 / Alpine floor, and
# bundling libstdc++ into a Node addon risks an ABI clash with Node's own copy.
#
# POSIX sh; runs unchanged under glibc (manylinux) and musl (Alpine busybox).
set -eu

bin_dir="${1:?usage: vendor-node-libs.sh <bin-dir> <out-dir>}"
out_dir="${2:?usage: vendor-node-libs.sh <bin-dir> <out-dir>}"
mkdir -p "$out_dir"

queue="$(mktemp)"
seen="$(mktemp)"
trap 'rm -f "$queue" "$seen"' EXIT

# Seed the work queue with every built .node.
for node in "$bin_dir"/*.node; do
  [ -e "$node" ] && printf '%s\n' "$node" >>"$queue"
done

is_base_lib() {
  case "$1" in
  ld-linux*| ld-musl* | libc.so* | libc.musl* | libc-*.so* | libm.so* | libmvec.so* | \
    libdl.so* | librt.so* | libpthread.so* | libresolv.so* | libgcc_s.so* | \
    libstdc++.so* | libssl.so* | libcrypto.so*) return 0 ;;
  *) return 1 ;;
  esac
}

while [ -s "$queue" ]; do
  bin="$(head -n1 "$queue")"
  tail -n +2 "$queue" >"$queue.tmp" && mv "$queue.tmp" "$queue"

  # Extract the absolute paths of the binary's resolved shared-lib deps. Handles
  # both `libfoo.so => /path (0x..)` and bare `/path (0x..)` ldd output lines.
  ldd "$bin" 2>/dev/null |
    sed -n 's/.*=> *\(\/[^ ]*\).*/\1/p; s/^[[:space:]]*\(\/[^ ]*\) (0x[0-9a-f]*)$/\1/p' |
    while IFS= read -r lib; do
      [ -f "$lib" ] || continue
      base="$(basename "$lib")"
      is_base_lib "$base" && continue
      grep -qxF "$base" "$seen" 2>/dev/null && continue
      printf '%s\n' "$base" >>"$seen"
      cp -L "$lib" "$out_dir/$base"
      chmod u+w "$out_dir/$base" 2>/dev/null || true
      printf '%s\n' "$out_dir/$base" >>"$queue"
      echo "vendored $base"
    done
done
