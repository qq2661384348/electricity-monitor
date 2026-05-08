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
INFRA_MANIFEST_FILE="${SCRIPT_DIR}/infra-manifest.json"
DEPLOY_RESULT_FILE="${SCRIPT_DIR}/deploy-result.json"
BACKUP_TIMESTAMP="$(date +%Y%m%d%H%M%S)"
POSTGRES_DATA_UID="${POSTGRES_DATA_UID:-70}"
POSTGRES_DATA_GID="${POSTGRES_DATA_GID:-70}"
REDIS_DATA_UID="${REDIS_DATA_UID:-999}"
REDIS_DATA_GID="${REDIS_DATA_GID:-999}"
APP_RUNTIME_UID="${APP_RUNTIME_UID:-1000}"
APP_RUNTIME_GID="${APP_RUNTIME_GID:-1000}"

APP_ROLLBACK_IMAGE_REF=""
MANIFEST_GIT_TAG=""
MANIFEST_GIT_SHA=""
MANIFEST_APP_IMAGE_REF=""
MANIFEST_APP_IMAGE_DIGEST=""
MANIFEST_APP_ARCHIVE_FILE=""
MANIFEST_APP_ARCHIVE_SHA256=""
MANIFEST_POSTGRES_IMAGE_REF=""
MANIFEST_POSTGRES_IMAGE_DIGEST=""
MANIFEST_POSTGRES_ARCHIVE_FILE=""
MANIFEST_POSTGRES_ARCHIVE_SHA256=""
MANIFEST_REDIS_IMAGE_REF=""
MANIFEST_REDIS_IMAGE_DIGEST=""
MANIFEST_REDIS_ARCHIVE_FILE=""
MANIFEST_REDIS_ARCHIVE_SHA256=""
DEPENDENCY_RECREATE_REASON=""

require_command() {
    command -v "$1" >/dev/null 2>&1 || error "缺少命令: $1"
}

check_prerequisites() {
    require_command docker
    require_command gzip
    require_command sha256sum
    require_command curl
    require_command stat
    require_command awk
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

prepare_secret_file_owner() {
    local path="$1"

    chown "${APP_RUNTIME_UID}:${APP_RUNTIME_GID}" "${path}"
    chmod 400 "${path}"
}

prepare_secret_files() {
    prepare_secret_file_owner "${APP_DATABASE_PASSWORD_SECRET_FILE}"
    prepare_secret_file_owner "${APP_JWT_SECRET_SECRET_FILE}"
    prepare_secret_file_owner "${APP_QQ_BOT_BEARER_TOKEN_SECRET_FILE}"
    prepare_secret_file_owner "${APP_EMAIL_SMTP_PASSWORD_SECRET_FILE}"
}

validate_data_dir_path() {
    local path="$1"
    local name="$2"

    [ -n "${path}" ] || error "${name} 不能为空"

    case "${path}" in
        "/"|"/root"|"/home"|"/usr"|"/var"|"/etc"|"/opt"|"/tmp")
            error "${name} 指向过宽的系统目录: ${path}"
            ;;
    esac
}

prepare_data_directories() {
    validate_data_dir_path "${POSTGRES_DATA_DIR}" "POSTGRES_DATA_DIR"
    validate_data_dir_path "${REDIS_DATA_DIR}" "REDIS_DATA_DIR"

    mkdir -p "${POSTGRES_DATA_DIR}" "${REDIS_DATA_DIR}"

    # release 使用 bind mount 而不是 Docker named volume，以保证服务器数据
    # 固定落在 <release-root> 下。显式设置容器用户 UID，
    # 避免out-of-repository deployment automation用 owner-only umask 创建 root:root 目录后导致服务无法初始化。
    chown -R "${POSTGRES_DATA_UID}:${POSTGRES_DATA_GID}" "${POSTGRES_DATA_DIR}"
    chown -R "${REDIS_DATA_UID}:${REDIS_DATA_GID}" "${REDIS_DATA_DIR}"
    chmod 700 "${POSTGRES_DATA_DIR}" "${REDIS_DATA_DIR}"
}

