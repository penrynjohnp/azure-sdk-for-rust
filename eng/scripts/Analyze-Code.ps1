#!/usr/bin/env pwsh

#Requires -Version 7.0
param(
  [string]$PackageInfoDirectory,
  [switch]$CheckWasm = $true,
  [switch]$SkipPackageAnalysis
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version 2.0

. (Join-Path $PSScriptRoot '..' 'common' 'scripts' 'common.ps1')

Write-Host @"
Analyzing code with
    RUSTFLAGS: '${env:RUSTFLAGS}'
    RUSTDOCFLAGS: '${env:RUSTDOCFLAGS}'
"@

if ($CheckWasm) {
  Invoke-LoggedCommand "rustup target add wasm32-unknown-unknown"
}

Invoke-LoggedCommand "cargo check --package azure_core --all-features --all-targets --keep-going"

Invoke-LoggedCommand "cargo fmt --all -- --check"

Invoke-LoggedCommand "cargo clippy --workspace --all-features --all-targets --keep-going --no-deps"

if ($CheckWasm) {
  Invoke-LoggedCommand "cargo clippy --target=wasm32-unknown-unknown --workspace --keep-going --no-deps"
}

Invoke-LoggedCommand "cargo doc --workspace --no-deps --all-features"

# Verify package dependencies
$verifyDependenciesScript = Join-Path $RepoRoot 'eng' 'scripts' 'verify-dependencies.rs' -Resolve

if (!$SkipPackageAnalysis) {
  if (!(Test-Path $PackageInfoDirectory)) {
    Write-Host "Analyzing workspace`n"
    return Invoke-LoggedCommand "&$verifyDependenciesScript $RepoRoot/Cargo.toml"
  }

  $packagesToTest = Get-ChildItem $PackageInfoDirectory -Filter "*.json" -Recurse
  | Get-Content -Raw
  | ConvertFrom-Json

  foreach ($package in $packagesToTest) {
    Write-Host "Analyzing package '$($package.Name)' in directory '$($package.DirectoryPath)'`n"
    Invoke-LoggedCommand "&$verifyDependenciesScript $($package.DirectoryPath)/Cargo.toml"
  }
}
