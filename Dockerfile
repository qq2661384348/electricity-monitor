# =============================================================================
# 电力监控系统 - Docker 多阶段构建
# =============================================================================
#
# 构建命令:
#   docker build -t electricity-monitor .
#
# 运行命令:
#   docker run -p 8000:8000 \
#     -e APP__DATABASE__HOST=your-db-host \
#     -e APP__REDIS__HOST=your-redis-host \
#     electricity-monitor
#
# =============================================================================

# -----------------------------------------------------------------------------
# 阶段 1: 构建环境
# -----------------------------------------------------------------------------
FROM rust:1.83-bookworm AS builder

# 安装构建依赖（按字母顺序排列）
# - build-essential: C 编译器等基础工具
# - cmake: 某些依赖可能需要
# - libssl-dev: OpenSSL 开发头文件（vendored 特性需要）
# - perl: OpenSSL 构建脚本需要
# - pkg-config: 库路径发现
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    cmake \
    libssl-dev \
    perl \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# 设置工作目录
WORKDIR /app

# 复制依赖文件（利用 Docker 缓存）
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./

# 创建虚拟 src 目录以缓存依赖编译
RUN mkdir -p src/bin && \
    echo 'fn main() { println!("placeholder"); }' > src/main.rs && \
    echo 'fn main() { println!("placeholder"); }' > src/bin/migrate.rs && \
    echo '' > src/lib.rs

# 预编译依赖（这一步会被缓存）
# 使用 static-build feature 启用静态链接
RUN cargo build --release --features static-build && rm -rf src

# 复制实际源代码
COPY src ./src
COPY migrations ./migrations
COPY config ./config
COPY diesel.toml ./

# 触发重新编译（因为源代码变了）
RUN touch src/main.rs && touch src/lib.rs

# 正式构建（使用静态链接）
RUN cargo build --release --features static-build

# -----------------------------------------------------------------------------
# 阶段 2: 运行环境
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
COPY --from=builder /app/target/release/server /app/server
COPY --from=builder /app/target/release/migrate /app/migrate

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
