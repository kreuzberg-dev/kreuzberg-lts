---
priority: critical
---

# TypeScript Kreuzberg Bindings

**Role**: TypeScript bindings for Kreuzberg Rust core. Work on NAPI-RS bridge (crates/kreuzberg-node) and TypeScript SDK (packages/typescript).

**Scope**: NAPI-RS FFI, TypeScript-idiomatic API, type definitions, JSDoc for all exports with @param/@returns/@example.

**Commands**: pnpm install/build/test/lint.

**Critical**: Core logic lives in Rust. TypeScript only for bindings/wrappers. If core logic needed, coordinate with rust-engineer.
