#!/usr/bin/env bash

set -euo pipefail

GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $1"; }
error()   { echo -e "${RED}[ERROR]${NC} $1" >&2; exit 1; }

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="${SCRIPT_DIR}/compose.yaml"
ENV_FILE="${SCRIPT_DIR}/.env"
ENV_EXAMPLE_FILE="${SCRIPT_DIR}/.env.example"
IMAGES_DIR="${SCRIPT_DIR}/images"
MANIFEST_FILE="${SCRIPT_DIR}/release-manifest.json"
DEPLOY_RESULT_FILE="${SCRIPT_DIR}/deploy-result.json"
BACKUP_TIMESTAMP="$(date +%Y%m%d%H%M%S)"

APP_BACKUP_NAME=""
REDIS_BACKUP_NAME=""
MANIFEST_GIT_TAG=""
MANIFEST_GIT_SHA=""
MANIFEST_APP_IMAGE_REF=""
MANIFEST_APP_IMAGE_DIGEST=""

require_command() {
    command -v "$1" >/dev/null 2>&1 || error "缺少命令: $1"
}

check_prerequisites() {
    require_command docker
    require_command gzip
    require_command curl
    require_command stat
    docker info >/dev/null 2>&1 || error "Docker 守护进程未运行"
    docker compose version >/dev/null 2>&1 || error "当前环境缺少 docker compose 插件"

    [ -f "${COMPOSE_FILE}" ] || error "未找到 ${COMPOSE_FILE}"
    [ -d "${IMAGES_DIR}" ] || error "未找到镜像目录 ${IMAGES_DIR}"
}

validate_secret_file_permissions() {
    local path="$1"
    local permissions=""

    [ -f "${path}" ] || error "secret file 不存在或不是常规文件: ${path}"

    permissions="$(stat -c '%A' "${path}")" || error "读取 secret file 权限失败: ${path}"

    if [ "${permissions:4:6}" != "------" ]; then
        error "secret file 权限过宽: ${path} (${permissions})。请收紧到仅 owner 可读写，例如 chmod 600。"
    fi
}

read_manifest_value() {
    local key="$1"
    if [ ! -f "${MANIFEST_FILE}" ]; then
        return 0
    fi

    sed -n -E 's/^[[:space:]]*"'${key}'"[[:space:]]*:[[:space:]]*"([^"]*)".*$/\1/p' "${MANIFEST_FILE}" | head -n 1
}

load_manifest() {
    if [ ! -f "${MANIFEST_FILE}" ]; then
        warn "未找到 release-manifest.json，将继续部署但缺少版本审计信息"
        return 0
    fi

    MANIFEST_GIT_TAG="$(read_manifest_value git_tag)"
    MANIFEST_GIT_SHA="$(read_manifest_value git_sha)"
    MANIFEST_APP_IMAGE_REF="$(read_manifest_value app_image_ref)"
    MANIFEST_APP_IMAGE_DIGEST="$(read_manifest_value app_image_digest)"

    info "读取 release manifest: tag=${MANIFEST_GIT_TAG:-unknown}, git_sha=${MANIFEST_GIT_SHA:-unknown}, app_image_ref=${MANIFEST_APP_IMAGE_REF:-unknown}"
}

prepare_env_file() {
    if [ ! -f "${ENV_FILE}" ]; then
        if [ -f "${ENV_EXAMPLE_FILE}" ]; then
            cp "${ENV_EXAMPLE_FILE}" "${ENV_FILE}"
            error "未找到 .env，已按 .env.example 生成默认文件。请至少准备 secrets 文件并填写对应 secret file 路径后重新执行。"
        fi

        error "未找到 .env 和 .env.example，无法继续部署"
    fi
}

