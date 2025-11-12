//! 哈希工具函数
//! 
//! 提供一致的哈希计算功能，用于roompath的快速查询

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// 计算roompath的哈希值
/// 
/// 使用Rust标准库的DefaultHasher（SipHash-1-3），提供：
/// - 确定性：相同输入总是产生相同输出
/// - 性能：非加密哈希，优化速度
/// - 碰撞抵抗：足够的分布均匀性
/// 
/// # 参数
/// - `roompath`: 房间路径字符串
/// 
/// # 返回值
/// i64类型的哈希值，可直接存储到PostgreSQL的BIGINT字段
/// 
/// # 示例
/// ```
/// use electricity_monitor_backend::utils::hash::calculate_roompath_hash;
/// 
/// let hash1 = calculate_roompath_hash("桂林/雁山/05栋/0501");
/// let hash2 = calculate_roompath_hash("桂林/雁山/05栋/0501");
/// assert_eq!(hash1, hash2); // 确定性
/// ```
pub fn calculate_roompath_hash(roompath: &str) -> i64 {
    let mut hasher = DefaultHasher::new();
    roompath.hash(&mut hasher);
    hasher.finish() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_determinism() {
        let path = "桂林/雁山/05栋/0501";
        let hash1 = calculate_roompath_hash(path);
        let hash2 = calculate_roompath_hash(path);
        assert_eq!(hash1, hash2, "相同输入应产生相同哈希值");
    }

    #[test]
    fn test_hash_uniqueness() {
        let hash1 = calculate_roompath_hash("桂林/雁山/05栋/0501");
        let hash2 = calculate_roompath_hash("桂林/雁山/05栋/0502");
        assert_ne!(hash1, hash2, "不同输入应产生不同哈希值");
    }

    #[test]
    fn test_hash_empty_string() {
        let hash = calculate_roompath_hash("");
        assert_ne!(hash, 0, "空字符串也应产生非零哈希值");
    }

    #[test]
    fn test_hash_unicode() {
        let hash1 = calculate_roompath_hash("测试/路径/中文");
        let hash2 = calculate_roompath_hash("测试/路径/中文");
        assert_eq!(hash1, hash2, "Unicode字符串应正确哈希");
    }

    #[test]
    fn test_hash_case_sensitivity() {
        let hash1 = calculate_roompath_hash("Path/To/Room");
        let hash2 = calculate_roompath_hash("path/to/room");
        assert_ne!(hash1, hash2, "哈希应区分大小写");
    }
}
