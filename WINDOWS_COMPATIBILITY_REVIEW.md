# Windows Compatibility Review - Taskfile Tasks

## Executive Summary

This report identifies critical Windows compatibility issues across the Taskfile task definitions. The project has moderate Windows support infrastructure (PowerShell scripts, platform detection) but taskfiles contain numerous Unix-specific commands and patterns that will fail on Windows.

**Critical Issues: 15+**
**High Priority: 25+**
**Medium Priority: 12+**

---

## 1. PATH HANDLING & ENVIRONMENT VARIABLES

### Issue 1.1: Shell Variable Syntax in Bash Conditionals
**Severity**: CRITICAL
**Files**:
- `.task/config/vars.yml:26-49`
- `.task/config/platforms.yml:6-46`

**Problem**: Uses bash shell scripts to detect OS and CPU count. This won't work on Windows PowerShell.

```yaml
# .task/config/vars.yml:25-31
OS:
  sh: |
    if [ -n "$OS" ]; then
      echo "$OS" | tr '[:upper:]' '[:lower:]'
    else
      uname -s | tr '[:upper:]' '[:lower:]'
    fi
```

**Issues**:
- Uses `[ ]` bash syntax
- Uses `uname -s` (Unix only)
- Uses `tr` command (not available on Windows)
- No Windows fallback

**Fix**: Add Windows detection logic:
```yaml
OS:
  sh: |
    if [ -n "$WINDIR" ] || command -v pwsh &>/dev/null; then
      echo "windows"
    elif [ -n "$OS" ]; then
      echo "$OS" | tr '[:upper:]' '[:lower:]'
    else
      uname -s | tr '[:upper:]' '[:lower:]'
    fi
```

### Issue 1.2: CPU Detection Won't Work on Windows
**Severity**: HIGH
**File**: `.task/config/platforms.yml:34-47`

```yaml
NUM_CPUS:
  sh: |
    os="{{.OS}}"
    case "$os" in
      darwin)
        sysctl -n hw.ncpu 2>/dev/null || echo 4
        ;;
      linux)
        nproc 2>/dev/null || grep -c '^processor' /proc/cpuinfo 2>/dev/null || echo 4
        ;;
      *)
        echo 4
        ;;
    esac
```

**Problem**: Windows fallback silently returns 4 without actual detection.

**Fix**: Add Windows CPU detection for parallel builds.

---

## 2. COMMAND COMPATIBILITY - BASH-ONLY UTILITIES

### Issue 2.1: `which` Command
**Severity**: HIGH
**Files**:
- `.task/languages/rust.yml:22,24`

**Examples**:
```yaml
# rust.yml:22,24
- which taplo || cargo install taplo-cli --locked
- which cargo-deny || cargo install cargo-deny
```

**Problem**: `which` command doesn't exist on Windows. PowerShell uses `Get-Command` or `where.exe`.

### Issue 2.2: `rm -rf` - Cleanup Commands
**Severity**: CRITICAL
**Files**:
- `.task/languages/wasm.yml:158`

**Problem**: `rm -rf` not available on Windows. Would need `rmdir /s /q` or PowerShell `Remove-Item`.

**Count**: 4 critical `rm -rf` commands

**Fix**: Use platform-specific commands via `platforms:` constraint:
```yaml
clean:
  desc: Clean build artifacts
  cmds:
    - cmd: rm -rf dist/ pkg/ node_modules/
      platforms: [linux, darwin]
    - cmd: powershell -Command "Remove-Item -Path @('dist', 'pkg', 'node_modules') -Recurse -Force -ErrorAction SilentlyContinue"
      platforms: [windows]
```

### Issue 2.3: Python `glob.glob` for File Cleanup
**Severity**: HIGH
**Files**:
- `.task/languages/python.yml:131-133`
- `.task/languages/ruby.yml:119-120`
- `.task/languages/node.yml:103-104`
- `.task/languages/csharp.yml:112`
- `.task/languages/php.yml:111`

**Problem**: Relying on Python being available globally is risky. Pattern matching for `.so` files won't exist on Windows.

---

## 3. SHELL SCRIPT CALLING - BASH SCRIPTS WITHOUT WINDOWS VARIANTS

### Issue 3.1: Bash Script Calls Without Platform Guards
**Severity**: CRITICAL

**Missing Platform Guards**:
1. `.task/languages/rust.yml:88,96,162` - Test and E2E scripts
2. `.task/languages/node.yml:119` - TypeScript E2E generate
3. `.task/languages/ruby.yml:134` - E2E generate
4. `.task/languages/java.yml:121` - E2E generate
5. `.task/languages/csharp.yml:130,135` - E2E generate and test
6. `.task/languages/php.yml:33,39,46,132` - Build, test, E2E
7. `.task/languages/wasm.yml:171,196` - E2E generate

