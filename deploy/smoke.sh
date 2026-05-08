#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ENV_FILE="${SCRIPT_DIR}/.env"
TARGETS_FILE="${SCRIPT_DIR}/smoke.targets"
MANIFEST_FILE="${SCRIPT_DIR}/release-manifest.json"
DEPLOY_RESULT_FILE="${SCRIPT_DIR}/deploy-result.json"
declare -a REQUIRED_HEADER_NAMES=()
declare -a REQUIRED_HEADER_VALUES=()

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

    override_captcha_csp_header
    load_required_headers
}

override_captcha_csp_header() {
    local api_url="${APP__CAPTCHA__API_URL:-}"
    local captcha_origin=""
    local scheme=""
    local rest=""
    local authority=""

    [ -n "${api_url}" ] || return 0

    case "${api_url}" in
        https://*)
            scheme="https"
            rest="${api_url#https://}"
            ;;
        http://*)
            scheme="http"
            rest="${api_url#http://}"
            ;;
        *)
            return 0
            ;;
    esac

    authority="${rest%%[/?#]*}"
    [ -n "${authority}" ] || return 0
    captcha_origin="${scheme}://${authority}"

    case "${captcha_origin}" in
        *[[:space:]]*|*\'*|*\"*|*\\*|*';'*|*'@'*)
            return 0
            ;;
    esac

    SMOKE_REQUIRED_HEADER__CONTENT_SECURITY_POLICY="default-src 'self'; base-uri 'self'; frame-ancestors 'none'; object-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline' https://fonts.googleapis.cn; font-src 'self' https://fonts.gstatic.cn data:; img-src 'self' data: blob:; connect-src 'self' ${captcha_origin}"
}

load_required_headers() {
    local variable_name=""
    local suffix=""
    local header_name=""

    while IFS= read -r variable_name; do
        suffix="${variable_name#SMOKE_REQUIRED_HEADER__}"
        header_name="$(printf '%s' "${suffix}" | tr '[:upper:]' '[:lower:]' | tr '_' '-')"
        REQUIRED_HEADER_NAMES+=("${header_name}")
        REQUIRED_HEADER_VALUES+=("${!variable_name}")
    done < <(compgen -A variable SMOKE_REQUIRED_HEADER__ | sort)
}

assert_response_headers() {
    local url="$1"
    local response_headers="$2"
    local index=""
    local header_name=""
    local expected_value=""
    local actual_value=""

    for index in "${!REQUIRED_HEADER_NAMES[@]}"; do
        header_name="${REQUIRED_HEADER_NAMES[${index}]}"
        expected_value="${REQUIRED_HEADER_VALUES[${index}]}"
        actual_value="$(
            printf '%s\n' "${response_headers}" \
                | awk -v header_name="${header_name}" '
                    BEGIN { IGNORECASE = 1 }
                    $0 ~ ("^" header_name ":") {
                        sub(/^[^:]+:[[:space:]]*/, "", $0)
                        sub(/\r$/, "", $0)
                        print
                        exit
                    }
                '
        )"

        if [ -z "${actual_value}" ]; then
            echo "[ERROR] smoke: ${url} 缺少响应头 ${header_name}" >&2
            exit 1
        fi

        if [ "${actual_value}" != "${expected_value}" ]; then
            echo "[ERROR] smoke: ${url} 的响应头 ${header_name}=${actual_value}，期望 ${expected_value}" >&2
            exit 1
        fi
    done
}

check_endpoint() {
    local url="$1"
    local response_headers=""

    echo "[INFO] smoke: checking ${url}"
    response_headers="$(curl -fsS -D - -o /dev/null "${url}")"
    assert_response_headers "${url}" "${response_headers}"
}

main() {
    require_command curl
    require_command grep
    require_command awk

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

    check_endpoint "${DEPLOY_HEALTHCHECK_URL}"
    check_endpoint "${DEPLOY_DB_HEALTHCHECK_URL}"
    check_endpoint "${DEPLOY_STATIC_ENTRY_URL}"

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
            echo "[ERROR] smoke: ${required_file} missing" >&2
            exit 1
        fi
    done

    echo "[OK] smoke checks passed"
}

main "$@"
