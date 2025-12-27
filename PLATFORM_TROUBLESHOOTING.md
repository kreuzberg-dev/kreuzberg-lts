# Platform Compatibility - Troubleshooting Guide

## Windows Troubleshooting

### Issue: Task Fails with "Command not found"

**Symptom**: Error like `bash: ./scripts/toml_format.sh: No such file or directory`

**Cause**: Bash script called without explicit `bash` prefix on Windows

**Solution**: All scripts should be called explicitly with `bash`:
```yaml
- cmd: bash scripts/toml_format.sh
  platforms: [windows]
```

**Status**: Already fixed in this release

---

### Issue: VERSION Variable Returns Empty or "unknown"

**Symptom**: Build fails with empty version number

**Cause**: Grep/sed not available and PowerShell fallback failed

**Solution**:
1. Ensure PowerShell is available: `powershell -NoProfile -Command "Write-Host 'OK'"`
2. Verify Cargo.toml exists in root directory
3. Check for special characters in version string

**Fallback**:
```powershell
# Manual version check
(Select-String -Path Cargo.toml -Pattern '^version = ' | Select-Object -First 1).Line
```

**Status**: Already fixed with proper fallback chain

---

### Issue: Architecture Detection Shows "unknown"

**Symptom**: Build system shows ARCH=unknown

**Cause**: Environment variable not set, PowerShell query failed

**Solution**:
1. Verify environment: `echo %PROCESSOR_ARCHITECTURE%`
2. Check if running in proper shell (not restricted)
3. Manually set if needed: `set PROCESSOR_ARCHITECTURE=AMD64` or `ARM64`

**Valid Values**:
- `AMD64` (standard x86-64)
- `ARM64` (Windows on ARM devices)
- `x86` (32-bit - rare)

**Status**: Already fixed with environment variable fallback

---

### Issue: "C#/.NET tests fail with path issues"

**Symptom**: `dotnet test` fails with path-related errors

**Cause**: Mixed path separators or relative path issues

**Solution**:
1. Run from correct directory: `cd packages/csharp`
2. Use absolute paths if needed
3. Verify NuGet packages restored: `dotnet restore`

**Proper Command**:
```powershell
cd packages/csharp
dotnet test Kreuzberg.Tests/Kreuzberg.Tests.csproj -c Release
```

**Status**: Already fixed with platform-specific commands

---

### Issue: "WASM clean fails with permission denied"

**Symptom**: PowerShell error: `Access to the path is denied`

**Cause**: Files locked by other process

**Solution**:
1. Close any open terminals/editors in those directories
2. Verify antivirus isn't blocking: exclude project directory
3. Try manual cleanup:
```powershell
Remove-Item -Path 'crates/kreuzberg-wasm/dist' -Recurse -Force
Remove-Item -Path 'crates/kreuzberg-wasm/pkg' -Recurse -Force
```

**Status**: Already handles with `ignore_error` and `-ErrorAction SilentlyContinue`

---

### Issue: "PowerShell execution policy blocks scripts"

**Symptom**: `PowerShell : File cannot be loaded because running scripts is disabled`

**Cause**: Execution policy set to Restricted

**Solution**:
```powershell
# Check current policy
Get-ExecutionPolicy -Scope CurrentUser

# Set to RemoteSigned (safe default)
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

# Or use -NoProfile flag (already done in our code)
powershell -NoProfile -Command "..."
```

**Status**: All our PowerShell calls use `-NoProfile` to avoid policy issues

---

### Issue: "Library not found for linking" (Go builds)

**Symptom**: Go build fails: `cannot find kreuzberg_ffi`

**Cause**: LD_LIBRARY_PATH not set correctly on Windows

**Solution**:
1. Verify Rust FFI is built: `cargo build --release --package kreuzberg-ffi`
2. Check PATH includes `target/release`: `echo %PATH%`
3. Run setup-go-cgo-env action (in CI) or set manually:
```powershell
$env:CGO_LDFLAGS="-Ltarget\x86_64-pc-windows-gnu\release -lkreuzberg_ffi -static-libgcc"
```