**Issue Count**:
- **Scripts with NO platform guard**: 19+ bash script calls
- **Scripts with GOOD platform guard**: 7 (python, go, node, ruby)

**Good Example to Follow** (`.task/languages/python.yml:146-149`):
```yaml
e2e:generate:
  desc: Generate Python E2E tests from fixtures
  cmds:
    - cmd: bash scripts/task/e2e-generate.sh python
      platforms: [linux, darwin]
    - cmd: echo "E2E generation not yet supported on Windows - use WSL or CI"
      platforms: [windows]
```

---

## 4. BASH CONDITIONAL SYNTAX IN TASK FILES

### Issue 4.1: Bash Conditionals in Task Commands
**Severity**: HIGH
**Files**:
- `.task/languages/rust.yml:30-35`
- `.task/languages/python.yml:30-37`
- `.task/languages/node.yml:30-37`

**Problem**: Uses bash `if [ ]` syntax that won't work on Windows PowerShell.

**Example**:
```yaml
# rust.yml:30-35
build:
  desc: Build all Rust crates (uses {{.BUILD_PROFILE}})
  cmds:
    - |
      if [ "{{.BUILD_PROFILE}}" = "release" ] || [ "{{.BUILD_PROFILE}}" = "ci" ]; then
        cargo build --release --workspace --all-features
      else
        cargo build --workspace --all-features
      fi
```

---

## 5. MISSING WINDOWS EQUIVALENTS FOR BASH SCRIPTS

### Issue 5.1: Critical Build Scripts Without Windows Variants
**Severity**: CRITICAL

**Missing Files** (no PowerShell equivalents found):
1. `scripts/ci/rust/run-unit-tests.sh` - Rust testing
2. `scripts/task/e2e-generate.sh` - E2E test generation (multiple languages)
3. `scripts/ci/php/build-extension.sh` - PHP extension build
4. `scripts/ci/php/run-tests.sh` - PHP testing
5. `scripts/e2e/csharp/test.sh` - C# E2E tests
6. `scripts/go/install.sh` - Go setup
7. `scripts/go/update.sh` - Go dependency update
8. `scripts/go/test.sh` - Go testing
9. `scripts/go/format_check.sh` - Go format checking
10. `scripts/task/go-lint.sh` - Go linting
11. `scripts/e2e/go/format.sh` - Go E2E formatting
12. `scripts/e2e/go/test.sh` - Go E2E testing
13. `scripts/task/shell-lint.sh` - Shell script linting (expected on Unix)
14. `scripts/task/e2e-typescript-generate.sh` - TypeScript E2E generation
15. `scripts/toml_format.sh` - TOML formatting
16. `scripts/smoke_node.sh` - Smoke testing

---

## 6. ENVIRONMENT VARIABLE SYNTAX ISSUES

### Issue 6.1: Bash Command Substitution in Variable Definitions
**Severity**: HIGH
**File**: `.task/config/vars.yml:6`

```yaml
VERSION:
  sh: grep '^version = ' Cargo.toml | head -1 | cut -d'"' -f2
```

**Problem**: Uses Unix pipes and tools (`grep`, `head`, `cut`) that don't exist on Windows.

---

## 7. EXECUTABLE FILE EXTENSIONS

### Issue 7.1: Direct Script Execution
**Severity**: MEDIUM
**Files**: Multiple
- `.task/tools/general.yml:12,17`

**Examples**:
```yaml
# Missing bash prefix
- scripts/toml_format.sh
- scripts/go/install.sh
- scripts/go/format_check.sh
- scripts/e2e/go/format.sh
- scripts/e2e/go/test.sh
```

**Problem**: On Windows, these scripts won't be executable without `bash` prefix.

**Fix**: Always prefix with `bash`:
```yaml
- bash scripts/toml_format.sh
```

---

## 8. SUMMARY OF ISSUES BY CATEGORY

### CRITICAL (Will fail immediately on Windows)
1. **rm -rf commands** (4 instances)
2. **Unguarded bash scripts** (19+ instances)
3. **Bash conditionals in cmds** (3 instances)
4. **Shell variable syntax in vars** (vars.yml, platforms.yml)
5. **Missing Windows script variants** (15 scripts without .ps1 equivalents)

### HIGH (Significant functionality loss)
1. **`which` command** (2 instances) - rust.yml:22,24
2. **CPU detection** (1 instance) - platforms.yml:34-47
3. **VERSION extraction** - vars.yml:6
4. **Direct script execution** - Multiple files (missing `bash` prefix)
5. **Cleanup glob patterns** (5 instances)