is_truthy() {
    case "$(printf '%s' "$1" | tr '[:upper:]' '[:lower:]')" in
        1|true|yes|y|on) return 0 ;;
        *) return 1 ;;
    esac
}

read_manifest_value() {
    local key="$1"
    local file="${2:-${MANIFEST_FILE}}"
    if [ ! -f "${file}" ]; then
        return 0
    fi

    sed -n -E 's/^[[:space:]]*"'${key}'"[[:space:]]*:[[:space:]]*"([^"]*)".*$/\1/p' "${file}" | head -n 1
}

read_manifest_infra_archive_value() {
    local image_name="$1"
    local key="$2"

    if [ ! -f "${MANIFEST_FILE}" ]; then
        return 0
    fi

    awk -v image_name="${image_name}" -v key="${key}" '
        $0 ~ "\"" image_name "\"[[:space:]]*:" {
            in_image = 1
            next
        }
        in_image && $0 ~ "}" {
            exit
        }
        in_image && $0 ~ "\"" key "\"[[:space:]]*:" {
            value = $0
            sub("^[[:space:]]*\"" key "\"[[:space:]]*:[[:space:]]*\"", "", value)
            sub("\".*$", "", value)
            print value
            exit
        }
    ' "${MANIFEST_FILE}"
}

derive_app_archive_file() {
    local image_name=""

    if [ -z "${MANIFEST_GIT_TAG}" ] || [ -z "${MANIFEST_APP_IMAGE_REF}" ]; then
        return 0
    fi

    image_name="${MANIFEST_APP_IMAGE_REF%:*}"
    image_name="${image_name##*/}"
    printf '%s-%s-linux-amd64.tar.gz\n' "${image_name}" "${MANIFEST_GIT_TAG}"
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
    MANIFEST_APP_ARCHIVE_FILE="$(read_manifest_value app_archive_file)"
    MANIFEST_APP_ARCHIVE_SHA256="$(read_manifest_value app_archive_sha256)"
    MANIFEST_POSTGRES_IMAGE_REF="$(read_manifest_value postgres_image_ref)"
    MANIFEST_POSTGRES_IMAGE_DIGEST="$(read_manifest_value postgres_image_digest)"
    MANIFEST_POSTGRES_ARCHIVE_FILE="$(read_manifest_infra_archive_value postgres file)"
    MANIFEST_POSTGRES_ARCHIVE_SHA256="$(read_manifest_infra_archive_value postgres sha256)"
    MANIFEST_REDIS_IMAGE_REF="$(read_manifest_value redis_image_ref)"
    MANIFEST_REDIS_IMAGE_DIGEST="$(read_manifest_value redis_image_digest)"
    MANIFEST_REDIS_ARCHIVE_FILE="$(read_manifest_infra_archive_value redis file)"
    MANIFEST_REDIS_ARCHIVE_SHA256="$(read_manifest_infra_archive_value redis sha256)"

    if [ -z "${MANIFEST_APP_ARCHIVE_FILE}" ]; then
        MANIFEST_APP_ARCHIVE_FILE="$(derive_app_archive_file)"
    fi

    if [ -f "${INFRA_MANIFEST_FILE}" ]; then
        MANIFEST_POSTGRES_ARCHIVE_FILE="${MANIFEST_POSTGRES_ARCHIVE_FILE:-$(read_manifest_value postgres_archive_file "${INFRA_MANIFEST_FILE}")}"
        MANIFEST_POSTGRES_ARCHIVE_SHA256="${MANIFEST_POSTGRES_ARCHIVE_SHA256:-$(read_manifest_value postgres_archive_sha256 "${INFRA_MANIFEST_FILE}")}"
        MANIFEST_REDIS_ARCHIVE_FILE="${MANIFEST_REDIS_ARCHIVE_FILE:-$(read_manifest_value redis_archive_file "${INFRA_MANIFEST_FILE}")}"
        MANIFEST_REDIS_ARCHIVE_SHA256="${MANIFEST_REDIS_ARCHIVE_SHA256:-$(read_manifest_value redis_archive_sha256 "${INFRA_MANIFEST_FILE}")}"
    fi

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
    : "${POSTGRES_CONTAINER_NAME:=electricity-postgres}"
    : "${REDIS_CONTAINER_NAME:=electricity-redis}"
    : "${APP_HOST_PORT:=11450}"
    : "${DEPLOY_HEALTHCHECK_URL:=http://127.0.0.1:${APP_HOST_PORT}/api/health}"
    : "${DEPLOY_HEALTHCHECK_RETRIES:=20}"
    : "${DEPLOY_HEALTHCHECK_INTERVAL:=3}"
    : "${DEPLOY_RECREATE_BASE_SERVICES:=false}"
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
    : "${POSTGRES_IMAGE_REF:=postgres:16-alpine}"
    : "${POSTGRES_USER:=${APP__DATABASE__USERNAME:-postgres}}"
    : "${POSTGRES_DB:=${APP__DATABASE__DATABASE:-electricity_pro}}"
    : "${POSTGRES_DATA_DIR:=./data/postgres}"
    : "${REDIS_IMAGE_REF:=redis:8-alpine}"
    : "${REDIS_DATA_DIR:=./data/redis}"

    [ -f "${APP_DATABASE_PASSWORD_SECRET_FILE}" ] || error "数据库密码 secret file 不存在: ${APP_DATABASE_PASSWORD_SECRET_FILE}"
    [ -f "${APP_JWT_SECRET_SECRET_FILE}" ] || error "JWT secret file 不存在: ${APP_JWT_SECRET_SECRET_FILE}"
    [ -f "${APP_QQ_BOT_BEARER_TOKEN_SECRET_FILE}" ] || error "QQ Bearer token secret file 不存在: ${APP_QQ_BOT_BEARER_TOKEN_SECRET_FILE}"
    [ -f "${APP_EMAIL_SMTP_PASSWORD_SECRET_FILE}" ] || error "SMTP 授权码 secret file 不存在: ${APP_EMAIL_SMTP_PASSWORD_SECRET_FILE}"

    validate_secret_file_permissions "${APP_DATABASE_PASSWORD_SECRET_FILE}"
    validate_secret_file_permissions "${APP_JWT_SECRET_SECRET_FILE}"
    validate_secret_file_permissions "${APP_QQ_BOT_BEARER_TOKEN_SECRET_FILE}"
    validate_secret_file_permissions "${APP_EMAIL_SMTP_PASSWORD_SECRET_FILE}"

    # release 镜像中的应用进程以 uid 1000 非 root 用户运行。Docker Compose
    # file secret 在本地 compose 模式下会保留宿主机文件权限，因此这里把
    # secret owner 显式切到应用 uid，同时继续保持 owner-only 读取。
    prepare_secret_files
    prepare_data_directories

    docker compose --env-file "${ENV_FILE}" -f "${COMPOSE_FILE}" config >/dev/null

    if [ -n "${MANIFEST_APP_IMAGE_REF}" ] && [ "${MANIFEST_APP_IMAGE_REF}" != "${APP_IMAGE_REF}" ]; then
        error "APP_IMAGE_REF (${APP_IMAGE_REF}) 与 release-manifest.json 中的 app_image_ref (${MANIFEST_APP_IMAGE_REF}) 不一致"
    fi

    if [ -n "${MANIFEST_POSTGRES_IMAGE_REF}" ] && [ "${MANIFEST_POSTGRES_IMAGE_REF}" != "${POSTGRES_IMAGE_REF}" ]; then
        error "POSTGRES_IMAGE_REF (${POSTGRES_IMAGE_REF}) 与 release-manifest.json 中的 postgres_image_ref (${MANIFEST_POSTGRES_IMAGE_REF}) 不一致"
    fi

    if [ -n "${MANIFEST_REDIS_IMAGE_REF}" ] && [ "${MANIFEST_REDIS_IMAGE_REF}" != "${REDIS_IMAGE_REF}" ]; then
        error "REDIS_IMAGE_REF (${REDIS_IMAGE_REF}) 与 release-manifest.json 中的 redis_image_ref (${MANIFEST_REDIS_IMAGE_REF}) 不一致"
    fi
}

