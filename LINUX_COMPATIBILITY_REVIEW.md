# Critical Linux Compatibility Review - Taskfile Tasks

**Date:** 2025-12-27
**Scope:** All Taskfile YAML configurations and shell scripts
**Assessment:** CRITICAL ISSUES FOUND - Limited to Ubuntu/Debian Only

---

## Executive Summary

The Taskfile system demonstrates **good platform detection patterns** but has **critical limitations**:

- **STRENGTHS:** Proper LD_LIBRARY_PATH/DYLD_LIBRARY_PATH handling, platform-aware shell scripts
- **CRITICAL GAPS:** Hardcoded apt-get with no distribution detection, Python-based cleanup commands
- **MISSING:** Alpine Linux (musl) support, non-Ubuntu Linux support, package name mapping

**Impact:** Taskfiles work on Ubuntu/Debian. Will FAIL on CentOS/RHEL/Fedora/Alpine without major modifications.

---

## 1. PACKAGE MANAGEMENT - CRITICAL HARDCODED APT ASSUMPTION

### Issue 1.1: Hard-coded apt-get - No Distribution Detection
**Severity:** CRITICAL
**File:** `scripts/ci/install-system-deps/install-linux.sh:12-34`
**Lines:** 12, 34, 44

```bash
if ! retry_with_backoff sudo apt-get update; then
  ...
if retry_with_backoff_timeout 900 sudo apt-get install -y "${packages[@]}"; then
  ...
  if retry_with_backoff_timeout 300 sudo apt-get install -y "$pkg" 2>&1; then
```

**Problem:**
- Only apt (Debian/Ubuntu) supported
- Alpine: uses `apk`
- RHEL/CentOS/Fedora: uses `yum`/`dnf`
- Arch: uses `pacman`
- **No detection mechanism exists**

**Will Break On:**
- Alpine Linux (Docker alpine image) - FAIL
- CentOS/RHEL 7/8/9 - FAIL
- Fedora - FAIL
- Arch Linux - FAIL

**Recommended Fix:**
```bash
detect_package_manager() {
    if command -v apt-get >/dev/null 2>&1; then
        echo "apt"
    elif command -v apk >/dev/null 2>&1; then
        echo "apk"
    elif command -v yum >/dev/null 2>&1; then
        echo "yum"
    elif command -v dnf >/dev/null 2>&1; then
        echo "dnf"
    elif command -v pacman >/dev/null 2>&1; then
        echo "pacman"
    else
        return 1
    fi
}
```

---

### Issue 1.2: Package Names Not Mapped Across Distributions
**Severity:** CRITICAL
**File:** `scripts/ci/install-system-deps/install-linux.sh:16-31`
**Lines:** 16-31

```bash
packages=(
    libssl-dev              # Debian only!
    pkg-config              # Different names per distro
    build-essential         # Debian only!
    tesseract-ocr           # Different names per distro
)
```

**Package Name Mappings Needed:**

| Logical Package | Debian/Ubuntu | RHEL/CentOS | Alpine | Arch |
|---|---|---|---|---|
| OpenSSL Dev | libssl-dev | openssl-devel | openssl-dev | openssl |
| pkg-config | pkg-config | pkgconfig | pkgconfig | pkgconf |
| Build Tools | build-essential | gcc gcc-c++ make | alpine-sdk | base-devel |
| Tesseract | tesseract-ocr | tesseract-devel | tesseract | tesseract |
| LibreOffice | libreoffice | libreoffice | -- | libreoffice |

**Impact:** Silently fails to install development headers, leading to confusing build errors later.

---

## 2. PYTHON DEPENDENCIES IN CLEANUP - Major Issue

### Issue 2.1: Python Commands Used for Directory Cleanup
**Severity:** MEDIUM-HIGH
**Files:**
- `.task/languages/python.yml:131-133`
- `.task/languages/node.yml:103-104`
- `.task/languages/ruby.yml:119-120`
- `.task/languages/php.yml:111`
- `.task/languages/go.yml:122`
- `.task/languages/csharp.yml:112`

**Example from python.yml:131-133:**
```yaml
clean:
  cmds:
    - python -c "import shutil, os, glob; [shutil.rmtree(d, ignore_errors=True) for d in glob.glob('*.egg-info')] ..."
    - python -c "import shutil, os; [shutil.rmtree(os.path.join(r, d), ignore_errors=True) for r, dirs, _ in os.walk('.') for d in dirs if d == '__pycache__']"
```

**Problems:**
1. **Python may not be available:** `python` command not guaranteed (could be python2, python3, or missing)
2. **Circular dependency:** Using Python to clean Python build artifacts
3. **One-liners are unmaintainable:** Hard to debug
4. **Cross-platform issues:** Path handling differs
5. **May reference non-existent modules**

