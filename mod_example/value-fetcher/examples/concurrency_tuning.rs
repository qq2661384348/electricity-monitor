//! 并发参数调优测试
//!
//! 测试不同并发参数对性能的影响，找出最优配置

use electricity_monitor::{
    config::ConfigLoader,
    fetcher::RoomBatchFetcher,
    infrastructure::{ElectricityParser, ReqwestAsyncClient},
};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 并发参数调优测试\n");
    println!("{}", "=".repeat(60));

    // 1. 加载配置
    let config = ConfigLoader::from_file("config.ini")?;
    let template_url = config
        .get("electric_charge", "url")
        .ok_or("配置文件中未找到 URL")?;

    // 提取基础房间ID
    let base_id = extract_roomid(&template_url)
        .and_then(|id| id.parse::<u32>().ok())
        .unwrap_or(3240);

    // 2. 测试不同并发参数
    let concurrency_levels = vec![20, 50, 100];
    let test_size = 200; // 测试200个房间

    for &concurrency in &concurrency_levels {
        println!("\n📊 测试并发数: {}", concurrency);
        println!("{}", "-".repeat(60));

        // 创建获取器
        let http_client = ReqwestAsyncClient::new(true)?;
        let parser = ElectricityParser::new()?;
        let fetcher =
            RoomBatchFetcher::new(template_url.clone(), http_client, parser, concurrency)?;

        // 准备房间ID列表
        let room_ids: Vec<u32> = (base_id..base_id + test_size).collect();

        // 执行测试（运行3次取平均）
        let mut total_time = std::time::Duration::ZERO;
        let mut total_success = 0;

        for round in 1..=3 {
            let start = Instant::now();
            let results = fetcher.fetch_batch_ids(room_ids.clone()).await;
            let duration = start.elapsed();

            let success = results.iter().filter(|r| r.electricity.is_some()).count();
            total_time += duration;
            total_success += success;

            println!(
                "  第 {} 轮: {:?} | 成功率: {}/{} ({:.1}%)",
                round,
                duration,
                success,
                test_size,
                (success as f64 / test_size as f64) * 100.0
            );
        }

        let avg_time = total_time / 3;
        let avg_success = total_success / 3;
        let avg_per_room = avg_time / test_size;

        println!("\n  ✅ 平均结果:");
        println!("     总耗时: {:?}", avg_time);
        println!(
            "     成功数: {}/{} ({:.1}%)",
            avg_success,
            test_size,
            (avg_success as f64 / test_size as f64) * 100.0
        );
        println!("     单房间: {:?}", avg_per_room);
        println!(
            "     吞吐量: {:.1} 请求/秒",
            test_size as f64 / avg_time.as_secs_f64()
        );
    }

    println!("\n{}", "=".repeat(60));
    println!("📈 调优建议:");
    println!("  - 并发过低: 吞吐量不足，无法充分利用网络带宽");
    println!("  - 并发过高: 可能触发服务端限流，成功率下降");
    println!("  - 最优并发: 在成功率和吞吐量之间取得平衡");

    Ok(())
}

/// 从 URL 中提取 roomid
fn extract_roomid(url: &str) -> Option<String> {
    url.split(&['?', '&'])
        .find(|part| part.starts_with("roomid="))
        .and_then(|part| part.split('=').nth(1))
        .map(|id| id.to_string())
}
