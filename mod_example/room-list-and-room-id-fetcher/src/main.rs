//! 房间信息爬取工具
//!
//! 高性能并发爬取房间信息，输出 JSON 文件

// 性能优化：使用 mimalloc 高性能内存分配器
// 相比系统默认分配器，在并发场景下可提升 15-25% 性能
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// 模块声明
mod client;
mod fetcher;
mod models;
mod parser;

use anyhow::Result;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter as TokioBufWriter};
use tracing_subscriber::EnvFilter;

use client::RoomClient;
use fetcher::RoomFetcher;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. 初始化日志系统（DEBUG 级别）
    init_tracing()?;

    tracing::info!("🚀 房间信息爬取工具启动");
    tracing::info!("📊 配置：并发数 = 50，重试次数 = 3");

    // 2. 创建 HTTP 客户端
    tracing::info!("🔧 正在初始化 HTTP 客户端...");
    let client = RoomClient::new()?;
    tracing::info!("✓ HTTP 客户端初始化成功");

    // 3. 创建爬取器（优化的并发模型）
    let fetcher = RoomFetcher::new(client);

    // 4. 开始爬取
    let start_time = std::time::Instant::now();

    match fetcher.fetch_all().await {
        Ok(rooms) => {
            let duration = start_time.elapsed();

            tracing::info!(
                "✅ 爬取完成！共获取 {} 个房间，耗时 {:.2?}",
                rooms.len(),
                duration
            );

            // 5. 输出到 JSON 文件（I/O优化）
            save_to_json(&rooms).await?;

            // 6. 输出统计信息
            print_statistics(&rooms, duration);

            tracing::info!("🎉 所有任务完成！");
        }
        Err(e) => {
            tracing::error!("❌ 爬取失败: {:?}", e);
            anyhow::bail!("爬取过程中发生错误: {e}");
        }
    }

    Ok(())
}

/// 初始化日志系统
fn init_tracing() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("room_fetcher=debug".parse()?))
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .init();

    Ok(())
}

/// 保存结果到 JSON 文件（I/O 优化版本）
async fn save_to_json(rooms: &[models::RoomInfo]) -> Result<()> {
    tracing::info!("💾 正在保存结果到文件（I/O优化）...");

    // 确保输出目录存在
    tokio::fs::create_dir_all("output").await?;

    // 序列化为格式化的 JSON（使用 SIMD 优化）
    let json = simd_json::to_string_pretty(rooms)?;

    // 使用异步文件写入 + 缓冲优化
    let output_path = "output/rooms.json";
    let file = File::create(output_path).await?;
    let mut writer = TokioBufWriter::new(file);
    
    writer.write_all(json.as_bytes()).await?;
    writer.flush().await?;

    tracing::info!("✓ 结果已保存到: {}（I/O优化）", output_path);

    Ok(())
}

/// 打印统计信息
fn print_statistics(rooms: &[models::RoomInfo], duration: std::time::Duration) {
    println!("\n{}", "=".repeat(60));
    println!("📈 爬取统计信息");
    println!("{}", "=".repeat(60));
    println!("✅ 总房间数:     {}", rooms.len());
    println!("⏱️  总耗时:       {duration:.2?}");

    if !rooms.is_empty() {
        #[allow(clippy::cast_precision_loss)]
        let avg_time = duration.as_secs_f64() / rooms.len() as f64;
        println!("📊 平均速度:     {avg_time:.2} 秒/房间");
        println!("🚀 吞吐量:       {:.2} 房间/秒", 1.0 / avg_time);
    }

    println!("{}", "=".repeat(60));

    // 打印前几个示例
    if !rooms.is_empty() {
        println!("\n📋 示例数据（前 5 个）：");
        for (idx, room) in rooms.iter().take(5).enumerate() {
            println!("  {}. {} (ID: {})", idx + 1, room.roompath, room.roomid);
        }

        if rooms.len() > 5 {
            println!("  ... 还有 {} 个房间", rooms.len() - 5);
        }
    }

    println!();
}
