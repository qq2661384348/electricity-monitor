//! Electricity Monitor Backend Server
//!
//! 高性能电力监控系统后端服务器入口

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    electricity_monitor_backend::bootstrap::app::run().await
}
