#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="${SCRIPT_DIR}/.env"
TARGETS_FILE="${SCRIPT_DIR}/smoke.targets"
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

load_targets() {
    [ -f "${TARGETS_FILE}" ] || {
        echo "[ERROR] missing smoke targets file: ${TARGETS_FILE}" >&2
        exit 1
    }

    set -a
    # shellcheck disable=SC1090
    . "${TARGETS_FILE}"
    set +a
}

main() {
    require_command curl
    require_command grep

    load_env
    load_targets

    : "${APP_HOST_PORT:=11450}"
    : "${SMOKE_HEALTH_ENDPOINT:=/api/health}"
    : "${SMOKE_DB_HEALTH_ENDPOINT:=/api/health/db}"
    : "${SMOKE_STATIC_ENTRY:=/}"
    : "${SMOKE_REQUIRED_FILES:=release-manifest.json deploy-result.json}"
    : "${DEPLOY_HEALTHCHECK_URL:=http://127.0.0.1:${APP_HOST_PORT}${SMOKE_HEALTH_ENDPOINT}}"
    : "${DEPLOY_DB_HEALTHCHECK_URL:=http://127.0.0.1:${APP_HOST_PORT}${SMOKE_DB_HEALTH_ENDPOINT}}"
    : "${DEPLOY_STATIC_ENTRY_URL:=http://127.0.0.1:${APP_HOST_PORT}${SMOKE_STATIC_ENTRY}}"

    echo "[INFO] smoke: checking ${DEPLOY_HEALTHCHECK_URL}"
    curl -fsS "${DEPLOY_HEALTHCHECK_URL}" >/dev/null

    echo "[INFO] smoke: checking ${DEPLOY_DB_HEALTHCHECK_URL}"
    curl -fsS "${DEPLOY_DB_HEALTHCHECK_URL}" >/dev/null

    echo "[INFO] smoke: checking ${DEPLOY_STATIC_ENTRY_URL}"
    curl -fsS "${DEPLOY_STATIC_ENTRY_URL}" >/dev/null

    for required_file in ${SMOKE_REQUIRED_FILES}; do
        file_path="${SCRIPT_DIR}/${required_file}"

        if [ -f "${file_path}" ]; then
            echo "[INFO] smoke: ${required_file} present"

            if [ "${required_file}" = "$(basename "${MANIFEST_FILE}")" ]; then
                grep -q '"git_tag"' "${file_path}"
                grep -q '"git_sha"' "${file_path}"
            fi

            if [ "${required_file}" = "$(basename "${DEPLOY_RESULT_FILE}")" ]; then
                grep -q '"status"' "${file_path}"
            fi
        else
            echo "[WARN] smoke: ${required_file} missing"
        fi
    done

    echo "[OK] smoke checks passed"
}

main "$@"
