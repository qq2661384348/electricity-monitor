param(
    [string]$ConfigPath = "config/default.toml"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$templatePath = Join-Path $repoRoot "config/development.toml.example"
$runtimeConfigPath = Join-Path $repoRoot $ConfigPath
$placeholder = 'password = "CHANGE-THIS-LOCAL-POSTGRES-PASSWORD"'

if (-not (Test-Path $runtimeConfigPath)) {
    Copy-Item $templatePath $runtimeConfigPath
    Write-Host "Created runtime config from development template: $runtimeConfigPath"
}

$content = Get-Content $runtimeConfigPath -Raw
if ($content -match [regex]::Escape($placeholder)) {
    throw "config/default.toml still contains the template database password placeholder. Update database.password before running backend checks."
}

$env:APP_ENV = "development"
$env:RUN_INTEGRATION_TESTS = "1"
$env:REDIS_HOST = "127.0.0.1"
$env:REDIS_PORT = "6379"

Push-Location $repoRoot
try {
    cargo run --bin migrate
    cargo test --lib
    cargo test --test auth_integration_test
    cargo test --test send_verification_code_integration_test
    cargo test --test release_readiness_test
}
finally {
    Pop-Location
}
