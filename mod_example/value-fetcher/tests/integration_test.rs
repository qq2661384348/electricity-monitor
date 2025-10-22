//! 集成测试
//!
//! 端到端测试电费查询系统的核心功能。

use electricity_monitor::{ElectricityFetcher, ErrorCode, FetchResult};

/// 测试 ElectricityFetcher 的基本创建
#[test]
fn test_fetcher_creation() {
    // 测试有效的 URL 前缀
    let fetcher = ElectricityFetcher::new("https://example.com?roomid=");
    assert!(fetcher.is_ok());

    // 测试另一个有效的 URL 前缀
    let fetcher2 = ElectricityFetcher::new("https://api.test.com/query?roomid=123");
    assert!(fetcher2.is_ok());
}

/// 测试无效的 URL 前缀
#[test]
fn test_invalid_url_prefix() {
    // 缺少协议
    let result = ElectricityFetcher::new("example.com?roomid=");
    assert!(result.is_err());

    // 缺少 roomid 参数
    let result = ElectricityFetcher::new("https://example.com?id=123");
    assert!(result.is_err());

    // 完全无效的 URL
    let result = ElectricityFetcher::new("not-a-url");
    assert!(result.is_err());
}

/// 测试 FetchResult 的基本功能
#[test]
fn test_fetch_result_basic() {
    let mut result = FetchResult::new();
    
    // 测试初始状态
    assert_eq!(result.total_count(), 0);
    assert_eq!(result.success_count(), 0);
    assert_eq!(result.failure_count(), 0);
    
    // 添加成功结果
    result.success.insert(3243, 121.5);
    result.success.insert(3244, 85.0);
    
    assert_eq!(result.success_count(), 2);
    assert_eq!(result.total_count(), 2);
    
    // 添加失败结果
    result.failures.insert(3245, 2); // NetworkError
    
    assert_eq!(result.failure_count(), 1);
    assert_eq!(result.total_count(), 3);
}

/// 测试 FetchResult 的统计方法
#[test]
fn test_fetch_result_statistics() {
    let mut result = FetchResult::new();
    
    result.success.insert(1, 100.0);
    result.success.insert(2, 200.0);
    result.failures.insert(3, 2);
    
    // 测试成功率
    let rate = result.success_rate();
    assert!((rate - 0.666).abs() < 0.01); // 2/3 ≈ 0.666
    
    // 测试状态判断
    assert!(!result.is_all_success());
    assert!(!result.is_all_failed());
}

/// 测试 FetchResult 的全部成功/失败判断
#[test]
fn test_fetch_result_all_status() {
    // 全部成功
    let mut result = FetchResult::new();
    result.success.insert(1, 100.0);
    assert!(result.is_all_success());
    assert!(!result.is_all_failed());
    
    // 全部失败
    let mut result = FetchResult::new();
    result.failures.insert(1, 2);
    assert!(!result.is_all_success());
    assert!(result.is_all_failed());
    
    // 空结果
    let result = FetchResult::new();
    assert!(!result.is_all_success());
    assert!(!result.is_all_failed());
}

/// 测试 ErrorCode 的基本功能
#[test]
fn test_error_code() {
    // 测试 from_u8
    let code = ErrorCode::from_u8(2);
    assert!(code.is_some());
    assert_eq!(code.unwrap(), ErrorCode::NetworkError);
    
    // 测试无效错误码
    let code = ErrorCode::from_u8(99);
    assert!(code.is_none());
    
    // 测试 as_u8
    assert_eq!(ErrorCode::NetworkError.as_u8(), 2);
    
    // 测试 description
    assert_eq!(ErrorCode::NetworkError.description(), "网络请求失败");
}

/// 测试 FetchResult 的错误描述获取
#[test]
fn test_fetch_result_error_description() {
    let mut result = FetchResult::new();
    result.failures.insert(3243, 2); // NetworkError
    result.failures.insert(3244, 3); // ParseError
    
    // 测试 get_error_description
    let desc = result.get_error_description(3243);
    assert_eq!(desc, Some("网络请求失败"));
    
    let desc = result.get_error_description(3244);
    assert_eq!(desc, Some("数据解析失败"));
    
    // 不存在的房间
    let desc = result.get_error_description(9999);
    assert_eq!(desc, None);
}

/// 测试 FetchResult 的迭代器方法
#[test]
fn test_fetch_result_iterators() {
    let mut result = FetchResult::new();
    result.success.insert(3245, 100.0);
    result.success.insert(3243, 200.0);
    result.failures.insert(3246, 2);
    result.failures.insert(3244, 3);
    
    // 测试 success_ids (应该排序)
    let ids = result.success_ids();
    assert_eq!(ids, vec![3243, 3245]);
    
    // 测试 failure_ids (应该排序)
    let ids = result.failure_ids();
    assert_eq!(ids, vec![3244, 3246]);
    
    // 测试 iter_errors
    let errors: Vec<_> = result.iter_errors().collect();
    assert_eq!(errors.len(), 2);
}

/// 测试 FetchResult 的容量预分配
#[test]
fn test_fetch_result_with_capacity() {
    let result = FetchResult::with_capacity(1000);
    
    // 应该能正常使用
    assert_eq!(result.total_count(), 0);
    
    // 预分配的容量不影响功能
    let _ = result.success;
    let _ = result.failures;
}

/// 测试空的房间 ID 列表
#[tokio::test]
async fn test_empty_room_ids() {
    let fetcher = ElectricityFetcher::new("https://example.com?roomid=").unwrap();
    let result = fetcher.fetch(&[]).await;
    
    assert!(result.is_ok());
    let result = result.unwrap();
    assert_eq!(result.total_count(), 0);
}

/// 测试 URL 前缀验证
#[test]
fn test_url_validation() {
    // 有效的 URL 前缀
    assert!(ElectricityFetcher::new("https://example.com?roomid=").is_ok());
    assert!(ElectricityFetcher::new("http://localhost:8080?roomid=").is_ok());
    assert!(ElectricityFetcher::new("https://api.com/path?roomid=123").is_ok());
    
    // 无效的 URL 前缀
    assert!(ElectricityFetcher::new("ftp://example.com?roomid=").is_err()); // 错误协议
    assert!(ElectricityFetcher::new("https://example.com").is_err()); // 缺少 roomid
    assert!(ElectricityFetcher::new("https://example.com?id=123").is_err()); // 参数名错误
}

/// 测试 ErrorCode 的显示特性
#[test]
fn test_error_code_display() {
    let code = ErrorCode::NetworkError;
    let display = format!("{}", code);
    assert_eq!(display, "网络请求失败");
    
    let code = ErrorCode::ParseError;
    let display = format!("{}", code);
    assert_eq!(display, "数据解析失败");
}

/// 测试 FetchResult 的成功率边界情况
#[test]
fn test_success_rate_edge_cases() {
    // 空结果
    let result = FetchResult::new();
    assert_eq!(result.success_rate(), 0.0);
    
    // 100% 成功
    let mut result = FetchResult::new();
    result.success.insert(1, 100.0);
    result.success.insert(2, 200.0);
    assert_eq!(result.success_rate(), 1.0);
    
    // 0% 成功
    let mut result = FetchResult::new();
    result.failures.insert(1, 2);
    result.failures.insert(2, 3);
    assert_eq!(result.success_rate(), 0.0);
}
