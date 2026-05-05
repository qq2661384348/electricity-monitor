param(
    [string]$ConfigPath = "config/development.toml"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$templatePath = Join-Path $repoRoot "config/development.toml.example"
$runtimeConfigPath = Join-Path $repoRoot $ConfigPath
$placeholder = 'password = "CHANGE-THIS-LOCAL-POSTGRES-PASSWORD"'
$qqApiUrlPlaceholder = 'api_url = "你的napcat应用URL"'
$qqApiUrlEmptyPlaceholder = 'api_url = ""'
$publicQqPlaceholder = 'public_qq_number = ""'
$qqTokenPlaceholder = 'bearer_token = ""'
$publicSiteDomainPlaceholder = 'domain = ""'
$publicSitePortPlaceholder = 'port = ""'

if (-not (Test-Path $runtimeConfigPath)) {
    Copy-Item $templatePath $runtimeConfigPath
    Write-Host "Created runtime config from development template: $runtimeConfigPath"
}

$content = Get-Content $runtimeConfigPath -Raw
if ($content -match [regex]::Escape($placeholder)) {
    throw "config/development.toml 仍然保留开发模板中的数据库密码占位值。运行后端检查前请先更新 database.password。"
}

if (
    $content -match [regex]::Escape($qqApiUrlPlaceholder) -or
    $content -match [regex]::Escape($qqApiUrlEmptyPlaceholder) -or
    $content -match [regex]::Escape($publicQqPlaceholder) -or
    $content -match [regex]::Escape($qqTokenPlaceholder) -or
    $content -match [regex]::Escape($publicSiteDomainPlaceholder) -or
    $content -match [regex]::Escape($publicSitePortPlaceholder)
) {
    throw "config/development.toml 仍有运行时通知配置留空。运行后端检查前请填写 qq_bot.api_url、qq_bot.public_qq_number、qq_bot.bearer_token、public_site.domain 与 public_site.port。"
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