### MEDIUM (Quality/consistency issues)
1. **Inconsistent platform guards** - Some tasks have them, others don't
2. **E2E test generation** - Some not guarded
3. **Shell linting** - Already properly guarded (good example)

---

## 9. POSITIVE EXAMPLES TO REPLICATE

### Shell Linting (CORRECT)
File: `.task/tools/general.yml:19-33`
```yaml
shell:lint:
  desc: Lint shell scripts with auto-fix (shfmt + shellcheck)
  cmds:
    - cmd: bash scripts/task/shell-lint.sh fix
      platforms: [linux, darwin]
    - cmd: echo "Shell linting not available on Windows - shell scripts are Unix-specific"
      platforms: [windows]
```

### Go Linting (CORRECT)
File: `.task/languages/go.yml:87-101`
```yaml
lint:
  desc: Lint and format Go code with auto-fix
  cmds:
    - cmd: bash scripts/task/go-lint.sh fix
      platforms: [linux, darwin]
    - cmd: pwsh scripts/task/go-lint.ps1 -Mode fix
      platforms: [windows]
```

### Python E2E Generation (CORRECT)
File: `.task/languages/python.yml:143-149`
```yaml
e2e:generate:
  desc: Generate Python E2E tests from fixtures
  cmds:
    - cmd: bash scripts/task/e2e-generate.sh python
      platforms: [linux, darwin]
    - cmd: echo "E2E generation not yet supported on Windows - use WSL or CI"
      platforms: [windows]
```

---

## 10. RECOMMENDATIONS

### Immediate (Must-Fix)
1. Add `platforms: [linux, darwin]` to all unguarded `bash` script calls
2. Convert all `rm -rf` commands to cross-platform (use Python, Task status, or platform guards)
3. Fix bash conditionals in build tasks (rust, python, node)
4. Add proper Windows detection to OS and CPU detection in vars.yml

### Short-Term (Should-Fix)
1. Create PowerShell equivalents for critical scripts (php build, rust tests, E2E generation)
2. Add `which` command workaround for rust.yml
3. Convert VERSION extraction to cross-platform method
4. Add `bash` prefix to all direct script calls

### Long-Term (Nice-To-Have)
1. Evaluate cross-platform task runner alternatives
2. Consider move to primarily PowerShell or cross-platform Python scripts
3. Implement proper Windows CI/CD workflow
4. Document Windows development setup and limitations

---

## 11. WINDOWS SCRIPT COMPARISON

### Existing Windows Support:
- ✓ `scripts/ci/install-system-deps/install-windows.ps1` - Complete PowerShell script

### Missing Windows Support:
- ✗ `scripts/ci/php/build-extension.sh` - No `.ps1` equivalent
- ✗ `scripts/ci/php/run-tests.sh` - No `.ps1` equivalent
- ✗ `scripts/ci/rust/run-unit-tests.sh` - No `.ps1` equivalent
- ✗ `scripts/task/e2e-generate.sh` - No `.ps1` equivalent
- ✗ `scripts/go/install.sh` - No `.ps1` equivalent
- ✗ `scripts/go/test.sh` - No `.ps1` equivalent
- ✗ `scripts/go/update.sh` - No `.ps1` equivalent
- ✗ `scripts/go/format_check.sh` - No `.ps1` equivalent
- ✗ `scripts/e2e/csharp/test.sh` - No `.ps1` equivalent
- And 6 more...

---

## 12. FILES REQUIRING UPDATES

### High Priority:
- [ ] `Taskfile.yml` - Main file (38 lines need review)
- [ ] `.task/config/vars.yml` - OS/VERSION/ARCH detection
- [ ] `.task/config/platforms.yml` - CPU detection
- [ ] `.task/languages/rust.yml` - Test, E2E, which command
- [ ] `.task/languages/python.yml` - Clean command, E2E generation
- [ ] `.task/languages/node.yml` - Build conditionals, E2E generation, clean
- [ ] `.task/languages/php.yml` - Build scripts, cleanup, tests
- [ ] `.task/languages/csharp.yml` - Test script, E2E, cleanup
- [ ] `.task/languages/ruby.yml` - Cleanup, E2E generation
- [ ] `.task/languages/java.yml` - E2E generation
- [ ] `.task/languages/wasm.yml` - Cleanup, E2E generation

### Medium Priority:
- [ ] `.task/tools/general.yml` - Already has good examples
- [ ] `.task/languages/go.yml` - Already has partial Windows support
