#!/bin/bash
# =============================================================================
# 电力监控系统 - 服务器部署脚本
# =============================================================================
#
# 使用方法:
#   1. 将此脚本和镜像文件放在同一目录
#   2. chmod +x deploy.sh
#   3. ./deploy.sh
#
# 镜像文件（可选，如已加载则跳过）:
#   - electricity-monitor.tar
#
# =============================================================================

set -e

# 颜色
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

info()    { echo -e "${BLUE}[INFO]${NC} $1"; }
success() { echo -e "${GREEN}[OK]${NC} $1"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $1"; }
error()   { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# =============================================================================
# 配置区（按需修改）
# =============================================================================

# 镜像名称
APP_IMAGE="electricity-monitor:latest"

# 容器名称
APP_CONTAINER="electricity-app"
REDIS_CONTAINER="electricity-redis"

# 网络名称
NETWORK_NAME="electricity-net"

# 端口映射（宿主机:容器）
APP_PORT="11451:8000"

# Redis 配置
REDIS_IMAGE="redis:8-alpine"
REDIS_MAX_MEMORY="128mb"

# =============================================================================
# 环境变量配置（按需修改）
# =============================================================================

# Redis 连接（容器内部通信，使用容器名称）
ENV_REDIS_HOST="$REDIS_CONTAINER"
ENV_REDIS_PORT="6379"

# 日志级别（可选：trace/debug/info/warn/error）
ENV_LOG_LEVEL="warn"

# 数据库配置（取消注释并填写实际值）
# ENV_DB_HOST="your-db-host"
# ENV_DB_PORT="5432"
# ENV_DB_USER="postgres"
# ENV_DB_PASS="your-password"
# ENV_DB_NAME="electricity"

# JWT 密钥（生产环境必须修改）
# ENV_JWT_SECRET="your-jwt-secret"

# =============================================================================
# 主逻辑
# =============================================================================

echo "============================================================"
echo -e "${GREEN}🚀 电力监控系统 - 服务器部署${NC}"
echo "============================================================"
echo ""

# 检查 Docker
command -v docker &> /dev/null || error "Docker 未安装"
docker info &> /dev/null || error "Docker 守护进程未运行"

# 加载镜像（如果存在 tar 文件）
if [ -f "electricity-monitor.tar" ]; then
    info "发现镜像文件，正在加载..."
    docker load -i electricity-monitor.tar
    success "镜像加载完成"
    echo ""
fi

# 检查镜像是否存在
if ! docker image inspect "$APP_IMAGE" &> /dev/null; then
    error "镜像 $APP_IMAGE 不存在，请先加载镜像文件"
fi

# 停止并删除旧容器（如果存在）
info "清理旧容器..."
docker rm -f "$APP_CONTAINER" 2>/dev/null || true
docker rm -f "$REDIS_CONTAINER" 2>/dev/null || true

# 创建网络（如果不存在）
if ! docker network inspect "$NETWORK_NAME" &> /dev/null; then
    info "创建 Docker 网络: $NETWORK_NAME"
    docker network create "$NETWORK_NAME"
fi

# 启动 Redis
info "启动 Redis 容器..."
docker run -d \
    --name "$REDIS_CONTAINER" \
    --network "$NETWORK_NAME" \
    --restart unless-stopped \
    "$REDIS_IMAGE" \
    redis-server --save "" --appendonly no --maxmemory "$REDIS_MAX_MEMORY" --maxmemory-policy allkeys-lru

# 等待 Redis 就绪
info "等待 Redis 就绪..."
sleep 2
docker exec "$REDIS_CONTAINER" redis-cli ping > /dev/null || error "Redis 启动失败"
success "Redis 已就绪"

# 构建环境变量参数
ENV_ARGS=(
    -e "APP__REDIS__HOST=$ENV_REDIS_HOST"
    -e "APP__REDIS__PORT=$ENV_REDIS_PORT"
    -e "APP__LOGGING__LEVEL=$ENV_LOG_LEVEL"
)

# 可选：数据库配置
[ -n "$ENV_DB_HOST" ] && ENV_ARGS+=(-e "APP__DATABASE__HOST=$ENV_DB_HOST")
[ -n "$ENV_DB_PORT" ] && ENV_ARGS+=(-e "APP__DATABASE__PORT=$ENV_DB_PORT")
[ -n "$ENV_DB_USER" ] && ENV_ARGS+=(-e "APP__DATABASE__USERNAME=$ENV_DB_USER")
[ -n "$ENV_DB_PASS" ] && ENV_ARGS+=(-e "APP__DATABASE__PASSWORD=$ENV_DB_PASS")
[ -n "$ENV_DB_NAME" ] && ENV_ARGS+=(-e "APP__DATABASE__DATABASE=$ENV_DB_NAME")

# 可选：JWT 密钥
[ -n "$ENV_JWT_SECRET" ] && ENV_ARGS+=(-e "APP__JWT__SECRET=$ENV_JWT_SECRET")

# 启动应用
info "启动应用容器..."
docker run -d \
    --name "$APP_CONTAINER" \
    --network "$NETWORK_NAME" \
    --restart unless-stopped \
    -p "$APP_PORT" \
    "${ENV_ARGS[@]}" \
    "$APP_IMAGE"

# 等待应用就绪
info "等待应用就绪..."
sleep 5

# 健康检查
if curl -sf http://localhost:11451/api/health > /dev/null 2>&1; then
    success "应用健康检查通过"
else
    warn "健康检查未响应，请稍后手动检查"
fi

echo ""
echo "============================================================"
echo -e "${GREEN}🎉 部署完成！${NC}"
echo "============================================================"
echo ""
echo "访问地址: http://$(hostname -I | awk '{print $1}'):11451"
echo ""
echo "常用命令:"
echo "  查看日志:   docker logs -f $APP_CONTAINER"
echo "  停止服务:   docker stop $APP_CONTAINER $REDIS_CONTAINER"
echo "  启动服务:   docker start $REDIS_CONTAINER $APP_CONTAINER"
echo "  重启服务:   docker restart $APP_CONTAINER"
echo ""