#!/usr/bin/env pwsh

param(
    [Parameter(Mandatory=$true)]
    [string]$Target
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Write-Host "=== Packaging CLI binary for $Target ==="

cd target/$Target/release
Compress-Archive -Path kreuzberg.exe -DestinationPath ../../../kreuzberg-cli-$Target.zip

Write-Host "Packaging complete: kreuzberg-cli-$Target.zip"
