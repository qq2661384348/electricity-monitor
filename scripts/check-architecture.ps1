$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
Set-Location $repoRoot

$errors = @()

$forbiddenFrontendImports = rg -n '@/services/api' frontend/src -g '*.ts' -g '*.tsx' -g '!frontend/src/services/api.ts'
if ($LASTEXITCODE -eq 0 -and $forbiddenFrontendImports) {
  $errors += "Forbidden frontend imports from '@/services/api':`n$forbiddenFrontendImports"
}

$forbiddenOptimized = rg -n 'electricity_service_optimized|sync_service_optimized' src --glob '!src/domain/services/room_sync/sync_service_optimized.rs'
if ($LASTEXITCODE -eq 0 -and $forbiddenOptimized) {
  $errors += "Forbidden optimized implementation references:`n$forbiddenOptimized"
}

$roomHandlerRepoNew = rg -n 'Repository::new\(' src/handlers/room.rs src/handlers/path_tree.rs
if ($LASTEXITCODE -eq 0 -and $roomHandlerRepoNew) {
  $errors += "Room handlers still instantiate repositories directly:`n$roomHandlerRepoNew"
}

if ($errors.Count -gt 0) {
  $errors | ForEach-Object { Write-Error $_ }
  exit 1
}

Write-Output 'Architecture checks passed.'
exit 0
