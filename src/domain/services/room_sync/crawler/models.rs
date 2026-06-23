//! 爬虫数据模型
//!
//! 定义爬虫模块的数据结构，支持1:N映射场景

use serde::{Deserialize, Serialize};

/// 新房间树接口的通用响应结构。
#[derive(Debug, Deserialize, Clone)]
pub struct UpayResponse<T> {
    #[serde(rename = "Total")]
    pub total: Option<i64>,

    #[serde(rename = "Data")]
    pub data: Option<Vec<T>>,
}

impl<T> UpayResponse<T> {
    pub fn into_data(self) -> Vec<T> {
        self.data.unwrap_or_default()
    }
}

/// 校区节点。
#[derive(Debug, Deserialize, Clone)]
pub struct SchoolComponent {
    #[serde(rename = "SchoolId")]
    pub school_id: Option<String>,

    #[serde(rename = "SchoolName")]
    pub school_name: Option<String>,
}

/// 楼栋节点。
#[derive(Debug, Deserialize, Clone)]
pub struct ApartmentComponent {
    #[serde(rename = "ApartID")]
    pub apart_id: Option<String>,

    #[serde(rename = "ApartName")]
    pub apart_name: Option<String>,
}

/// 房间节点。
#[derive(Debug, Deserialize, Clone)]
pub struct RoomListComponent {
    #[serde(rename = "RoomID")]
    pub room_id: Option<String>,

    #[serde(rename = "RoomName")]
    pub room_name: Option<String>,
}

/// 房间信息（最终输出结构）
///
/// 输出格式：
/// ```json
/// {
///     "roompath": "东环校区/东环北区12栋/三楼/B12313",
///     "roomid": "3241"
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomInfo {
    /// 房间完整路径（校区/楼栋/房间）
    pub roompath: String,

    /// 房间唯一标识符
    pub roomid: String,
}

/// 房间数据（最终输出结构）
///
/// 支持1:N映射：一个roomid可以对应多个roompath
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomData {
    /// 房间ID。外部接口可能返回超过 i32 的 18 位业务 ID，后端内部使用 i64 保存。
    pub roomid: i64,

    /// 所有房间路径（已去重和排序）
    pub roompaths: Vec<String>,

    /// 主要路径（roompaths中的第一个）
    pub primary_roompath: String,

    /// 路径数量
    pub path_count: usize,
}

impl RoomData {
    /// 创建新的RoomData实例
    ///
    /// # 参数
    /// - `roomid`: 房间ID
    /// - `roompaths`: 房间路径列表（会自动去重和排序）
    ///
    /// # 示例
    /// ```
    /// use electricity_monitor_backend::domain::services::room_sync::crawler::models::RoomData;
    ///
    /// let paths = vec![
    ///     "桂林/雁山/05栋/0501".to_string(),
    ///     "广西/桂林/雁山/05栋/0501".to_string(),
    ///     "桂林/雁山/05栋/0501".to_string(),  // 重复
    /// ];
    ///
    /// let room = RoomData::new(101, paths);
    /// assert_eq!(room.path_count, 2);  // 去重后
    /// assert_eq!(room.primary_roompath, "广西/桂林/雁山/05栋/0501");  // 排序后第一个
    /// ```
    pub fn new(roomid: i64, mut roompaths: Vec<String>) -> Self {
        // 去重和排序
        roompaths.sort();
        roompaths.dedup();

        // 取第一个作为主路径
        let primary_roompath = roompaths
            .first()
            .cloned()
            .unwrap_or_else(|| format!("未知路径/{}", roomid));

        let path_count = roompaths.len();

        Self {
            roomid,
            roompaths,
            primary_roompath,
            path_count,
        }
    }

    /// 判断是否有额外路径
    pub fn has_additional_paths(&self) -> bool {
        self.path_count > 1
    }
}

/// 合并统计信息
///
/// 记录1:N合并过程的统计数据
#[derive(Debug, Clone, Serialize)]
pub struct MergeStatistics {
    /// 原始记录数（合并前）
    pub raw_count: usize,

    /// 唯一roomid数量（合并后）
    pub unique_roomid_count: usize,

    /// 有多个路径的roomid数量
    pub multi_path_count: usize,

    /// 单个roomid最多路径数
    pub max_paths: usize,

    /// 平均路径数
    pub avg_paths: f64,

    /// roomid解析失败的数量
    pub parse_error_count: usize,
}

impl MergeStatistics {
    /// 计算统计信息
    pub fn calculate(raw_count: usize, merged: &[RoomData], parse_error_count: usize) -> Self {
        let unique_roomid_count = merged.len();
        let multi_path_count = merged.iter().filter(|r| r.path_count > 1).count();
        let max_paths = merged.iter().map(|r| r.path_count).max().unwrap_or(0);
        let total_paths: usize = merged.iter().map(|r| r.path_count).sum();
        let avg_paths = if unique_roomid_count > 0 {
            total_paths as f64 / unique_roomid_count as f64
        } else {
            0.0
        };

        Self {
            raw_count,
            unique_roomid_count,
            multi_path_count,
            max_paths,
            avg_paths,
            parse_error_count,
        }
    }

    /// 输出统计日志
    pub fn log(&self) {
        tracing::info!(
            "爬虫数据合并完成: 原始记录={}, 唯一roomid={}, 1:N场景={}, 最多路径={}, 平均路径={:.2}, 解析失败={}",
            self.raw_count,
            self.unique_roomid_count,
            self.multi_path_count,
            self.max_paths,
            self.avg_paths,
            self.parse_error_count
        );

        if self.parse_error_count > 0 {
            tracing::warn!(
                "存在{}条roomid解析失败的记录，占比{:.2}%",
                self.parse_error_count,
                (self.parse_error_count as f64 / self.raw_count as f64) * 100.0
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_data_new() {
        let paths = vec![
            "桂林/雁山/05栋/0501".to_string(),
            "广西/桂林/雁山/05栋/0501".to_string(),
            "桂林/雁山/05栋/0501".to_string(), // 重复
        ];

        let room = RoomData::new(101, paths);

        assert_eq!(room.roomid, 101);
        assert_eq!(room.path_count, 2); // 去重后
        assert_eq!(room.roompaths.len(), 2);
        assert_eq!(room.primary_roompath, "广西/桂林/雁山/05栋/0501"); // 排序后第一个
        assert!(room.has_additional_paths());
    }

    #[test]
    fn test_room_data_single_path() {
        let paths = vec!["桂林/雁山/05栋/0501".to_string()];
        let room = RoomData::new(101, paths);

        assert_eq!(room.path_count, 1);
        assert!(!room.has_additional_paths());
    }

    #[test]
    fn test_room_data_empty_paths() {
        let paths = vec![];
        let room = RoomData::new(101, paths);

        assert_eq!(room.path_count, 0);
        assert_eq!(room.primary_roompath, "未知路径/101");
    }

    #[test]
    fn test_merge_statistics_calculate() {
        let merged = vec![
            RoomData::new(101, vec!["path1".to_string()]),
            RoomData::new(102, vec!["path2".to_string(), "path3".to_string()]),
            RoomData::new(103, vec!["path4".to_string()]),
        ];

        let stats = MergeStatistics::calculate(10, &merged, 2);

        assert_eq!(stats.raw_count, 10);
        assert_eq!(stats.unique_roomid_count, 3);
        assert_eq!(stats.multi_path_count, 1); // 只有102有多个路径
        assert_eq!(stats.max_paths, 2);
        assert_eq!(stats.parse_error_count, 2);
        assert!((stats.avg_paths - 1.33).abs() < 0.01); // 4/3 ≈ 1.33
    }
}
