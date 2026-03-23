//! RoomAggregate聚合根
//!
//! 将Room和其所有RoomPath组合成一个聚合根，用于完整的房间信息表示

use serde::{Deserialize, Serialize};

use super::{Room, RoomPath};

/// RoomAggregate聚合根
///
/// 包含房间基本信息和所有关联的路径信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomAggregate {
    /// 房间基本信息
    pub room: Room,

    /// 额外的房间路径列表（不包含primary_roompath）
    pub additional_paths: Vec<RoomPath>,
}

impl RoomAggregate {
    /// 创建新的聚合根
    pub fn new(room: Room, additional_paths: Vec<RoomPath>) -> Self {
        Self {
            room,
            additional_paths,
        }
    }

    /// 获取所有路径（包含primary和additional）
    pub fn all_roompaths(&self) -> Vec<String> {
        let mut paths = vec![self.room.primary_roompath.clone()];
        paths.extend(self.additional_paths.iter().map(|p| p.roompath.clone()));
        paths
    }

    /// 获取路径总数
    pub fn total_path_count(&self) -> usize {
        1 + self.additional_paths.len()
    }

    /// 检查是否有额外路径
    pub fn has_additional_paths(&self) -> bool {
        !self.additional_paths.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_room() -> Room {
        Room {
            id: Uuid::new_v4(),
            roomid: 123,
            electricity_fee: 50.0,
            send_flag: false,
            threshold: 100.0,
            room_name: "测试房间".to_string(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
            primary_roompath: "桂林/雁山/05栋/0501".to_string(),
            primary_roompath_hash: 12345,
            has_additional_paths: false,
            is_active: true,
            source_type: "manual".to_string(),
            external_id: None,
            last_synced_at: None,
            last_recovered_at: None,
        }
    }

    fn create_test_path(roomid: i32, roompath: String) -> RoomPath {
        RoomPath {
            id: Uuid::new_v4(),
            roomid,
            roompath,
            roompath_hash: 67890,
            room_name: "测试房间".to_string(),
            source_type: "api_sync".to_string(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }

    #[test]
    fn test_aggregate_creation() {
        let room = create_test_room();
        let paths = vec![create_test_path(
            123,
            "桂林/雁山/05栋/05楼/0501".to_string(),
        )];

        let aggregate = RoomAggregate::new(room, paths);

        assert_eq!(aggregate.total_path_count(), 2);
        assert!(aggregate.has_additional_paths());
    }

    #[test]
    fn test_all_roompaths() {
        let room = create_test_room();
        let paths = vec![
            create_test_path(123, "桂林/雁山/05栋/05楼/0501".to_string()),
            create_test_path(123, "广西/桂林/雁山/05栋/0501".to_string()),
        ];

        let aggregate = RoomAggregate::new(room, paths);
        let all_paths = aggregate.all_roompaths();

        assert_eq!(all_paths.len(), 3);
        assert_eq!(all_paths[0], "桂林/雁山/05栋/0501");
    }

    #[test]
    fn test_no_additional_paths() {
        let room = create_test_room();
        let aggregate = RoomAggregate::new(room, vec![]);

        assert_eq!(aggregate.total_path_count(), 1);
        assert!(!aggregate.has_additional_paths());
    }
}
