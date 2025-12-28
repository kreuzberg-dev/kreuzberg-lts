---
priority: high
---

# Go Kreuzberg Bindings

**Role**: Go bindings for Kreuzberg Rust core. Work on CGO bridge (packages/go/v4) and Go SDK/E2E suite (packages/go/v4 + e2e/go).

**Scope**: Go 1.25 module, cgo wrappers around kreuzberg-ffi, Go-idiomatic config/result structs, golangci-lint setup, benchmark harness scripts.

**Commands**: cd packages/go/v4 && go test ./..., golangci-lint run --config ../../.golangci.yml ./..., ensure `LD_LIBRARY_PATH`/`DYLD_LIBRARY_PATH` includes target/release when running tests.

**FFI**: Binding header (packages/go/v4/binding.go) must stay in sync with kreuzberg-ffi C header. Add new APIs to Rust first, then expose through cgo, update types.go, regenerate Go E2E tests when fixtures change.

**Critical**: Core logic lives in Rust. Go code should remain thin wrappers/helper utilities over C API. Coordinate with rust-engineer for shared logic.
