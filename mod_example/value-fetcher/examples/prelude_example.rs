//! Prelude 使用示例
//!
//! 展示如何使用 prelude 模块和新增的便捷方法

use electricity_monitor::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 📦 Prelude 使用示例 ===");
    println!();

    // 1. 创建查询器
    let url_prefix = "https://zywxhd02.gxust.edu.cn/Home/GetRoomInfo?roomid=";
    let fetcher = ElectricityFetcher::new(url_prefix)?;

    // 2. 查询一批房间
    let room_ids = vec![3243, 3244, 3245, 4286, 635, 1714];
    println!("🔍 查询房间: {:?}", room_ids);
    println!();

    let result = fetcher.fetch(&room_ids).await?;

    // 3. 使用便捷方法查看统计
    println!("=== 📊 统计信息 ===");
    println!("总数量: {}", result.total_count());
    println!("成功数: {}", result.success_count());
    println!("失败数: {}", result.failure_count());
    println!("成功率: {:.1}%", result.success_rate() * 100.0);
    println!();

    // 4. 遍历成功结果
    println!("=== ✅ 成功房间 ===");
    for room_id in result.success_ids() {
        if let Some(&fee) = result.success.get(&room_id) {
            if fee < 0.0 {
                println!("房间 {}: {:.2} 元 ⚠️ 欠费", room_id, fee);
            } else {
                println!("房间 {}: {:.2} 元", room_id, fee);
            }
        }
    }
    println!();

    // 5. 使用 iter_errors() 遍历失败（自动获取描述）
    println!("=== ❌ 失败房间 ===");
    for (room_id, desc) in result.iter_errors() {
        println!("房间 {}: {}", room_id, desc);
    }
    println!();

    // 6. 使用 Display trait 直接打印错误码
    println!("=== 🔍 错误码演示 ===");
    if let Some(&error_code) = result.failures.get(&4286) {
        if let Some(ec) = ErrorCode::from_u8(error_code) {
            // 直接使用 {} 格式化（得益于 Display trait）
            println!("房间 4286 错误码: {} (code: {})", ec, ec.as_u8());
        }
    }
    println!();

    // 7. 判断查询结果状态
    println!("=== 📋 查询状态 ===");
    if result.is_all_success() {
        println!("✅ 全部成功！");
    } else if result.is_all_failed() {
        println!("❌ 全部失败！");
    } else {
        println!("⚠️ 部分成功，部分失败");
    }

    Ok(())
}