is_expected_archive_file() {
    local archive_file="$1"

    [ -n "${MANIFEST_APP_ARCHIVE_FILE}" ] && [ "${archive_file}" = "${MANIFEST_APP_ARCHIVE_FILE}" ] && return 0
    [ -n "${MANIFEST_POSTGRES_ARCHIVE_FILE}" ] && [ "${archive_file}" = "${MANIFEST_POSTGRES_ARCHIVE_FILE}" ] && return 0
    [ -n "${MANIFEST_REDIS_ARCHIVE_FILE}" ] && [ "${archive_file}" = "${MANIFEST_REDIS_ARCHIVE_FILE}" ] && return 0

    return 1
}

assert_archive_is_expected() {
    local archive="$1"
    local archive_file=""

    if [ ! -f "${MANIFEST_FILE}" ]; then
        return 0
    fi

    archive_file="$(basename "${archive}")"
    if ! is_expected_archive_file "${archive_file}"; then
        error "发现 release manifest 未声明的镜像归档: ${archive_file}。请移除未知归档或重新生成 release artifact。"
    fi
}

verify_archive_sha256() {
    local archive="$1"
    local expected_sha256="$2"
    local purpose="$3"
    local actual_sha256=""

    [ -n "${expected_sha256}" ] || return 0
    [ -f "${archive}" ] || error "release manifest 要求的 ${purpose} 镜像归档不存在: ${archive}"

    actual_sha256="$(sha256sum "${archive}" | awk '{print $1}')"
    if [ "${actual_sha256}" != "${expected_sha256}" ]; then
        error "${purpose} 镜像归档 SHA256 不一致: $(basename "${archive}")，actual=${actual_sha256}，expected=${expected_sha256}"
    fi

    info "已校验 ${purpose} 镜像归档 SHA256: $(basename "${archive}")"
}