**Status**: Handled by `scripts/lib/library-paths.sh`

---

## Linux Troubleshooting

### Issue: "LD_LIBRARY_PATH not working"

**Symptom**: Linker error: `libkreuzberg_ffi.so: cannot open shared object file`

**Cause**: Library path not set or set incorrectly

**Solution**:
```bash
# Check current path
echo $LD_LIBRARY_PATH

# Set manually for testing
export LD_LIBRARY_PATH=$PWD/target/release:$LD_LIBRARY_PATH

# Verify library exists
ls -la target/release/libkreuzberg_ffi.so
```

**Status**: Properly configured in `scripts/lib/library-paths.sh`

---

### Issue: "APT package manager not available"

**Symptom**: `apt-get: command not found`

**Cause**: Running on non-Debian system (CentOS, Fedora, etc.)

**Solution**:
1. Check your Linux distribution: `cat /etc/os-release`
2. Use appropriate package manager:
   - **Fedora/RHEL**: `dnf install`, `yum install`
   - **Arch**: `pacman -S`
   - **Alpine**: `apk add`

**Status**: Not an issue - package installations isolated to CI scripts

---

### Issue: "CPU count detection fails"

**Symptom**: NUM_CPUS shows incorrect value or "4" (fallback)

**Cause**: `nproc` not available or `/proc/cpuinfo` not readable

**Solution**:
```bash
# Check available tools
which nproc
cat /proc/cpuinfo | grep -c '^processor'
sysctl -n hw.ncpu

# Or set manually
export NUM_CPUS=8
```

**Status**: Multiple fallback methods already implemented

---

## macOS Troubleshooting

### Issue: "Library not found for libkreuzberg_ffi"

**Symptom**: Linker error for dylib

**Cause**: DYLD_LIBRARY_PATH not set correctly

**Solution**:
```bash
# Check current path
echo $DYLD_LIBRARY_PATH

# Set manually for testing
export DYLD_LIBRARY_PATH=$PWD/target/release:$DYLD_LIBRARY_PATH
export DYLD_FALLBACK_LIBRARY_PATH=$PWD/target/release:$DYLD_FALLBACK_LIBRARY_PATH

# Verify library exists
ls -la target/release/libkreuzberg_ffi.dylib
```

**Status**: Both paths configured in `scripts/lib/library-paths.sh`

---

### Issue: "M1/M2 (ARM64) build fails"

**Symptom**: `error[E0514]: found crate compiled for x86_64`

**Cause**: Building for wrong architecture or architecture mismatch

**Solution**:
```bash
# Verify your architecture
uname -m  # Returns: arm64

# Clean and rebuild for your architecture
cargo clean
cargo build --release

# If cross-compiling to x86_64:
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin
```

**Status**: Rust/cargo handles this automatically

---

### Issue: "CPU detection returns wrong count"

**Symptom**: `sysctl -n hw.ncpu` returns incorrect value

**Cause**: Virtualization or restricted CPU access

**Solution**:
```bash
# Check actual count
sysctl hw.physicalcpu
sysctl hw.logicalcpu

# Or check system settings
system_profiler SPHardwareDataType | grep Cores
```

**Status**: `sysctl` fallback already implemented in `platforms.yml`

---

## Cross-Platform Issues

### Issue: "Platform detection shows wrong OS"

**Symptom**: `{{.OS}}` detects Windows when running on Linux under WSL

**Cause**: WSL environment mimics Windows but runs Linux kernel

**Solution**:
```bash
# Check what Taskfile sees
echo $OS            # May show "Windows_NT" in WSL
uname -s            # Shows "Linux"

# Override if needed
export GOOS=linux

# Or use WSL2 (better isolation)
```

**Status**: Multiple detection methods with proper fallbacks

---

### Issue: "Path separators causing issues"

**Symptom**: Path works on one OS but not another

**Cause**: Hardcoded path separators

**Solution**:
- Use forward slashes in Taskfile (cross-platform): `crates/kreuzberg-wasm/dist`
- Use backslashes only in batch/PowerShell: `.\vendor\bin\phpunit.bat`
- Let tools handle separator conversion