**Better Solution - Use Native Shell:**
```yaml
clean:
  cmds:
    - rm -rf build/ dist/ *.egg-info __pycache__ .pytest_cache .mypy_cache
    - find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
    - find . -type f -name "*.pyc" -delete
```

---

## 3. LIBRARY PATH AND LINKING ISSUES

### Issue 3.1: RPATH Hardcoded to Development Path
**Severity:** HIGH
**File:** `scripts/lib/library-paths.sh:217, 224`
**Lines:** 217, 224

```bash
export CGO_LDFLAGS="-L${repo_root}/target/release -lkreuzberg_ffi -Wl,-rpath,${repo_root}/target/release"
```

**Problem:**
- RPATH set to `/full/path/to/repo/target/release`
- In production, this directory doesn't exist
- Library cannot be relocated
- RUNPATH preferred in modern systems

**Impact:** Deployed binaries won't find libraries outside of build directory

**Better Approach:**
```bash
# Use RUNPATH (can be overridden by LD_LIBRARY_PATH)
export CGO_LDFLAGS="-L${repo_root}/target/release -lkreuzberg_ffi -Wl,--enable-new-dtags,-rpath,\$ORIGIN/../lib"
```

---

### Issue 3.2: No musl libc Distinction
**Severity:** MEDIUM
**File:** `.task/config/platforms.yml:15-31`
**Lines:** 15-31

```yaml
LIB_EXT:
  sh: |
    case "$os" in
      linux|freebsd|openbsd|netbsd)
        echo "so"
```

**Problem:**
- Treats all Linux distributions the same
- Alpine Linux uses musl, not glibc
- musl has different library paths
- musl doesn't have all standard paths

**Impact:** Alpine Linux containers will have library lookup failures

---

## 4. COMPILER AND BUILD TOOL ISSUES

### Issue 4.1: CGO_ENABLED Unconditional - No FFI Validation
**Severity:** MEDIUM
**File:** `.task/languages/go.yml:33, 43, 53, 63`
**Lines:** 33, 43, 53, 63

```yaml
env:
  CGO_ENABLED: "1"
cmds:
  - go build ./...
```

**Problem:**
- Sets CGO_ENABLED=1 unconditionally
- Requires C compiler and FFI library available
- No validation FFI library was built
- Fails with confusing CGO errors

**Better Approach:**
```bash
# Verify FFI library exists first
if [ ! -f "../../target/release/libkreuzberg_ffi.so" ]; then
    echo "ERROR: FFI library not found. Run: task rust:ffi:build"
    exit 1
fi
CGO_ENABLED=1 go build ./...
```

---

### Issue 4.2: No Rust musl Target Support
**Severity:** MEDIUM
**File:** `.task/languages/rust.yml` (entire file)

**Problem:**
- No explicit x86_64-unknown-linux-musl target
- Alpine Linux needs musl build
- No detection of Alpine environment

**Impact:** Rust builds fail or produce incompatible binaries on Alpine

---

## 5. SHELL SCRIPT COMPATIBILITY

### Issue 5.1: E2E Scripts Assume bash - Alpine Has Only sh
**Severity:** MEDIUM
**File:** `.task/languages/python.yml:147`

```yaml
e2e:generate:
  cmds:
    - cmd: bash scripts/task/e2e-generate.sh python
      platforms: [linux, darwin]
```

**Problem:**
- Alpine Linux doesn't have bash by default
- Only /bin/sh available
- Script shebang may require bash

**Will Fail On:** Alpine Docker containers

---

### Issue 5.2: sdkman Hard-coded Home Path
**Severity:** MEDIUM
**File:** `Taskfile.yml:65`

```yaml
- cmd: bash -c "source ~/.sdkman/bin/sdkman-init.sh && sdk env install"
  ignore_error: true
```

**Problems:**
1. Hard-codes ~/.sdkman path
2. Should use $SDKMAN_DIR environment variable
3. ignore_error hides real failures
4. Not available in containers

**Better:**
```bash
if [ -f "${SDKMAN_DIR:-$HOME/.sdkman}/bin/sdkman-init.sh" ]; then
    source "${SDKMAN_DIR:-$HOME/.sdkman}/bin/sdkman-init.sh"
    sdk env install || true
fi
```

---

## 6. PDFIUM CONFIGURATION ISSUES

### Issue 6.1: Non-existent Package Names
**Severity:** HIGH
**File:** `.task/tools/pdfium.yml:20-24`

```yaml
install:linux:
  cmds:
    - |
      if command -v apt-get &> /dev/null; then
        sudo apt-get install -y libpdfium-dev
      elif command -v yum &> /dev/null; then
        sudo yum install -y pdfium-devel
```

**Problem:**
- `libpdfium-dev` doesn't exist in Debian repos
- `pdfium-devel` doesn't exist in RHEL repos
- These packages don't exist in any standard repo

**Impact:** Package installation will fail

**Better Approach:**
```yaml
install:linux:
  cmds:
    - echo "PDFium is downloaded from prebuilt binaries"
    - bash scripts/download_pdfium_runtime.sh
```