verify_optional_archive_sha256() {
    local archive_file="$1"
    local expected_sha256="$2"
    local purpose="$3"
    local archive_path=""

    [ -n "${archive_file}" ] || return 0
    [ -n "${expected_sha256}" ] || return 0

    archive_path="${IMAGES_DIR}/${archive_file}"
    if [ -f "${archive_path}" ]; then
        verify_archive_sha256 "${archive_path}" "${expected_sha256}" "${purpose}"
    fi
}

verify_release_archives() {
    if [ -n "${MANIFEST_APP_ARCHIVE_SHA256}" ]; then
        [ -n "${MANIFEST_APP_ARCHIVE_FILE}" ] || error "release manifest 缺少 app 镜像归档文件名，无法校验 app_archive_sha256"
        verify_archive_sha256 "${IMAGES_DIR}/${MANIFEST_APP_ARCHIVE_FILE}" "${MANIFEST_APP_ARCHIVE_SHA256}" "app"
    fi

    verify_optional_archive_sha256 "${MANIFEST_POSTGRES_ARCHIVE_FILE}" "${MANIFEST_POSTGRES_ARCHIVE_SHA256}" "postgres"
    verify_optional_archive_sha256 "${MANIFEST_REDIS_ARCHIVE_FILE}" "${MANIFEST_REDIS_ARCHIVE_SHA256}" "redis"
}

load_images() {
    local loaded=0

    while IFS= read -r -d '' archive; do
        assert_archive_is_expected "${archive}"
        info "加载镜像归档: $(basename "${archive}")"
        gzip -dc "${archive}" | docker load >/dev/null
        loaded=$((loaded + 1))
    done < <(find "${IMAGES_DIR}" -maxdepth 1 -type f -name '*.tar.gz' -print0 | sort -z)

    [ "${loaded}" -gt 0 ] || error "未在 ${IMAGES_DIR} 中找到任何 .tar.gz 镜像归档"
}