**Status**: Properly handled in all our configurations

---

### Issue: "Different behavior on different machines"

**Symptom**: Works on one person's Windows but not another's

**Cause**: Different environment setup, missing tools, or PowerShell version

**Solution**:
1. Run setup task: `task setup`
2. Verify all build tools installed:
   ```bash
   rustc --version
   python --version
   node --version
   dotnet --version
   ```
3. Check PowerShell version (Windows): `$PSVersionTable.PSVersion`
4. Ensure PATH is correctly set

**Status**: Setup task improved with better error messages

---

## Diagnostic Commands

### Universal Diagnostics

```bash
# Show detected platform
echo "OS: {{.OS}}"
echo "Architecture: {{.ARCH}}"
echo "Executable ext: {{.EXE_EXT}}"
echo "Library ext: {{.LIB_EXT}}"
echo "CPU count: {{.NUM_CPUS}}"

# Check git
git --version

# Check Taskfile
task --version
```

### Windows-Specific Diagnostics

```powershell
# Environment
Write-Host "OS: $env:OS"
Write-Host "SYSTEMROOT: $env:SYSTEMROOT"
Write-Host "PATH: $env:PATH"

# PowerShell version
$PSVersionTable.PSVersion

# Processor info
[System.Environment]::ProcessorCount

# Network and execution
whoami
Get-ExecutionPolicy
```

### Linux-Specific Diagnostics

```bash
# System info
uname -a
lsb_release -a

# Available tools
which grep sed awk nproc

# CPU info
nproc
cat /proc/cpuinfo | grep -c "^processor"

# Library paths
echo $LD_LIBRARY_PATH
ldconfig -p
```

### macOS-Specific Diagnostics

```bash
# System info
uname -a
sw_vers

# CPU info
sysctl -n hw.ncpu
sysctl -n hw.physicalcpu

# Architecture
uname -m

# Library paths
echo $DYLD_LIBRARY_PATH
echo $DYLD_FALLBACK_LIBRARY_PATH
```

---

## Getting Help

### Before Reporting an Issue

1. **Run diagnostics**: See section above
2. **Check platform compatibility**:
   - Windows: Are you using WSL or native Windows?
   - macOS: Are you on Intel or M1/M2?
   - Linux: What distribution? (Ubuntu, Fedora, etc.)

3. **Verify prerequisites**:
   ```bash
   # All platforms
   task --version
   rustc --version
   cargo --version

   # If using Python
   python --version

   # If using Node
   node --version
   npm --version
   ```

4. **Check Taskfile logs**:
   ```bash
   task -v build  # Verbose output
   ```

### When Reporting

Include:
1. Operating system and version
2. Architecture (x86_64, arm64, etc.)
3. Output of `task --version`
4. Full error message with context
5. Which task failed: `task build`, `task test`, etc.

---

## Known Limitations

1. **E2E test generation** requires bash (use WSL on Windows)
2. **Shell linting** requires Unix tools (skipped on Windows)
3. **SDKman** is Unix-only (manual Java setup on Windows)
4. **Ruby extension building** may require additional Windows setup

---

## Quick Fix Summary

| Issue | Quick Fix |
|-------|-----------|
| PowerShell won't run | Use `-NoProfile` flag |
| Version/ARCH returns empty | Verify Cargo.toml, check PowerShell |
| LD_LIBRARY_PATH not working | Set manually: `export LD_LIBRARY_PATH=$PWD/target/release` |
| DYLD paths on macOS | Set both: `DYLD_LIBRARY_PATH` and `DYLD_FALLBACK_LIBRARY_PATH` |
| Windows paths fail | Use forward slashes or backslashes appropriately |
| C# tests fail | Run from `packages/csharp` directory |
| CPU count wrong | Check `sysctl` or `nproc` output |
| Architecture detection fails | Set `PROCESSOR_ARCHITECTURE` or `GOOS` env var |

---

**Last Updated**: 2025-12-27
**Platform Support**: Windows 10+, Linux (most distributions), macOS 10.15+
**Test Status**: All platforms verified compatible
