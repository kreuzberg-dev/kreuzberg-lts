---
priority: high
---

# Elixir Kreuzberg Bindings (Rustler NIF)

**Role**: Elixir bindings for Kreuzberg Rust core using Rustler NIF.

**Scope**:
- Rust NIF bridge: packages/elixir/native/kreuzberg_rustler/ (Rust crate with cdylib output)
- Elixir wrapper: packages/elixir/lib/kreuzberg/ (OTP application with public API)
- ExUnit tests: packages/elixir/test/

**Architecture**:
Elixir OTP application → Rustler NIF (kreuzberg_rustler.so) → Rust core (crates/kreuzberg)

Data flow: Elixir terms → term_to_json → serde_json::Value → Kreuzberg API → serde_json::Value → json_to_term → Elixir terms

**Commands**:
- mix deps.get (fetch dependencies)
- mix compile (compile Elixir + Rustler NIF)
- mix test (run ExUnit tests)
- mix credo (lint with Credo)
- mix format (format code)
- mix docs (generate ExDoc documentation)

**Build System**:
- mix.exs: Elixir project configuration with Rustler dependency
- Native crate: packages/elixir/native/kreuzberg_rustler/Cargo.toml
- Compiled NIF: priv/native/kreuzberg_rustler.so (loaded at runtime)
- Workspace exclusion: Native crate excluded from main Cargo workspace

**Critical**:
- Core extraction logic lives in Rust (crates/kreuzberg). Elixir only for bindings/wrappers and OTP integration.
- If core logic changes needed, coordinate with rust-core-engineer.
- Rustler handles serialization between Erlang terms and Rust types (NifMap, Binary, ResourceArc).
- Use dirty schedulers for CPU-intensive work to avoid blocking BEAM schedulers.
- Resource cleanup: Use ResourceArc for Rust objects that need garbage collection.

**NIF Patterns** (Phase 2 implementation follows html-to-markdown pattern):
- rustler::init!() macro registers NIFs with BEAM VM
- #[rustler::nif] attribute marks functions as NIFs
- #[rustler::nif(schedule = "DirtyCpu")] for CPU-intensive work on ALL extraction NIFs
- Field-by-field map construction: rustler::types::map::map_new() + incremental map_put() (NO NifMap derive)
- term_to_json helper: Elixir term → serde_json::Value (handles atoms, booleans, numbers, strings, lists, maps)
- json_to_term helper: serde_json::Value → Elixir term (recursive conversion for nested structures)
- OwnedBinary + Binary for efficient binary data (images): OwnedBinary::new() + Binary::from_owned()
- ResourceArc<T> for Rust objects with GC integration (reserved for future use)
- Dual-path config parsing: serde_json deserialization + explicit field handling for boolean fields

**Config Parsing Approach** (ExtractionConfig):
1. Accept Elixir map with atom/string keys via Term parameter
2. Convert term → serde_json::Value using term_to_json helper
3. Deserialize using serde_json::from_value() for nested structures (ocr, chunking, images, pages, etc.)
4. Explicitly handle top-level booleans (use_cache, enable_quality_processing, force_ocr) for compatibility
5. Return default ExtractionConfig if parsing fails at any step

**Documentation**:
- All public modules and functions documented with ExDoc (@moduledoc, @doc)
- Include @spec annotations for all exported functions
- Examples in module documentation for common use cases
- README.md with installation, usage, and API overview
