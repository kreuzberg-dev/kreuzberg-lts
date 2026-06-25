# Third-Party Licenses

Xberg itself is licensed under [MIT](LICENSE). This file documents
notable third-party **native** libraries that Xberg links against or that are
redistributed in the published container images, with an emphasis on copyleft
(LGPL/GPL) components and how Xberg stays compliant.

> Rust crate dependencies and their licenses are governed by `deny.toml`
> (`cargo deny check licenses`). This file covers the **system/native** libraries
> that are linked at the C ABI level and are not visible to `cargo deny`.

## libheif (HEIF / HEIC / AVIF decoding) — LGPL

- **Feature:** optional `heic` Cargo feature (part of `full`/`formats`). Disabled
  by default. The standalone CLI release binaries (`xberg-cli-*.tar.gz`) are
  built **without** `heic`, so they do not link `libheif` at all.
- **License:** GNU Lesser General Public License (LGPL). See the upstream
  [`COPYING`](https://github.com/strukturag/libheif/blob/master/COPYING) for the
  authoritative version and text.
- **Linking:** **Dynamic only.** Xberg resolves `libheif` via `pkg-config`
  (`-lheif`) against the system shared library; it is never statically linked.
  The musl CLI container build explicitly disables `crt-static`
  (`RUSTFLAGS="-C target-feature=-crt-static"`) so the resulting binary loads
  `libheif.so` at runtime rather than embedding it. The static-build
  (`embedded-libheif`) feature has been **removed** from `xberg-libheif`, so
  there is no supported way to statically link `libheif` into a Xberg build.
- **Redistribution (container images):** the `full`/`core` images ship the
  unmodified upstream `libheif` (v1.23.0, built from the official release tarball)
  as a standalone `libheif.so*` in `/usr/local/lib`. Because it is a separate,
  dynamically-loaded shared object, you may replace it with your own build of
  `libheif` to satisfy LGPL §6 (the "replace the library" requirement). Upstream
  source: <https://github.com/strukturag/libheif> (release v1.23.0).

## libheif codec plugins (container images only)

`libheif` loads codec backends as separate dynamically-loaded plugin `.so`s. The
container images install these from the distro package manager (apt/apk); each
retains its own upstream license and is redistributed unmodified:

| Library  | Role            | License (upstream)          |
| -------- | --------------- | --------------------------- |
| libde265 | HEVC **decode** | LGPL-3.0-or-later           |
| libdav1d | AV1 **decode**  | BSD-2-Clause                |
| libaom   | AV1 dec/enc     | BSD-2-Clause + patent grant |
| libx265  | HEVC **encode** | **GPL-2.0-or-later**        |

All are loaded dynamically (separate `.so`, replaceable). **Note:** Xberg only
*decodes* HEIF/HEIC/AVIF, so the HEVC **encoder** `libx265` (GPL) is not required
for Xberg's functionality; it is pulled in only as a default `libheif` plugin
dependency and can be dropped from the images to avoid shipping a GPL component.

## ONNX Runtime (OCR / ML features)

- **Feature:** optional (`paddle-ocr`, `layout-detection`, `embeddings`,
  `reranker`, `auto-rotate`, transcription). License: MIT. Linked dynamically
  (system `libonnxruntime.so`) in the musl/container builds; bundled per the
  `ort-bundled` feature (official Microsoft binaries) otherwise.
