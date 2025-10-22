//! 业务示例：批量查询 6000 房间电费并输出日志
//!
//! 功能：
//! - 批量查询房间 1-6000
//! - 输出成功结果到 ./tests/sus.log
//! - 输出失败结果到 ./tests/err.log
//!
//! 设计特点：
//! - 优雅的错误处理
//! - 结构化日志输出
//! - 进度反馈
//! - 统计信息汇总

use electricity_monitor::{ElectricityFetcher, ErrorCode};
use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

/// 日志输出器（封装文件写入逻辑）
struct LogWriter {
    success_writer: BufWriter<File>,
    error_writer: BufWriter<File>,
}

impl LogWriter {
    /// 创建日志输出器
    fn new(success_path: &str, error_path: &str) -> std::io::Result<Self> {
        // 确保目录存在
        if let Some(parent) = Path::new(success_path).parent() {
            create_dir_all(parent)?;
        }

        Ok(Self {
            success_writer: BufWriter::new(File::create(success_path)?),
            error_writer: BufWriter::new(File::create(error_path)?),
        })
    }

    /// 写入成功记录
    fn write_success(&mut self, room_id: u16, fee: f32) -> std::io::Result<()> {
        writeln!(self.success_writer, "{}:{:.2}", room_id, fee)
    }

    /// 写入失败记录
    fn write_error(&mut self, room_id: u16, description: &str) -> std::io::Result<()> {
        writeln!(self.error_writer, "{}:{}", room_id, description)
    }

    /// 刷新缓冲区
    fn flush(&mut self) -> std::io::Result<()> {
        self.success_writer.flush()?;
        self.error_writer.flush()?;
        Ok(())
    }
}

/// 业务执行器
struct BusinessExecutor {
    fetcher: ElectricityFetcher,
    log_writer: LogWriter,
}

impl BusinessExecutor {
    /// 创建业务执行器
    fn new(url_prefix: &str) -> Result<Self, Box<dyn std::error::Error>> {
        println!("🔧 初始化业务执行器...");

        let fetcher = ElectricityFetcher::new(url_prefix)?;
        let log_writer = LogWriter::new("./tests/sus.log", "./tests/err.log")?;

        println!("✅ 初始化完成");
        println!("   - URL 前缀: {}", url_prefix);
        println!("   - 成功日志: ./tests/sus.log");
        println!("   - 失败日志: ./tests/err.log");
        println!();

        Ok(Self {
            fetcher,
            log_writer,
        })
    }

    /// 执行批量查询和日志输出
    async fn execute(
        &mut self,
        start_id: u16,
        end_id: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let total_count = (end_id - start_id + 1) as usize;
        println!(
            "📊 开始查询 {} 个房间（{} - {}）",
            total_count, start_id, end_id
        );
        println!();

        // 准备房间 ID 列表
        let room_ids: Vec<u16> = (start_id..=end_id).collect();

        // 执行查询
        println!("🚀 正在查询...");
        let start_time = Instant::now();
        let result = self.fetcher.fetch(&room_ids).await?;
        let query_duration = start_time.elapsed();

        println!("✅ 查询完成，耗时 {:?}", query_duration);
        println!();

        // 输出统计信息
        self.print_statistics(&result, total_count, query_duration);

        // 写入日志
        println!("📝 正在写入日志...");
        self.write_logs(&result)?;
        println!("✅ 日志写入完成");
        println!();

        Ok(())
    }

    /// 打印统计信息
    fn print_statistics(
        &self,
        result: &electricity_monitor::FetchResult,
        total_count: usize,
        duration: std::time::Duration,
    ) {
        println!("=== 📈 统计信息 ===");
        println!("总房间数: {}", total_count);
        println!(
            "成功数量: {} ({:.1}%)",
            result.success_count(),
            result.success_rate() * 100.0
        );
        println!(
            "失败数量: {} ({:.1}%)",
            result.failure_count(),
            (1.0 - result.success_rate()) * 100.0
        );
        println!("查询耗时: {:?}", duration);
        println!(
            "吞吐量: {:.1} 请求/秒",
            total_count as f64 / duration.as_secs_f64()
        );
        println!();
    }

    /// 写入日志文件
    fn write_logs(&mut self, result: &electricity_monitor::FetchResult) -> std::io::Result<()> {
        let start_time = Instant::now();

        // 写入成功记录
        for (room_id, fee) in &result.success {
            self.log_writer.write_success(*room_id, *fee)?;
        }

        // 写入失败记录
        for (room_id, error_code) in &result.failures {
            let description = ErrorCode::from_u8(*error_code)
                .map(|ec| ec.description())
                .unwrap_or("未知错误");
            self.log_writer.write_error(*room_id, description)?;
        }

        // 刷新缓冲区
        self.log_writer.flush()?;

        let write_duration = start_time.elapsed();
        println!("   - 成功记录: {} 条", result.success_count());
        println!("   - 失败记录: {} 条", result.failure_count());
        println!("   - 写入耗时: {:?}", write_duration);

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 🏢 电费监控业务示例 ===");
    println!();

    // 配置参数
    let url_prefix = "https://zywxhd02.gxust.edu.cn/Home/GetRoomInfo?roomid=";
    let start_room_id = 1u16;
    let end_room_id = 6000u16;

    // 创建业务执行器
    let mut executor = BusinessExecutor::new(url_prefix)?;

    // 执行业务逻辑
    executor.execute(start_room_id, end_room_id).await?;

    // 最终总结
    println!("=== 🎉 业务执行完成 ===");
    println!("✅ 成功日志: ./tests/sus.log");
    println!("✅ 失败日志: ./tests/err.log");
    println!();
    println!("提示：可以使用以下命令查看日志内容：");
    println!("  - cat tests/sus.log | head -n 10");
    println!("  - cat tests/err.log | head -n 10");
    println!("  - wc -l tests/sus.log tests/err.log");

    Ok(())
}
