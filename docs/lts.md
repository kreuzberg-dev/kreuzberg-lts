---
description: "Kreuzberg v4 LTS support policy, backwards-compatibility guarantees, and how to migrate to Xberg (v5+)."
---

# Kreuzberg v4 LTS

!!! warning "This is the legacy v4 line"

    Active development has moved to **[Xberg](https://github.com/xberg-io/xberg)** (v5 and later).
    This repository — [`kreuzberg-dev/kreuzberg-lts`](https://github.com/kreuzberg-dev/kreuzberg-lts) —
    is the long-term-support home for the Kreuzberg **v4** line. New projects should start on Xberg.

## Support policy

Kreuzberg v4 receives **LTS fixes until the end of 2026, on a best-effort basis**. LTS scope is:

- Critical bug fixes and security patches.
- Compatibility fixes for supported runtimes and platforms.
- **No new features** — feature work lands in [Xberg](https://github.com/xberg-io/xberg).

After 2026, the v4 line is considered end-of-life. Released versions remain available on their
registries indefinitely.

## License

Kreuzberg v4 LTS is licensed under the **MIT License** (earlier v4 releases shipped under the
Elastic License 2.0; the LTS line is MIT going forward). See [`LICENSE`](https://github.com/kreuzberg-dev/kreuzberg-lts/blob/main/LICENSE).

## Language bindings

v4 LTS keeps all v4 bindings, including the **R binding** — v4 is the **last line to ship R support**;
it is removed in Xberg (v5). If you depend on the R binding, stay on v4 LTS.

## Installing v4

The v4 packages keep their existing names and install paths:

| Ecosystem | Install |
|-----------|---------|
| Python (PyPI) | `pip install "kreuzberg>=4,<5"` |
| npm | `npm install @kreuzberg/kreuzberg@^4` |
| Go | `go get github.com/kreuzberg-dev/kreuzberg-lts/v4` |
| Docker | `docker pull ghcr.io/kreuzberg-dev/kreuzberg-full:4` |
| Helm | `helm install kreuzberg oci://ghcr.io/kreuzberg-dev/charts/kreuzberg --version 4.10.0` |

### A note for Go users

The Go module path is now **`github.com/kreuzberg-dev/kreuzberg-lts/v4`**. Existing pins to the old
`github.com/kreuzberg-dev/kreuzberg` path keep resolving from the Go module proxy cache, but **new v4
LTS releases publish under the new path** — update your import to receive them.

## Migrating to Xberg (v5)

New features, performance work, and long-term maintenance happen in Xberg. To move off v4:

- Read the [Xberg migration guide](https://github.com/xberg-io/xberg#migration).
- Xberg's Go module is `github.com/xberg-io/xberg`.
- The R binding is not available in v5 — remain on v4 LTS if you need it.

If you are not ready to migrate, staying on v4 LTS is fully supported through the window above.
