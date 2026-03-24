#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="${SCRIPT_DIR}/.env"
MANIFEST_FILE="${SCRIPT_DIR}/release-manifest.json"
DEPLOY_RESULT_FILE="${SCRIPT_DIR}/deploy-result.json"

require_command() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "[ERROR] missing command: $1" >&2
        exit 1
    }
}

load_env() {
    [ -f "${ENV_FILE}" ] || return 0

    while IFS= read -r line || [ -n "${line}" ]; do
        line="${line%$'\r'}"
        case "${line}" in
            ''|'#'*) continue ;;
        esac
        export "${line}"
    done < "${ENV_FILE}"
}

main() {
    require_command curl
    require_command grep

    load_env

    : "${APP_HOST_PORT:=11450}"
    : "${DEPLOY_HEALTHCHECK_URL:=http://127.0.0.1:${APP_HOST_PORT}/api/health}"
    : "${DEPLOY_DB_HEALTHCHECK_URL:=http://127.0.0.1:${APP_HOST_PORT}/api/health/db}"

    echo "[INFO] smoke: checking ${DEPLOY_HEALTHCHECK_URL}"
    curl -fsS "${DEPLOY_HEALTHCHECK_URL}" >/dev/null

    echo "[INFO] smoke: checking ${DEPLOY_DB_HEALTHCHECK_URL}"
    curl -fsS "${DEPLOY_DB_HEALTHCHECK_URL}" >/dev/null

    if [ -f "${MANIFEST_FILE}" ]; then
        echo "[INFO] smoke: manifest present"
        grep -q '"git_tag"' "${MANIFEST_FILE}"
        grep -q '"git_sha"' "${MANIFEST_FILE}"
    else
        echo "[WARN] smoke: manifest missing"
    fi

    if [ -f "${DEPLOY_RESULT_FILE}" ]; then
        echo "[INFO] smoke: deploy-result present"
        grep -q '"status"' "${DEPLOY_RESULT_FILE}"
    else
        echo "[WARN] smoke: deploy-result missing"
    fi

    echo "[OK] smoke checks passed"
}

main "$@"
