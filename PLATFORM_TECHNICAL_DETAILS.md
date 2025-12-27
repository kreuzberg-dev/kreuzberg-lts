# Platform Compatibility - Technical Details

## Architecture

### Platform Detection Strategy

The system uses a multi-layer platform detection approach:

1. **Primary**: Taskfile's `{{.OS}}` variable (derived from uname/environment)
2. **Secondary**: Environment variables (WINDIR, SYSTEMROOT, MSYSTEM)
3. **Tertiary**: PowerShell queries (Windows-specific)
4. **Fallback**: Sensible defaults (assume non-Windows if detection fails)

Located in: `.task/config/vars.yml` (lines 25-70)

### Variable Expansion

Key platform variables available throughout tasks:

```yaml
{{.OS}}              # Operating system: windows, linux, darwin
{{.ARCH}}            # Architecture: x86_64, arm64, x64, etc.
{{.EXE_EXT}}         # Binary extension: .exe (Windows), "" (Unix)
{{.LIB_EXT}}         # Library extension: dll, so, dylib
{{.NUM_CPUS}}        # CPU count for parallel builds
{{.VERSION}}         # Project version from Cargo.toml
{{.BUILD_PROFILE}}   # Build profile: dev, release, ci
```

## Platform-Specific Implementation Details

### Windows (.exe, PowerShell, .dll)

**Detection Methods**:
- Direct OS check: `{{.OS}}` equals "windows"
- Environment: `$WINDIR` or `$SYSTEMROOT` set
- Git Bash: `$MSYSTEM` variable present
- PowerShell query: Check Windows OS environment

**Path Handling**:
- Uses backslashes in batch commands: `.\vendor\bin\phpunit.bat`
- PowerShell paths with forward slashes: `crates/kreuzberg-wasm/dist`
- `PATH` separator: semicolon (`;`)
- Library paths in `PATH` variable

**Critical Fixes**:

1. **Binary Execution**:
   ```yaml
   - cmd: ./vendor/bin/phpunit          # Unix
     platforms: [linux, darwin]
   - cmd: .\vendor\bin\phpunit.bat      # Windows batch
     platforms: [windows]
   ```

2. **Version Extraction** (Previously: only `awk`):
   ```sh
   if command -v grep &>/dev/null && command -v sed &>/dev/null; then
     grep '^version = ' Cargo.toml | sed 's/version = "\([^"]*\)"/\1/'
   else
     powershell -Command "(Select-String -Path Cargo.toml -Pattern '^version = ' ...)"
   fi
   ```

3. **Directory Cleanup** (Previously: only `rm -rf`):
   ```yaml
   - cmd: bash -c "rm -rf crates/kreuzberg-wasm/dist ..."
     platforms: [linux, darwin]
   - cmd: powershell -Command "Remove-Item -Path @(...) -Recurse ..."
     platforms: [windows]
   ```

4. **CPU Detection** (Previously: only `nproc`):
   - Primary: `[System.Environment]::ProcessorCount` (PowerShell)
   - Fallback: `NUMBER_OF_PROCESSORS` environment variable
   - Fallback: Return hardcoded 4

### Linux (.so, ELF, apt-get)

**Detection Methods**:
- uname returns "Linux"
- Presence of `/proc/cpuinfo`
- `apt-get` or `yum` available for package management

**Path Handling**:
- LD_LIBRARY_PATH for runtime libraries
- Colon (`:`) as PATH separator
- Standard FHS paths: /usr/local/lib, /usr/lib, etc.

**Key Characteristics**:
- `uname -m` returns x86_64, arm64, aarch64, etc.
- CPU count via `nproc` command
- Library extension: `.so` (Shared Object)

**No Changes Needed**: Linux support is comprehensive and working correctly.

### macOS (.dylib, Mach-O, brew)

**Detection Methods**:
- uname returns "Darwin"
- Presence of /usr/bin/sw_vers
- sysctl command availability

**Path Handling**:
- DYLD_LIBRARY_PATH for runtime libraries
- DYLD_FALLBACK_LIBRARY_PATH as fallback search path
- Colon (`:`) as PATH separator
- Homebrew paths: /usr/local/opt, /opt/homebrew