load_env() {
    while IFS= read -r line || [ -n "${line}" ]; do
        line="${line%$'\r'}"

        case "${line}" in
            ''|'#'*) continue ;;
        esac

        if [[ "${line}" != *=* ]]; then
            error "环境文件存在非法行: ${line}"
        fi

        local key="${line%%=*}"
        local value="${line#*=}"

        if [[ "${value}" =~ ^\".*\"$ ]] || [[ "${value}" =~ ^\'.*\'$ ]]; then
            value="${value:1:${#value}-2}"
        fi

        export "${key}=${value}"
    done < "${ENV_FILE}"

    : "${APP_CONTAINER_NAME:=electricity-app}"
    : "${REDIS_CONTAINER_NAME:=electricity-redis}"
    : "${APP_HOST_PORT:=11450}"
    : "${DEPLOY_HEALTHCHECK_URL:=http://127.0.0.1:${APP_HOST_PORT}/api/health}"
    : "${DEPLOY_HEALTHCHECK_RETRIES:=20}"
    : "${DEPLOY_HEALTHCHECK_INTERVAL:=3}"
    : "${APP_IMAGE_REF:?APP_IMAGE_REF 未配置}"
    : "${APP_DATABASE_PASSWORD_SECRET_FILE:?APP_DATABASE_PASSWORD_SECRET_FILE 未配置}"
    : "${APP_JWT_SECRET_SECRET_FILE:?APP_JWT_SECRET_SECRET_FILE 未配置}"
    : "${APP_QQ_BOT_BEARER_TOKEN_SECRET_FILE:?APP_QQ_BOT_BEARER_TOKEN_SECRET_FILE 未配置}"
    : "${APP_EMAIL_SMTP_PASSWORD_SECRET_FILE:?APP_EMAIL_SMTP_PASSWORD_SECRET_FILE 未配置}"
    : "${APP__CORS__ALLOWED_ORIGINS:?APP__CORS__ALLOWED_ORIGINS 未配置}"
    : "${APP__QQ_BOT__API_URL:?APP__QQ_BOT__API_URL 未配置}"
    : "${APP__QQ_BOT__PUBLIC_QQ_NUMBER:?APP__QQ_BOT__PUBLIC_QQ_NUMBER 未配置}"
    : "${APP__PUBLIC_SITE__DOMAIN:?APP__PUBLIC_SITE__DOMAIN 未配置}"
    : "${APP__PUBLIC_SITE__PORT:?APP__PUBLIC_SITE__PORT 未配置}"
    : "${APP__ADMIN__DEFAULT_QQ_NUMBER:?APP__ADMIN__DEFAULT_QQ_NUMBER 未配置}"
    : "${REDIS_IMAGE_REF:=redis:8-alpine}"

    [ -f "${APP_DATABASE_PASSWORD_SECRET_FILE}" ] || error "数据库密码 secret file 不存在: ${APP_DATABASE_PASSWORD_SECRET_FILE}"
    [ -f "${APP_JWT_SECRET_SECRET_FILE}" ] || error "JWT secret file 不存在: ${APP_JWT_SECRET_SECRET_FILE}"
    [ -f "${APP_QQ_BOT_BEARER_TOKEN_SECRET_FILE}" ] || error "QQ Bearer token secret file 不存在: ${APP_QQ_BOT_BEARER_TOKEN_SECRET_FILE}"
    [ -f "${APP_EMAIL_SMTP_PASSWORD_SECRET_FILE}" ] || error "SMTP 授权码 secret file 不存在: ${APP_EMAIL_SMTP_PASSWORD_SECRET_FILE}"

    validate_secret_file_permissions "${APP_DATABASE_PASSWORD_SECRET_FILE}"
    validate_secret_file_permissions "${APP_JWT_SECRET_SECRET_FILE}"
    validate_secret_file_permissions "${APP_QQ_BOT_BEARER_TOKEN_SECRET_FILE}"
    validate_secret_file_permissions "${APP_EMAIL_SMTP_PASSWORD_SECRET_FILE}"

    docker compose --env-file "${ENV_FILE}" -f "${COMPOSE_FILE}" config >/dev/null

    if [ -n "${MANIFEST_APP_IMAGE_REF}" ] && [ "${MANIFEST_APP_IMAGE_REF}" != "${APP_IMAGE_REF}" ]; then
        error "APP_IMAGE_REF (${APP_IMAGE_REF}) 与 release-manifest.json 中的 app_image_ref (${MANIFEST_APP_IMAGE_REF}) 不一致"
    fi
}

load_images() {
    local loaded=0

    while IFS= read -r -d '' archive; do
        info "加载镜像归档: $(basename "${archive}")"
        gzip -dc "${archive}" | docker load >/dev/null
        loaded=$((loaded + 1))
    done < <(find "${IMAGES_DIR}" -maxdepth 1 -type f -name '*.tar.gz' -print0 | sort -z)

    [ "${loaded}" -gt 0 ] || error "未在 ${IMAGES_DIR} 中找到任何 .tar.gz 镜像归档"
}

backup_container() {
    local container_name="$1"
    local backup_name=""

    if docker container inspect "${container_name}" >/dev/null 2>&1; then
        backup_name="${container_name}-backup-${BACKUP_TIMESTAMP}"
        local image_id
        image_id="$(docker inspect -f '{{.Image}}' "${container_name}")"

        docker image tag "${image_id}" "${container_name}:rollback-${BACKUP_TIMESTAMP}" >/dev/null 2>&1 || true

        info "备份现有容器 ${container_name} -> ${backup_name}"
        docker stop "${container_name}" >/dev/null 2>&1 || true
        docker rename "${container_name}" "${backup_name}"
    fi

    printf '%s' "${backup_name}"
}

start_new_release() {
    info "启动新版本容器"
    docker compose --env-file "${ENV_FILE}" -f "${COMPOSE_FILE}" up -d
}

wait_for_health() {
    local attempt=1

    while [ "${attempt}" -le "${DEPLOY_HEALTHCHECK_RETRIES}" ]; do
        if curl -fsS "${DEPLOY_HEALTHCHECK_URL}" >/dev/null 2>&1; then
            return 0
        fi

        info "健康检查未通过，等待后重试 (${attempt}/${DEPLOY_HEALTHCHECK_RETRIES})"
        sleep "${DEPLOY_HEALTHCHECK_INTERVAL}"
        attempt=$((attempt + 1))
    done

    return 1
}

restore_container() {
    local backup_name="$1"
    local stable_name="$2"

    if [ -n "${backup_name}" ] && docker container inspect "${backup_name}" >/dev/null 2>&1; then
        info "恢复容器 ${backup_name} -> ${stable_name}"
        docker rename "${backup_name}" "${stable_name}"
        docker start "${stable_name}" >/dev/null
    fi
}

rollback() {
    warn "部署失败，开始回滚"

    docker rm -f "${APP_CONTAINER_NAME}" >/dev/null 2>&1 || true
    docker rm -f "${REDIS_CONTAINER_NAME}" >/dev/null 2>&1 || true

    restore_container "${REDIS_BACKUP_NAME}" "${REDIS_CONTAINER_NAME}"
    restore_container "${APP_BACKUP_NAME}" "${APP_CONTAINER_NAME}"
}

write_deploy_result() {
    local status="$1"
    local message="$2"

    cat > "${DEPLOY_RESULT_FILE}" <<EOF
{
  "status": "${status}",
  "message": "${message}",
  "deployed_at": "$(date -u +'%Y-%m-%dT%H:%M:%SZ')",
  "git_tag": "${MANIFEST_GIT_TAG}",
  "git_sha": "${MANIFEST_GIT_SHA}",
  "app_image_ref": "${MANIFEST_APP_IMAGE_REF:-${APP_IMAGE_REF}}",
  "app_image_digest": "${MANIFEST_APP_IMAGE_DIGEST}",
  "healthcheck_url": "${DEPLOY_HEALTHCHECK_URL}",
  "app_backup": "${APP_BACKUP_NAME}",
  "redis_backup": "${REDIS_BACKUP_NAME}"
}
EOF
}

main() {
    echo "============================================================"
    echo -e "${GREEN}🚀 Electricity Monitor Release Deploy${NC}"
    echo "============================================================"

    check_prerequisites
    load_manifest
    prepare_env_file
    load_env
    load_images

    APP_BACKUP_NAME="$(backup_container "${APP_CONTAINER_NAME}")"
    REDIS_BACKUP_NAME="$(backup_container "${REDIS_CONTAINER_NAME}")"

    if ! start_new_release; then
        rollback
        write_deploy_result "rolled_back" "docker compose 启动失败，已回滚"
        error "docker compose 启动失败，已尝试回滚"
    fi

    if wait_for_health; then
        write_deploy_result "deployed" "健康检查通过"
        success "部署完成，健康检查通过"
        if [ -n "${APP_BACKUP_NAME}" ] || [ -n "${REDIS_BACKUP_NAME}" ]; then
            warn "已保留旧容器备份，必要时可手动清理"
            [ -n "${APP_BACKUP_NAME}" ] && info "应用备份: ${APP_BACKUP_NAME}"
            [ -n "${REDIS_BACKUP_NAME}" ] && info "Redis 备份: ${REDIS_BACKUP_NAME}"
        fi
        return 0
    fi

    rollback
    write_deploy_result "rolled_back" "健康检查失败，已回滚到上一个可运行版本"
    error "健康检查失败，已回滚到上一个可运行版本"
}

main "$@"
