# =============================================================================
# 电力监控系统 - Docker 多阶段构建（cargo-chef 优化版）
# =============================================================================
#
# 构建命令:
#   docker build -t electricity-monitor .
#
# 高级构建（使用 BuildKit 缓存）:
#   DOCKER_BUILDKIT=1 docker build -t electricity-monitor .
#
# 运行命令:
#   docker run -p 8000:8000 \
#     -e APP__DATABASE__HOST=your-db-host \
#     -e APP__REDIS__HOST=your-redis-host \
#     electricity-monitor
#
# =============================================================================

# syntax=docker/dockerfile:1.4

# -----------------------------------------------------------------------------
# 阶段 1: Chef 基础镜像（安装 cargo-chef）
# -----------------------------------------------------------------------------
FROM rust:1.87-bookworm AS chef

# 安装构建依赖
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    libssl-dev \
    perl \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# 安装 cargo-chef（用于智能依赖缓存）
RUN cargo install cargo-chef --locked

WORKDIR /app

# -----------------------------------------------------------------------------
# 阶段 2: Planner（分析依赖，生成 recipe.json）
# -----------------------------------------------------------------------------
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY src ./src

# 生成依赖配方（recipe.json）
RUN cargo chef prepare --recipe-path recipe.json

# -----------------------------------------------------------------------------
# 阶段 3: Builder（编译依赖 + 应用）
# -----------------------------------------------------------------------------
FROM chef AS builder

# 复制依赖配方
COPY --from=planner /app/recipe.json recipe.json

# 构建依赖（这一层会被缓存，只有 Cargo.toml/Cargo.lock 变化时才重建）
# 使用 BuildKit 缓存挂载加速 cargo registry 下载
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo chef cook --release --features static-build --recipe-path recipe.json

# 复制源代码和配置
COPY src ./src
COPY build.rs ./
COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY config ./config
COPY diesel.toml ./

# 构建应用（利用已缓存的依赖）
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release --features static-build && \
    cp /app/target/release/server /app/server && \
    cp /app/target/release/migrate /app/migrate

# -----------------------------------------------------------------------------
# 阶段 4: 运行环境（最小化镜像）
# -----------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# 安装运行时依赖并创建非 root 用户
# - ca-certificates: HTTPS 证书
# - curl: 健康检查
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -m -u 1000 app

# 设置工作目录
WORKDIR /app

# 从构建阶段复制二进制文件
COPY --from=builder /app/server /app/server
COPY --from=builder /app/migrate /app/migrate

# 复制配置和静态文件
COPY config ./config
COPY migrations ./migrations

# 创建静态文件目录（运行时挂载或复制）
RUN mkdir -p static && chown -R app:app /app

# 切换到非 root 用户
USER app

# 暴露端口
EXPOSE 8000

# 健康检查
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/api/health || exit 1

# 启动命令
CMD ["./server"]
