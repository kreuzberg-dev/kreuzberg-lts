#!/usr/bin/env pwsh
# Build FFI library for Go bindings
# Used by: ci-go.yaml - Build FFI library step
# Supports: Windows (MinGW), Unix (Linux/macOS)
#
# Environment Variables (Windows):
# - ORT_STRATEGY: Should be set to 'system' for using system ONNX Runtime
# - ORT_LIB_LOCATION: Path to ONNX Runtime lib directory
# - ORT_SKIP_DOWNLOAD: Set to 1 to skip downloading ONNX Runtime
# - ORT_PREFER_DYNAMIC_LINK: Set to 1 for dynamic linking
# - KREUZBERG_BUILD_VERBOSE: Set to 1 for verbose output (default: enabled in CI)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$startTime = Get-Date
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$repoRoot = if ($env:REPO_ROOT) { $env:REPO_ROOT } else { Split-Path -Parent (Split-Path -Parent (Split-Path -Parent $scriptDir)) }
$isVerbose = $env:KREUZBERG_BUILD_VERBOSE -eq "1" -or $env:CI -eq "true"

$IsWindowsOS = $PSVersionTable.Platform -eq 'Win32NT' -or $PSVersionTable.PSVersion.Major -lt 6

if ($IsWindowsOS) {
    Write-Host "=========================================="
    Write-Host "Building Kreuzberg FFI for Windows MinGW"
    Write-Host "=========================================="
    Write-Host ""

    Write-Host "=== System Information ==="
    Write-Host "OS: $([System.Environment]::OSVersion.VersionString)"
    Write-Host "Architecture: $([System.Environment]::Is64BitOperatingSystem ? 'x64' : 'x86')"
    Write-Host "Processor Count: $([Environment]::ProcessorCount)"
    Write-Host ""

    # sccache wrapper can break MinGW gcc builds; ensure it's disabled here
    $env:RUSTC_WRAPPER = ""
    $env:SCCACHE_GHA_ENABLED = "false"
    # zstd-sys: disable legacy compression to avoid problematic legacy source build on MinGW
    $env:ZSTD_DISABLE_LEGACY = "1"

    # MSYS2 UCRT64 toolchain is already added to PATH by CI workflow
    # Set environment variables for toolchain
    $env:CC = "gcc"
    $env:AR = "ar"
    $env:RANLIB = "ranlib"
    $env:PKG_CONFIG = "pkg-config"
    # Use CMake-based build to rely on system zstd from MSYS2
    $env:ZSTD_SYS_USE_CMAKE = "1"
    # NASM required by ring pregenerated objects
    $env:NASM = "nasm"

    # TARGET_* variables are checked FIRST by cc-rs (these are critical for forcing MinGW)
    # cc-rs priority: TARGET_AR/AR_<target>/AR for ar, same for CC, RANLIB
    $env:TARGET_CC = "gcc"
    $env:TARGET_AR = "ar"
    $env:TARGET_RANLIB = "ranlib"
    $env:TARGET_CXX = "g++"

    # Set target-specific environment variables for cc crate
    # The cc crate checks both hyphen and underscore variants
    # This prevents auto-detection from picking MSVC tools on Windows
    $env:CC_x86_64_pc_windows_gnu = "gcc"
    $env:AR_x86_64_pc_windows_gnu = "ar"
    $env:RANLIB_x86_64_pc_windows_gnu = "ranlib"
    ${env:CC_x86_64-pc-windows-gnu} = "gcc"
    ${env:AR_x86_64-pc-windows-gnu} = "ar"
    ${env:RANLIB_x86_64-pc-windows-gnu} = "ranlib"

    # Also set CXX for C++ code
    $env:CXX = "g++"
    $env:CXX_x86_64_pc_windows_gnu = "g++"
    ${env:CXX_x86_64-pc-windows-gnu} = "g++"

    # Cargo-specific linker configuration
    $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER = "x86_64-w64-mingw32-gcc"

    # Disable MSVC detection
    $env:CC_PREFER_CLANG = "1"

    Write-Host "=== Toolchain Verification ==="
    Write-Host "Verifying MinGW toolchain is available..."
    try {
        Write-Host "gcc version:"
        & gcc --version | Select-Object -First 1
    } catch {
        Write-Host "ERROR: gcc not found in PATH" -ForegroundColor Red
        throw "MinGW gcc not available"
    }

    try {
        Write-Host "ar version:"
        & ar --version | Select-Object -First 1
    } catch {
        Write-Host "ERROR: ar not found in PATH" -ForegroundColor Red
        throw "MinGW ar not available"
    }

    try {
        Write-Host "NASM version:"
        & nasm --version | Select-Object -First 1
    } catch {
        Write-Host "WARNING: NASM not found (may be optional)" -ForegroundColor Yellow
    }
    Write-Host ""

    Write-Host "=== Build Configuration ==="
    Write-Host "FFI crate directory: $repoRoot/crates/kreuzberg-ffi"
    Write-Host "Output directory: $repoRoot/target/x86_64-pc-windows-gnu/release"
    Write-Host "Target triple: x86_64-pc-windows-gnu"
    Write-Host ""

    Write-Host "=== Environment Variables ==="
    Write-Host "CC: $env:CC"
    Write-Host "AR: $env:AR"
    Write-Host "RUSTFLAGS: $($env:RUSTFLAGS ?? '<not set>')"
    Write-Host "CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER: $env:CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER"
    Write-Host ""

    Write-Host "=== Building FFI Library ==="
    $TargetTriple = "x86_64-pc-windows-gnu"

    # MinGW cannot use ONNX Runtime (ort-sys) because Windows ONNX Runtime only provides
    # MSVC .lib files, not MinGW-compatible import libraries. Cargo.toml conditionally excludes
    # embeddings/ort features for Windows MinGW builds.
    Write-Host "Building kreuzberg-ffi (ONNX Runtime incompatible with MinGW, conditionally disabled)"

    if ($isVerbose) {
        Write-Host "Running: cargo build -p kreuzberg-ffi --release --target $TargetTriple -vv --timings"
        & cargo build -p kreuzberg-ffi --release --target $TargetTriple -vv --timings
    } else {
        Write-Host "Running: cargo build -p kreuzberg-ffi --release --target $TargetTriple"
        & cargo build -p kreuzberg-ffi --release --target $TargetTriple
    }

    $buildExit = $LASTEXITCODE
    if ($buildExit -ne 0) {
        Write-Host "ERROR: Build failed with exit code $buildExit" -ForegroundColor Red
        throw "Cargo build failed"
    }

    Write-Host ""
    Write-Host "=== Build Artifacts Verification ==="
    $gnuTargetDir = "target\$TargetTriple\release"
    $releaseDir = "target\release"

    Write-Host "Checking for FFI libraries in ${gnuTargetDir}:"
    $builtLibs = Get-ChildItem -Path $gnuTargetDir -Filter "libkreuzberg_ffi.*" -ErrorAction SilentlyContinue
    if ($builtLibs) {
        Write-Host "✓ FFI libraries found:"
        foreach ($lib in $builtLibs) {
            Write-Host "  $($lib.Name) ($([math]::Round($lib.Length / 1MB, 2)) MB)"
        }

        Write-Host ""
        Write-Host "Copying to $releaseDir for CGO..."
        if (-not (Test-Path $releaseDir)) {
            New-Item -ItemType Directory -Path $releaseDir | Out-Null
        }
        foreach ($lib in $builtLibs) {
            Copy-Item -Path $lib.FullName -Destination $releaseDir -Force
            Write-Host "  Copied: $($lib.Name)"
        }
    } else {
        Write-Host "✗ ERROR: No FFI libraries found in $gnuTargetDir" -ForegroundColor Red
        Write-Host "Directory contents:"
        Get-ChildItem -Path $gnuTargetDir -ErrorAction SilentlyContinue | Select-Object -First 10 | ForEach-Object {
            Write-Host "  $($_.Name)"
        }
        throw "Build failed - no artifacts generated"
    }

    Write-Host ""
    Write-Host "Verifying artifacts in ${releaseDir}:"
    if (Test-Path $releaseDir) {
        Get-ChildItem -Path $releaseDir -Filter "libkreuzberg_ffi.*" | ForEach-Object {
            Write-Host "  ✓ $($_.Name)"
        }
    }

} else {
    Write-Host "=========================================="
    Write-Host "Building Kreuzberg FFI for Unix target"
    Write-Host "=========================================="
    Write-Host ""

    Write-Host "=== System Information ==="
    Write-Host "OS: $(uname -s)"
    Write-Host "Architecture: $(uname -m)"
    Write-Host ""

    # Configure ONNX Runtime environment for macOS and Linux
    if ($env:ORT_LIB_LOCATION) {
        Write-Host "=== ONNX Runtime Configuration (Unix) ==="
        Write-Host "ORT_STRATEGY: $($env:ORT_STRATEGY ?? '<not set>')"
        Write-Host "ORT_LIB_LOCATION: $env:ORT_LIB_LOCATION"
        Write-Host "ORT_SKIP_DOWNLOAD: $($env:ORT_SKIP_DOWNLOAD ?? '<not set>')"
        Write-Host "ORT_PREFER_DYNAMIC_LINK: $($env:ORT_PREFER_DYNAMIC_LINK ?? '<not set>')"

        # Ensure RUSTFLAGS includes -L flag for library directory
        if ($env:RUSTFLAGS) {
            if ($env:RUSTFLAGS -notmatch "-L") {
                $env:RUSTFLAGS = "$($env:RUSTFLAGS) -L $($env:ORT_LIB_LOCATION)"
            }
        } else {
            $env:RUSTFLAGS = "-L $($env:ORT_LIB_LOCATION)"
        }
        Write-Host "RUSTFLAGS: $env:RUSTFLAGS"
        Write-Host ""
    }

    Write-Host "=== Building FFI Library ==="
    if ($isVerbose) {
        Write-Host "Running: cargo build -p kreuzberg-ffi --release -vv --timings"
        & cargo build -p kreuzberg-ffi --release -vv --timings
    } else {
        Write-Host "Running: cargo build -p kreuzberg-ffi --release"
        & cargo build -p kreuzberg-ffi --release
    }

    $buildExit = $LASTEXITCODE
    if ($buildExit -ne 0) {
        Write-Host "ERROR: Build failed with exit code $buildExit" -ForegroundColor Red
        throw "Cargo build failed"
    }

    Write-Host ""
    Write-Host "=== Build Artifacts Verification ==="
    $releaseDir = "target\release"
    Write-Host "Checking for FFI libraries in ${releaseDir}:"
    $builtLibs = Get-ChildItem -Path $releaseDir -Filter "libkreuzberg_ffi.*" -ErrorAction SilentlyContinue
    if ($builtLibs) {
        Write-Host "✓ FFI libraries found:"
        foreach ($lib in $builtLibs) {
            Write-Host "  $($lib.Name) ($([math]::Round($lib.Length / 1MB, 2)) MB)"
        }
    } else {
        Write-Host "✗ WARNING: No FFI libraries found" -ForegroundColor Yellow
    }
}

$endTime = Get-Date
$duration = $endTime - $startTime
Write-Host ""
Write-Host "=========================================="
Write-Host "FFI Build completed in $([math]::Round($duration.TotalSeconds, 2))s"
Write-Host "=========================================="
