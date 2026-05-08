#!/usr/bin/env bash
# =============================================================================
# 电力监控系统 - Docker 运维脚本
# =============================================================================
#
# 用法:
#   ./deploy/build.sh build [TAG]      # 构建镜像
#   ./deploy/build.sh up               # 启动服务（构建 + 运行）
#   ./deploy/build.sh down             # 停止服务
#   ./deploy/build.sh restart          # 重启服务
#   ./deploy/build.sh logs [SERVICE]   # 查看日志
#   ./deploy/build.sh status           # 查看状态
#   ./deploy/build.sh clean            # 清理未使用的镜像
#   ./deploy/build.sh export           # 本地导出镜像（调试/应急，不是推荐生产发布主线）
#
# =============================================================================

set -euo pipefail

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

IMAGE_NAME="electricity-monitor"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_FILE="${SCRIPT_DIR}/docker-compose.local.yml"
DOCKERFILE_PATH="${SCRIPT_DIR}/Dockerfile"

# 辅助函数
info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $1"; }
error()   { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

separator() { echo "============================================================"; }

docker_compose() {
    docker compose -f "${COMPOSE_FILE}" "$@"
}

ensure_runtime_config() {
    local template="${REPO_ROOT}/config/development.toml.example"
    local runtime_config="${REPO_ROOT}/config/development.toml"

    if [ -f "${runtime_config}" ]; then
        return
    fi

    [ -f "${template}" ] || error "缺少开发模板配置: ${template}"
    cp "${template}" "${runtime_config}"
    warn "检测到缺少 config/development.toml，已从 config/development.toml.example 复制本地运行时配置。"
    warn "本地 Docker Compose 会覆盖数据库、Redis 和必要公开运行时值；若离开 Compose 直接 cargo run，仍需按模板填写真实local environment配置。"
}

# 检查 Docker
check_docker() {
    command -v docker &> /dev/null || error "Docker 未安装"
    docker info &> /dev/null || error "Docker 守护进程未运行"
}

# 构建镜像
cmd_build() {
    local tag="${1:-latest}"
    
    separator
    echo -e "${GREEN}🔨 构建镜像: ${IMAGE_NAME}:${tag}${NC}"
    separator

    ensure_runtime_config
    
    local start_time=$(date +%s)
    
    DOCKER_BUILDKIT=1 docker build \
        --progress=plain \
        --file "${DOCKERFILE_PATH}" \
        --tag "${IMAGE_NAME}:${tag}" \
        "${REPO_ROOT}"
    
    local elapsed=$(($(date +%s) - start_time))
    
    echo ""
    success "构建完成！耗时 ${elapsed} 秒"
    docker images "${IMAGE_NAME}" --format "table {{.Tag}}\t{{.Size}}\t{{.CreatedAt}}"
}

# 启动服务
cmd_up() {
    separator
    echo -e "${GREEN}🚀 启动服务${NC}"
    separator

    ensure_runtime_config
    
    # 先构建
    docker_compose build
    
    # 启动
    docker_compose up -d
    
    echo ""
    success "服务已启动"
    echo ""
    info "访问地址: http://127.0.0.1:11450"
    info "健康检查: http://127.0.0.1:11450/api/health"
    echo ""
    info "查看日志: ./deploy/build.sh logs"
    info "停止服务: ./deploy/build.sh down"
}

# 停止服务
cmd_down() {
    info "停止服务..."
    docker_compose down
    success "服务已停止"
}

# 重启服务
cmd_restart() {
    info "重启服务..."
    docker_compose restart
    success "服务已重启"
}

# 查看日志
cmd_logs() {
    local service="${1:-}"
    if [ -n "$service" ]; then
        docker_compose logs -f "$service"
    else
        docker_compose logs -f
    fi
}

# 查看状态
cmd_status() {
    separator
    echo -e "${GREEN}📊 服务状态${NC}"
    separator
    echo ""
    docker_compose ps
    echo ""
    
    # 显示资源使用
    info "资源使用:"
    docker stats --no-stream --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}" \
        $(docker_compose ps -q 2>/dev/null) 2>/dev/null || true
}

# 清理
cmd_clean() {
    info "清理未使用的 Docker 资源..."
    docker system prune -f
    docker image prune -f
    success "清理完成"
}

# 导出镜像
cmd_export() {
    separator
    echo -e "${GREEN}📦 导出镜像${NC}"
    separator

    warn "该命令仅用于本地调试或应急导出；推荐的生产发布主线为 GitHub Actions release artifact"
    ensure_runtime_config
    
    # 导出应用镜像
    info "导出应用镜像..."
    docker save -o "electricity-app.tar" "${IMAGE_NAME}:latest"
    local app_size=$(du -h "electricity-app.tar" | cut -f1)
    success "electricity-app.tar ($app_size)"
    
    # 导出 Redis 镜像
    info "导出 Redis 镜像..."
    docker save -o "electricity-redis.tar" "redis:8-alpine"
    local redis_size=$(du -h "electricity-redis.tar" | cut -f1)
    success "electricity-redis.tar ($redis_size)"
    
    echo ""
    echo "部署到服务器:"
    echo "  1. 上传以下镜像归档到服务器:"
    echo "     - electricity-app.tar"
    echo "     - electricity-redis.tar"
    echo "  2. 同时从仓库 deploy/ 目录携带以下模板:"
    echo "     - deploy/deploy.sh"
    echo "     - deploy/compose.release.yml"
    echo "     - deploy/release.env.example"
    echo "  3. 在服务器按 release 包同名方式重命名后执行 deploy.sh"
}

# 帮助信息
cmd_help() {
    echo "电力监控系统 - Docker 运维脚本"
    echo ""
    echo "用法: ./deploy/build.sh <命令> [参数]"
    echo ""
    echo "命令:"
    echo "  build [TAG]     构建镜像（默认 TAG: latest）"
    echo "  up              启动服务（构建 + 运行）"
    echo "  down            停止服务"
    echo "  restart         重启服务"
    echo "  logs [SERVICE]  查看日志（app、postgres 或 redis）"
    echo "  status          查看服务状态"
    echo "  export [FILE]   导出镜像为 tar 文件"
    echo "  clean           清理未使用的镜像"
    echo "  help            显示帮助信息"
    echo ""
    echo "示例:"
    echo "  ./deploy/build.sh build    # 构建镜像"
    echo "  ./deploy/build.sh up       # 启动所有服务"
    echo "  ./deploy/build.sh export   # 本地导出镜像（调试/应急）"
    echo "  ./deploy/build.sh logs app # 只看应用日志"
}

# 主入口
main() {
    check_docker
    
    local cmd="${1:-help}"
    shift || true
    
    case "$cmd" in
        build)   cmd_build "$@" ;;
        up)      cmd_up ;;
        down)    cmd_down ;;
        restart) cmd_restart ;;
        logs)    cmd_logs "$@" ;;
        status)  cmd_status ;;
        export)  cmd_export "$@" ;;
        clean)   cmd_clean ;;
        help|--help|-h) cmd_help ;;
        *)       error "未知命令: $cmd（使用 ./deploy/build.sh help 查看帮助）" ;;
    esac
}

main "$@"
