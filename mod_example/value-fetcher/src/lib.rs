//! # 电费监控系统 - Rust 库
//!
//! 高性能、低内存的异步电费监控 Rust 库，提供简洁的 API 和内存优化设计。
//!
//! ## 快速开始
//!
//! ### 使用 prelude（推荐）⭐⭐⭐
//!
//! ```no_run
//! use electricity_monitor::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let fetcher = ElectricityFetcher::new("https://api.com?roomid=")?;
//!     let result = fetcher.fetch(&[3243, 3244, 3245]).await?;
//!
//!     // 使用便捷方法
//!     println!("成功: {}/{}", result.success_count(), result.total_count());
//!     println!("成功率: {:.1}%", result.success_rate() * 100.0);
//!
//!     // 迭代错误（自动获取描述）
//!     for (room_id, desc) in result.iter_errors() {
//!         println!("房间 {} 失败: {}", room_id, desc);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ### 完整示例
//!
//! ```no_run
//! use electricity_monitor::{ElectricityFetcher, ErrorCode};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. 创建查询器
//!     let fetcher = ElectricityFetcher::new(
//!         "https://api.example.com/query?roomid="
//!     )?;
//!
//!     // 2. 批量查询（房间 ID 使用 u16 类型）
//!     let room_ids = vec![3243, 3244, 3245];
//!     let result = fetcher.fetch(&room_ids).await?;
//!
//!     // 3. 处理成功结果
//!     for (room_id, fee) in &result.success {
//!         println!("房间 {}: {:.2} 元", room_id, fee);
//!     }
//!
//!     // 4. 处理失败结果
//!     for (room_id, error_code) in &result.failures {
//!         let desc = ErrorCode::from_u8(*error_code)
//!             .map(|ec| ec.description())
//!             .unwrap_or("未知错误");
//!         println!("房间 {} 失败: {}", room_id, desc);
//!     }
//!
//!     println!("成功率: {:.1}%", result.success_rate() * 100.0);
//!     Ok(())
//! }
//! ```
//!
//! ## 核心特性
//!
//! - ⚡ **简洁 API**: 仅需 `new()` + `fetch()`，无需手动配置
//! - 🚀 **智能路由**: 自动选择批量/流式模式（≤2000 批量，>2000 流式）
//! - 💾 **内存优化**: 对比传统设计节省 **83.9%** 内存 ⭐⭐⭐
//! - ⚡ **性能优化**: 并发 50（验证最优），FastPrefix 加速（15倍）
//! - 🔧 **错误处理**: 每个房间独立错误，不影响其他查询
//! - 🎯 **错误码系统**: u8 整数错误码 + 零开销查询
//! - 🛡️ **类型安全**: 编译时类型检查，消除运行时错误
//!
//! ## 内存优化设计
//!
//! 使用更小的数据类型以减少内存占用：
//!
//! - **房间 ID**: u16（2 字节）vs 传统 u32（4 字节），节省 50%
//! - **电费值**: f32（4 字节）vs 传统 f64（8 字节），节省 50%
//! - **错误码**: u8（1 字节）vs 传统 Result（32 字节），节省 96.9%
//!
//! **实测数据**（基准测试）：
//! - 100 房间: 0.58 KB（对比传统设计节省 **83.4%**）
//! - 1000 房间: 5.68 KB（对比传统设计节省 **83.9%**）⭐⭐⭐
//!
//! ## 性能指标
//!
//! | 指标 | 数值 |
//! |------|------|
//! | 并发数 | 50（经调优验证） |
//! | 超时 | 8 秒 |
//! | 吞吐量 | ~140 请求/秒 |
//! | URL 构建 | FastPrefix ~37ns, Generic ~549ns |
//! | 预估性能 | 10,000 房间约 1.2 分钟 |
//!
//! ## 配置文件示例
//!
//! ```ini
//! [electric_charge]
//! url_prefix = https://zywxhd02.gxust.edu.cn/Home/GetRoomInfo?roomid=
//! ```
//!
//! ## 架构设计
//!
//! - **Facade 模式**: `ElectricityFetcher` 封装所有内部实现
//! - **策略模式**: 智能选择批量/流式执行模式
//! - **错误隔离**: 单个失败不影响整体查询
//! - **性能优化**: FastPrefix URL 构建、连接池、并发控制

#![warn(missing_docs)]
#![warn(clippy::all)]

/// 错误处理模块
pub mod error;

/// 配置加载模块（内部使用，示例中可见）
#[doc(hidden)]
pub mod config;

/// 电费获取模块（核心业务）
pub mod fetcher;

/// 内部实现模块（完全隐藏）
mod internal;

// ============================================================
// 预导入模块（Prelude）
// ============================================================

/// 预导入模块
///
/// 包含所有常用类型，方便快速开始。
///
/// # 使用示例
///
/// ```no_run
/// use electricity_monitor::prelude::*;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // 现在可以直接使用所有常用类型
/// let fetcher = ElectricityFetcher::new("https://api.com?roomid=")?;
/// let result = fetcher.fetch(&[3243, 3244]).await?;
///
/// println!("成功率: {:.1}%", result.success_rate() * 100.0);
/// # Ok(())
/// # }
/// ```
pub mod prelude {
    //! 预导入模块，包含所有常用类型

    pub use crate::ElectricityFetcher;
    pub use crate::ErrorCode;
    pub use crate::FetchError;
    pub use crate::FetchResult;
}

// ============================================================
// 类型别名（Type Aliases）
// ============================================================

/// 房间 ID 类型别名
///
/// 使用 u16 以节省内存（范围 0-65535）
pub type RoomId = u16;

/// 电费值类型别名
///
/// 使用 f32 以节省内存，精度足够（约 6-9 位有效数字）
pub type ElectricityFee = f32;

// ============================================================
// 公开 API（库的主要接口）
// ============================================================

/// 电费查询器（主要接口）
///
/// 使用示例：
/// ```no_run
/// use electricity_monitor::ElectricityFetcher;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fetcher = ElectricityFetcher::new("https://api.com?roomid=")?;
/// let results = fetcher.fetch(&[3243, 3244, 3245]).await?;
/// # Ok(())
/// # }
/// ```
pub use fetcher::ElectricityFetcher;

/// 批量查询结果
///
/// 分离成功和失败的查询结果，内存优化版本。
///
/// 使用示例：
/// ```no_run
/// use electricity_monitor::{ElectricityFetcher, FetchResult};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let fetcher = ElectricityFetcher::new("https://api.com?roomid=")?;
/// let result: FetchResult = fetcher.fetch(&[3243, 3244]).await?;
/// println!("成功: {}/{}", result.success_count(), result.total_count());
/// # Ok(())
/// # }
/// ```
pub use fetcher::FetchResult;

/// 错误类型
pub use error::FetchError;

/// 错误码枚举
///
/// 将 `FetchError` 映射为简单的整数错误码（u8）。
///
/// 使用示例：
/// ```no_run
/// use electricity_monitor::{ErrorCode, FetchResult};
///
/// # let result = FetchResult::new();
/// for (room_id, error_code) in &result.failures {
///     if let Some(ec) = ErrorCode::from_u8(*error_code) {
///         println!("房间 {} 失败: {}", room_id, ec.description());
///     }
/// }
/// ```
pub use error::ErrorCode;

// ============================================================
// 内部类型（仅用于示例程序，标记为隐藏）
// ============================================================

#[doc(hidden)]
pub use config::ConfigLoader;
#[doc(hidden)]
pub use error::{ElectricityError, Result};
