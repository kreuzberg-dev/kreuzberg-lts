# Extract and test CLI binary (Windows)
# Used by: ci-rust.yaml - Extract and test CLI (Windows) step
# Arguments: TARGET (e.g., x86_64-pc-windows-msvc)

param(
    [Parameter(Mandatory=$true)]
    [string]$Target
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Write-Host "=== Testing CLI binary for $Target ===" -ForegroundColor Green

# Setup library paths
Write-Host "Configuring library paths..." -ForegroundColor Green
if ($env:KREUZBERG_PDFIUM_PREBUILT) {
    Write-Host "  KREUZBERG_PDFIUM_PREBUILT: $env:KREUZBERG_PDFIUM_PREBUILT"
    $env:PATH = "$env:KREUZBERG_PDFIUM_PREBUILT\bin;" + $env:PATH
} else {
    Write-Host "  Warning: KREUZBERG_PDFIUM_PREBUILT not set"
}

# Check for zip file
$zipFile = "kreuzberg-cli-$Target.zip"
Write-Host "Looking for archive: $zipFile" -ForegroundColor Green
if (-not (Test-Path $zipFile)) {
    Write-Host "ERROR: Archive not found: $zipFile" -ForegroundColor Red
    Write-Host "Files in current directory:" -ForegroundColor Yellow
    Get-ChildItem -File | ForEach-Object { Write-Host "  $_" }
    exit 1
}

# Extract archive
Write-Host "Extracting: $zipFile" -ForegroundColor Green
try {
    Expand-Archive -Path $zipFile -DestinationPath . -Force
    Write-Host "Archive extracted successfully" -ForegroundColor Green
} catch {
    Write-Host "ERROR: Failed to extract archive: $_" -ForegroundColor Red
    exit 1
}

# Verify binary exists
$binaryPath = ".\kreuzberg.exe"
Write-Host "Checking for binary: $binaryPath" -ForegroundColor Green
if (-not (Test-Path $binaryPath)) {
    Write-Host "ERROR: Binary not found after extraction" -ForegroundColor Red
    Write-Host "Files extracted:" -ForegroundColor Yellow
    Get-ChildItem -Recurse | ForEach-Object { Write-Host "  $_" }
    exit 1
}

# Test binary
Write-Host "Testing binary version..." -ForegroundColor Green
try {
    & $binaryPath --version
    Write-Host "Version check passed" -ForegroundColor Green
} catch {
    Write-Host "ERROR: Failed to run version check: $_" -ForegroundColor Red
    exit 1
}

Write-Host "Testing binary help..." -ForegroundColor Green
try {
    & $binaryPath --help | Select-Object -First 10
    Write-Host "Help check passed" -ForegroundColor Green
} catch {
    Write-Host "ERROR: Failed to run help: $_" -ForegroundColor Red
    exit 1
}

Write-Host "=== CLI tests passed! ===" -ForegroundColor Green
