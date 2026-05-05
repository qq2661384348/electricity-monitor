#!/usr/bin/env bash
set -euo pipefail

config_path="${1:-config/development.toml}"

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd -- "${script_dir}/.." && pwd)"
template_path="${repo_root}/config/development.toml.example"
runtime_config_path="${repo_root}/${config_path}"
placeholder='password = "CHANGE-THIS-LOCAL-POSTGRES-PASSWORD"'

if [[ ! -f "${runtime_config_path}" ]]; then
    cp "${template_path}" "${runtime_config_path}"
    echo "Created runtime config from development template: ${runtime_config_path}"
fi

if grep -Fq "${placeholder}" "${runtime_config_path}"; then
    echo "config/development.toml 仍然保留开发模板中的数据库密码占位值。运行后端检查前请先更新 database.password。" >&2
    echo "Linux 开发可以连接系统 PostgreSQL，也可以连接映射到 127.0.0.1:5432 的 Docker PostgreSQL 容器；这里仍需要写入当前本地连接实际使用的密码或非空开发值。" >&2
    exit 1
fi

export APP_ENV=development
export RUN_INTEGRATION_TESTS=1
export REDIS_HOST=127.0.0.1
export REDIS_PORT=6379

cd "${repo_root}"

cargo run --bin migrate
cargo test --lib
cargo test --test auth_integration_test
cargo test --test send_verification_code_integration_test
cargo test --test release_readiness_test