**Key Characteristics**:
- `uname -m` returns x86_64 (Intel) or arm64 (M1/M2)
- CPU count via `sysctl -n hw.ncpu`
- Library extension: `.dylib` (Dynamic Library)
- Universal binary support via Rust target triples

**Specific Considerations**:

1. **CPU Detection** (macOS-specific):
   ```sh
   if [ "$os" = "darwin" ]; then
     sysctl -n hw.ncpu 2>/dev/null || echo 4
   fi
   ```

2. **Library Paths for Go**:
   ```sh
   export DYLD_LIBRARY_PATH="${repo_root}/target/release:${DYLD_LIBRARY_PATH:-}"
   export DYLD_FALLBACK_LIBRARY_PATH="${repo_root}/target/release:..."
   ```

3. **M1/M2 Support**:
   - arch64 detection via `uname -m`
   - Target triple system via Rust: `aarch64-apple-darwin`
   - No additional work needed - handled by build tools

## Language-Specific Implementations

### Rust (All Platforms ✓)
- `cargo` handles cross-platform compilation
- FFI library built to `target/release/` (universal)
- Already working on all three platforms

### Python (All Platforms ✓)
- `maturin` handles platform-specific compilation
- `uv` is cross-platform
- Clean tasks use Python's cross-platform `shutil`

### Node.js (All Platforms ✓)
- `pnpm` is cross-platform
- `napi` handles platform-specific builds
- Clean tasks use Python script

### Go (All Platforms with custom handling ✓)
- CGO requires platform-specific setup
- Windows: Uses `.dll` (via MSVC or MinGW)
- Linux: Uses `.so` (via gcc/clang)
- macOS: Uses `.dylib` (via clang)
- Path handling: `library-paths.sh` sets up correctly per platform

### C# (All Platforms ✓)
- `dotnet` is cross-platform
- Native libraries handled by NuGet system
- Windows: Direct `dotnet test` execution (no bash needed)

### PHP (Mostly Windows ✓)
- `composer` is cross-platform
- Extension building platform-specific (bash scripts on Unix)
- E2E tests: Unix uses `./vendor/bin/phpunit`, Windows uses `.bat`

### Ruby (Unix Only)
- Requires compilation tools
- Extension building via Rake
- Windows support not critical for this project

### Java (All Platforms ✓)
- `maven` is cross-platform
- Native libraries handled via Maven plugins
- Works on Windows via direct Maven commands

## Critical Paths and Fallback Chains

### VERSION Extraction Path

```
Unix Available:
  1. grep '^version = ' Cargo.toml
  2. sed 's/version = "\([^"]*\)"/\1/'

Fallback (Windows):
  1. PowerShell Select-String
  2. Regex matching
  3. Return "0.0.0" if all fail
```

### ARCH Detection Path

```
Unix Available:
  1. uname -m (returns x86_64, arm64, aarch64, etc.)

Fallback (Windows):
  1. PowerShell [System.Environment]::GetEnvironmentVariable
  2. Return "unknown" if fails
```

### CPU Detection Path

```
Windows:
  1. PowerShell [System.Environment]::ProcessorCount
  2. NUMBER_OF_PROCESSORS env var

macOS:
  1. sysctl -n hw.ncpu

Linux:
  1. nproc
  2. grep -c '^processor' /proc/cpuinfo

Fallback:
  1. sysctl -n hw.ncpu (BSD/macOS)
  2. Return 4 (hardcoded fallback)
```

## Library Path Configuration

### Linux
```bash
export LD_LIBRARY_PATH="${pdfium_lib}/lib:${LD_LIBRARY_PATH:-}"
export LD_LIBRARY_PATH="${ort_lib}:${LD_LIBRARY_PATH:-}"
export LD_LIBRARY_PATH="${repo_root}/target/release:${LD_LIBRARY_PATH:-}"
```

### macOS
```bash
export DYLD_LIBRARY_PATH="${pdfium_lib}/lib:${DYLD_LIBRARY_PATH:-}"
export DYLD_FALLBACK_LIBRARY_PATH="${pdfium_lib}/lib:${DYLD_FALLBACK_LIBRARY_PATH:-}"
export DYLD_LIBRARY_PATH="${repo_root}/target/release:${DYLD_LIBRARY_PATH:-}"
export DYLD_FALLBACK_LIBRARY_PATH="${repo_root}/target/release:${DYLD_FALLBACK_LIBRARY_PATH:-}"
```