assert_image_available() {
    local image_ref="$1"
    local purpose="$2"

    if ! docker image inspect "${image_ref}" >/dev/null 2>&1; then
        error "缺少离线镜像: ${image_ref} (${purpose})。请先在同一 release 目录解压 infra images artifact，或确认服务器已预置该镜像；部署脚本不会从外部 registry 拉取镜像。"
    fi
}

assert_required_images_available() {
    # app 包日常只携带应用镜像；PostgreSQL / Redis 镜像可由 infra 包
    # 首次解压或由服务器既有镜像提供。这里在 compose up 前显式检查，
    # 避免 Docker Compose 发现镜像缺失后尝试访问外部 registry。
    assert_image_available "${APP_IMAGE_REF}" "app"
    assert_image_available "${POSTGRES_IMAGE_REF}" "postgres"
    assert_image_available "${REDIS_IMAGE_REF}" "redis"
}

assert_image_digest() {
    local image_ref="$1"
    local expected_digest="$2"
    local purpose="$3"
    local actual_digest=""

    [ -n "${expected_digest}" ] || return 0

    actual_digest="$(docker image inspect "${image_ref}" --format '{{.Id}}')" || error "读取离线镜像摘要失败: ${image_ref} (${purpose})"
    if [ "${actual_digest}" != "${expected_digest}" ]; then
        error "离线镜像摘要与 release manifest 不一致: ${image_ref} (${purpose})，actual=${actual_digest}，expected=${expected_digest}"
    fi
}

assert_required_image_digests() {
    assert_image_digest "${APP_IMAGE_REF}" "${MANIFEST_APP_IMAGE_DIGEST}" "app"
    assert_image_digest "${POSTGRES_IMAGE_REF}" "${MANIFEST_POSTGRES_IMAGE_DIGEST}" "postgres"
    assert_image_digest "${REDIS_IMAGE_REF}" "${MANIFEST_REDIS_IMAGE_DIGEST}" "redis"
}

capture_app_rollback_image() {
    local image_id=""

    if docker container inspect "${APP_CONTAINER_NAME}" >/dev/null 2>&1; then
        image_id="$(docker inspect -f '{{.Image}}' "${APP_CONTAINER_NAME}")"
        APP_ROLLBACK_IMAGE_REF="${APP_CONTAINER_NAME}:rollback-${BACKUP_TIMESTAMP}"
        docker image tag "${image_id}" "${APP_ROLLBACK_IMAGE_REF}" >/dev/null
        info "已为当前应用镜像创建回滚标签: ${APP_ROLLBACK_IMAGE_REF}"
    fi
}

append_dependency_recreate_reason() {
    local purpose="$1"
    local container_name="$2"
    local current_image_id="$3"
    local target_image_id="$4"

    DEPENDENCY_RECREATE_REASON="${DEPENDENCY_RECREATE_REASON}
- ${purpose}: container=${container_name}, current=${current_image_id}, target=${target_image_id}"
}

collect_dependency_recreate_reason() {
    local container_name="$1"
    local image_ref="$2"
    local purpose="$3"
    local current_image_id=""
    local target_image_id=""

    if ! docker container inspect "${container_name}" >/dev/null 2>&1; then
        return 0
    fi

    current_image_id="$(docker inspect -f '{{.Image}}' "${container_name}")"
    target_image_id="$(docker image inspect "${image_ref}" --format '{{.Id}}')" || error "读取目标镜像摘要失败: ${image_ref} (${purpose})"

    if [ "${current_image_id}" != "${target_image_id}" ]; then
        append_dependency_recreate_reason "${purpose}" "${container_name}" "${current_image_id}" "${target_image_id}"
    fi
}

collect_dependency_recreate_reasons() {
    DEPENDENCY_RECREATE_REASON=""
    collect_dependency_recreate_reason "${POSTGRES_CONTAINER_NAME}" "${POSTGRES_IMAGE_REF}" "PostgreSQL"
    collect_dependency_recreate_reason "${REDIS_CONTAINER_NAME}" "${REDIS_IMAGE_REF}" "Redis"
}

