#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"

$target = $env:CLI_TARGET
if ([string]::IsNullOrWhiteSpace($target)) { throw "CLI_TARGET is required" }

$stage = "kreuzberg-cli-$target"
Remove-Item -Recurse -Force $stage -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Path $stage | Out-Null

$candidates = @(
  ("target/" + $target + "/release/kreuzberg.exe"),
  ("target/release/kreuzberg.exe"),
  ("target/" + $target + "/release/kreuzberg-cli.exe"),
  ("target/release/kreuzberg-cli.exe")
)

$exePath = $null
foreach ($p in $candidates) {
  if (Test-Path $p) { $exePath = $p; break }
}

if (-not $exePath) {
  Write-Host "CLI binary not found. Searched:"
  $candidates | ForEach-Object { Write-Host "  - $_" }
  Write-Host ""
  Write-Host "Directory listing (target):"
  if (Test-Path "target") { Get-ChildItem -Recurse -Depth 3 "target" | Select-Object FullName | Format-Table -AutoSize } else { Write-Host "  (target directory missing)" }
  throw "CLI binary not found for target $target"
}

Copy-Item $exePath $stage
Copy-Item "LICENSE" $stage
Copy-Item "README.md" $stage

if (Test-Path ("target/" + $target + "/release/pdfium.dll")) {
  Copy-Item ("target/" + $target + "/release/pdfium.dll") $stage
}

# ~keep Bundle the CPU ONNX Runtime DLL next to the exe. The CLI builds with
# ort-dynamic and dlopens ONNX Runtime at runtime, honoring ORT_DYLIB_PATH for
# GPU builds; the Windows loader resolves the fallback DLL from the exe's own
# directory. ORT_BUNDLE_DIR is the extracted official ORT win-x64 tgz root.
$ortBundleDir = $env:ORT_BUNDLE_DIR
if ($ortBundleDir -and (Test-Path (Join-Path $ortBundleDir "lib"))) {
  $ortLib = Join-Path $ortBundleDir "lib"
  Get-ChildItem -Path $ortLib -Filter "onnxruntime.dll*" | ForEach-Object {
    Copy-Item $_.FullName $stage
  }
} else {
  Write-Host "skipping ONNX Runtime bundling (ORT_BUNDLE_DIR is unset or empty)"
}

Compress-Archive -Path "$stage/*" -DestinationPath ($stage + ".zip") -Force
Remove-Item -Recurse -Force $stage
