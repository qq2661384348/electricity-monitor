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
BACKUP_TIMESTAMP="$(date +%Y%m%d%H%M%S)"

APP_BACKUP_NAME=""
REDIS_BACKUP_NAME=""

require_command() {
    command -v "$1" >/dev/null 2>&1 || error "缺少命令: $1"
}

check_prerequisites() {
    require_command docker
    require_command gzip
    require_command curl
    docker info >/dev/null 2>&1 || error "Docker 守护进程未运行"
    docker compose version >/dev/null 2>&1 || error "当前环境缺少 docker compose 插件"

    [ -f "${COMPOSE_FILE}" ] || error "未找到 ${COMPOSE_FILE}"
    [ -d "${IMAGES_DIR}" ] || error "未找到镜像目录 ${IMAGES_DIR}"
}

prepare_env_file() {
    if [ ! -f "${ENV_FILE}" ]; then
        if [ -f "${ENV_EXAMPLE_FILE}" ]; then
            cp "${ENV_EXAMPLE_FILE}" "${ENV_FILE}"
            error "未找到 .env，已按 .env.example 生成默认文件。请至少填写 APP__JWT__SECRET 后重新执行。"
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
    : "${REDIS_IMAGE_REF:=redis:8-alpine}"

    if [ -z "${APP__JWT__SECRET:-}" ] || [ "${APP__JWT__SECRET}" = "CHANGE-ME" ] || [ "${APP__JWT__SECRET}" = "CHANGE-THIS-IN-PRODUCTION-ENV" ]; then
        error "APP__JWT__SECRET 未正确配置，请编辑 ${ENV_FILE}"
    fi

    docker compose --env-file "${ENV_FILE}" -f "${COMPOSE_FILE}" config >/dev/null
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

main() {
    echo "============================================================"
    echo -e "${GREEN}🚀 Electricity Monitor Release Deploy${NC}"
    echo "============================================================"

    check_prerequisites
    prepare_env_file
    load_env
    load_images

    APP_BACKUP_NAME="$(backup_container "${APP_CONTAINER_NAME}")"
    REDIS_BACKUP_NAME="$(backup_container "${REDIS_CONTAINER_NAME}")"

    if ! start_new_release; then
        rollback
        error "docker compose 启动失败，已尝试回滚"
    fi

    if wait_for_health; then
        success "部署完成，健康检查通过"
        if [ -n "${APP_BACKUP_NAME}" ] || [ -n "${REDIS_BACKUP_NAME}" ]; then
            warn "已保留旧容器备份，必要时可手动清理"
            [ -n "${APP_BACKUP_NAME}" ] && info "应用备份: ${APP_BACKUP_NAME}"
            [ -n "${REDIS_BACKUP_NAME}" ] && info "Redis 备份: ${REDIS_BACKUP_NAME}"
        fi
        return 0
    fi

    rollback
    error "健康检查失败，已回滚到上一个可运行版本"
}

main "$@"
