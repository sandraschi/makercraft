# MakerCraft start: launches native desktop app (egui)
param([switch]$Build = $false, [Parameter(ValueFromRemainingArguments=$true)][string[]]$AppArgs)
$ErrorActionPreference = "Stop"

$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
if ($null -eq (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: cargo not found." -ForegroundColor Red
    exit 1
}

Set-Location -LiteralPath $PSScriptRoot
if ($Build) { & cargo build -p makercraft-app --bin makercraft }
if ($AppArgs -and $AppArgs.Count -gt 0) { & cargo run -p makercraft-app --bin makercraft -- @AppArgs } else { & cargo run -p makercraft-app --bin makercraft }