### Windows
```bash
export PATH="${pdfium_lib}/bin;${PATH:-}"
export PATH="${ort_lib};${PATH:-}"
export PATH="${repo_root}/target/release;${PATH:-}"
```

**Key Insight**: Windows doesn't use separate library path variables - libraries go in PATH or current directory.

## RPATH Handling

### Design Principle
RPATH (Runtime Path) is NOT hardcoded to development paths to ensure portability.

### Implementation
```sh
# Development (Unix only - Windows doesn't support rpath)
export CGO_LDFLAGS="-L${repo_root}/target/release -lkreuzberg_ffi -Wl,-rpath,${repo_root}/target/release"

# Production: Uses system library paths or explicit LD_LIBRARY_PATH
```

**Why**: Hardcoded dev paths would break when code is deployed or built on different machines.

## Error Handling Strategies

### 1. Graceful Degradation
```yaml
- cmd: optional_tool
  ignore_error: true
```
Used for: actionlint, optional linters

### 2. Platform-Specific Messaging
```yaml
- cmd: echo "Operation not supported on Windows - use WSL or CI"
  platforms: [windows]
```
Used for: E2E generation scripts, shell linting

### 3. Fallback Chains
```sh
# Try Unix tools first
if command -v grep &>/dev/null; then
  # Use Unix approach
else
  # Fallback to PowerShell/Windows approach
fi
```

## Shell Script Compatibility

### Files Running on Windows
- Must use `bash` explicitly: `bash scripts/toml_format.sh`
- Or provide native alternative: `powershell -Command ...`

### Files Unix-Only
- Shell linting (shfmt, shellcheck not available on Windows)
- E2E generation (complex bash logic)
- Message shown on Windows: "Not available - use WSL or CI"

## Build Tool Compatibility

| Tool | Windows | Linux | macOS | Notes |
|------|---------|-------|-------|-------|
| cargo | ✓ | ✓ | ✓ | Native support |
| python | ✓ | ✓ | ✓ | Via Python.org or managers |
| pnpm | ✓ | ✓ | ✓ | Native support |
| dotnet | ✓ | ✓ | ✓ | Native support |
| mvn | ✓ | ✓ | ✓ | Requires Java |
| composer | ✓ | ✓ | ✓ | Requires PHP |
| bundle | ✓ | ✓ | ✓ | Requires Ruby |
| go | ✓ | ✓ | ✓ | CGO needs platform setup |
| maturin | ✓ | ✓ | ✓ | Python Rust bridge |
| napi | ✓ | ✓ | ✓ | Node Rust bridge |

## Testing Strategies

### Unit Test Compatibility
- `cargo test` - Works everywhere
- `pytest` - Works everywhere
- `jest/vitest` - Works everywhere
- `dotnet test` - Works everywhere
- `mvn test` - Works everywhere

### E2E Test Compatibility
- Most E2E tests require Unix shell scripts (bash)
- Run in CI or WSL on Windows
- Helpful messages guide users to alternatives

## Performance Considerations

### CPU-Aware Builds
```yaml
NUM_CPUS: {{.NUM_CPUS}}
```
Used in parallel build configurations to optimize for available cores.

### Platform-Specific Optimizations
- Windows: Static linking for C++ libraries (no runtime dependencies)
- Linux: Dynamic linking (standard practice)
- macOS: Universal binaries for broad compatibility

## Future Enhancements

1. **Add PowerShell parameter validation** for Windows-specific tasks
2. **Create parallel CI jobs** for Windows native builds
3. **Add M1/M2 specific testing** in CI pipeline
4. **Containerize Windows builds** if needed for consistency

## References

- Taskfile Documentation: https://taskfile.dev
- Go CGO Cross-Compilation: https://github.com/golang/go/wiki/WindowsCrossCompiling
- Rust Platform-Specific Code: https://doc.rust-lang.org/reference/conditional-compilation.html
- macOS DYLD Documentation: https://developer.apple.com/documentation/macos-release-notes