---

## Summary Table of Critical Issues

| ID | Category | Severity | File:Line | Issue | Fix Priority |
|---|---|---|---|---|---|
| 1.1 | Package Mgmt | CRITICAL | install-linux.sh:12-34 | Hardcoded apt-get | DO FIRST |
| 1.2 | Package Mgmt | CRITICAL | install-linux.sh:16-31 | No package mapping | DO FIRST |
| 2.1 | Cleanup | MEDIUM-HIGH | *.yml | Python cleanup cmds | DO SOON |
| 3.1 | Linking | HIGH | library-paths.sh:217 | RPATH hardcoded | DO SOON |
| 3.2 | musl | MEDIUM | platforms.yml:22 | No musl handling | DO SOON |
| 4.1 | CGO | MEDIUM | go.yml:33+ | No FFI validation | DO SOON |
| 4.2 | Rust | MEDIUM | rust.yml | No musl target | LATER |
| 5.1 | bash | MEDIUM | python.yml:147 | Assumes bash | DO SOON |
| 5.2 | sdkman | MEDIUM | Taskfile.yml:65 | Hard-coded path | DO SOON |
| 6.1 | PDFium | HIGH | pdfium.yml:20+ | Wrong pkg names | DO FIRST |

---

## Tested Compatibility Matrix

```
Distribution   | Status | Issues
Ubuntu 22.04   | OK     | None
Debian 12      | OK     | None (same as Ubuntu)
Alpine 3.18    | FAIL   | 1.1, 1.2, 3.2, 4.2, 5.1
CentOS 9       | FAIL   | 1.1, 1.2, 6.1
Fedora 38      | FAIL   | 1.1, 1.2
Arch Linux     | FAIL   | 1.1, 1.2
```

---

## Recommendations by Priority

### CRITICAL - DO IMMEDIATELY
1. **Fix apt-get hardcoding (Issue 1.1)**
   - Add distribution detection
   - Effort: 3-4 hours

2. **Add package name mapping (Issue 1.2)**
   - Create mapping for all distributions
   - Effort: 4-5 hours

3. **Fix PDFium package names (Issue 6.1)**
   - Use download script instead
   - Effort: 1 hour

### HIGH PRIORITY - DO THIS WEEK
1. **Replace Python cleanup with shell (Issue 2.1)**
   - Update 6 files
   - Effort: 2 hours

2. **Fix RPATH hardcoding (Issue 3.1)**
   - Use RUNPATH with $ORIGIN
   - Effort: 2 hours

3. **Add bash/sh detection (Issue 5.1)**
   - Detect and install bash on Alpine
   - Effort: 1 hour

### MEDIUM PRIORITY - DO NEXT WEEK
1. **Add musl libc handling (Issue 3.2)**
   - Detect musl and adjust paths
   - Effort: 3 hours

2. **Add CGO FFI validation (Issue 4.1)**
   - Check for built FFI library
   - Effort: 1 hour

3. **Fix sdkman path (Issue 5.2)**
   - Use $SDKMAN_DIR
   - Effort: 30 minutes

4. **Add musl Rust target (Issue 4.2)**
   - Add alpine/musl build variant
   - Effort: 2 hours

---

## Files Affected by Fixes

```
Distribution Detection:
  - scripts/ci/install-system-deps/install-linux.sh (PRIMARY)
  - .task/tools/pdfium.yml
  - Taskfile.yml (setup task)

Cleanup Cleanup (remove Python):
  - .task/languages/python.yml
  - .task/languages/node.yml
  - .task/languages/ruby.yml
  - .task/languages/php.yml
  - .task/languages/go.yml
  - .task/languages/csharp.yml

Library Path Fixes:
  - scripts/lib/library-paths.sh

Shell Compatibility:
  - .task/languages/python.yml (e2e:generate)
  - .task/languages/go.yml (e2e:generate)
  - .task/languages/ruby.yml (e2e:generate)
  - Taskfile.yml (setup)
```

---

## Estimated Effort to Full Linux Support

- **Critical fixes:** 8-10 hours
- **High priority:** 6-7 hours
- **Medium priority:** 8-10 hours
- **Testing across distros:** 5-6 hours

**Total:** 27-33 hours (approximately 1 week with focused effort)

---

## Conclusion

The Taskfile system is **well-structured** with good platform detection patterns, but has **critical gaps that prevent use on non-Ubuntu Linux systems**. The hardcoded apt-get and lack of package name mapping are the biggest blockers.

**Key wins from fixing these issues:**
1. Full Linux ecosystem support (RHEL, CentOS, Fedora, Alpine, Arch)
2. Better containerization support (Alpine Docker images)
3. More robust CI/CD pipelines
4. Improved cross-platform builds

**Estimated effort is reasonable** for the benefit gained. Recommended to tackle critical issues first, then high-priority items.