start_dependencies() {
    info "启动 PostgreSQL 和 Redis"
    collect_dependency_recreate_reasons

    if [ -n "${DEPENDENCY_RECREATE_REASON}" ]; then
        if ! is_truthy "${DEPLOY_RECREATE_BASE_SERVICES}"; then
            printf '%s\n' "${DEPENDENCY_RECREATE_REASON}" >&2
            error "检测到 PostgreSQL / Redis 现有容器镜像与当前 release 目标镜像不一致。默认 app-only 发布使用 --no-recreate，不会切换基础服务镜像；若这是计划内基础镜像升级，请先确认数据库备份和停机窗口，再设置 DEPLOY_RECREATE_BASE_SERVICES=true 后重跑。"
        fi

        warn "DEPLOY_RECREATE_BASE_SERVICES=true，正在重建 PostgreSQL / Redis 以切换基础镜像。请确认已完成数据库备份。"
        printf '%s\n' "${DEPENDENCY_RECREATE_REASON}" >&2
        docker compose --env-file "${ENV_FILE}" -f "${COMPOSE_FILE}" up -d --force-recreate postgres redis
        return 0
    fi

    docker compose --env-file "${ENV_FILE}" -f "${COMPOSE_FILE}" up -d --no-recreate postgres redis
}

run_migrations() {
    info "执行数据库迁移"
    docker compose --env-file "${ENV_FILE}" -f "${COMPOSE_FILE}" run --rm migrate
}

start_new_release() {
    info "启动新版本应用容器"
    docker compose --env-file "${ENV_FILE}" -f "${COMPOSE_FILE}" up -d --no-deps app
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

rollback() {
    warn "部署失败，开始回滚应用容器"

    if [ -z "${APP_ROLLBACK_IMAGE_REF}" ]; then
        warn "没有可用的应用回滚镜像标签，跳过应用容器回滚"
        return 0
    fi

    if ! docker image inspect "${APP_ROLLBACK_IMAGE_REF}" >/dev/null 2>&1; then
        warn "应用回滚镜像不存在: ${APP_ROLLBACK_IMAGE_REF}"
        return 0
    fi

    APP_IMAGE_REF="${APP_ROLLBACK_IMAGE_REF}" docker compose --env-file "${ENV_FILE}" -f "${COMPOSE_FILE}" up -d --no-deps app
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
  "app_rollback_image": "${APP_ROLLBACK_IMAGE_REF}"
}
EOF
}

main() {
    echo "============================================================"
    echo -e "${GREEN}🚀 Electricity Monitor Release Deploy${NC}"
    echo "============================================================"

    cd "${SCRIPT_DIR}"
    check_prerequisites
    load_manifest
    prepare_env_file
    load_env
    verify_release_archives
    load_images
    assert_required_images_available
    assert_required_image_digests

    capture_app_rollback_image

    if ! start_dependencies; then
        write_deploy_result "failed" "PostgreSQL 或 Redis 启动失败，应用容器未切换"
        error "PostgreSQL 或 Redis 启动失败，应用容器未切换"
    fi

    if ! run_migrations; then
        write_deploy_result "failed" "数据库迁移失败，应用容器未切换"
        error "数据库迁移失败，应用容器未切换"
    fi

    if ! start_new_release; then
        rollback
        write_deploy_result "rolled_back" "docker compose 启动失败，已回滚"
        error "docker compose 启动失败，已尝试回滚"
    fi

    if wait_for_health; then
        write_deploy_result "deployed" "健康检查通过"
        success "部署完成，健康检查通过"
        if [ -n "${APP_ROLLBACK_IMAGE_REF}" ]; then
            info "应用回滚镜像标签: ${APP_ROLLBACK_IMAGE_REF}"
        fi
        return 0
    fi

    rollback
    write_deploy_result "rolled_back" "健康检查失败，已回滚到上一个可运行版本"
    error "健康检查失败，已回滚到上一个可运行版本"
}

main "$@"
