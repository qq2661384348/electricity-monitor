#!/usr/bin/env bash
set -euo pipefail

cargo fmt --check
cargo test --lib
cargo test --test release_readiness_test
cargo clippy --all-targets -- -D warnings
cargo audit -q
bash -n deploy/deploy.sh
docker compose -f deploy/docker-compose.local.yml config >/dev/null

# OpenSpec artifacts are intended for open collaboration; keep them free of
# workstation-specific source-tree paths while allowing documented runtime paths.
dev_path_regex='(^|[[:space:]`"(])((/mnt/[[:alnum:]_.-]+/Users/[^[:space:]`")]+)|(/root/[[:alnum:]_.-]+(/[[:alnum:]_.-]+)+)|(/home/[[:alnum:]_.-]+(/[[:alnum:]_.-]+)+)|([A-Za-z]:\\[^[:space:]`")]+)|(\\\\wsl\$\\[^[:space:]`")]+))'
! rg -n "$dev_path_regex" openspec
