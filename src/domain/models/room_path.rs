//! RoomPath领域模型
//!
//! 用于1:N房间路径映射的扩展表

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::infrastructure::database::schema::room_paths;

/// RoomPath实体（房间路径扩展表）
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = room_paths)]
pub struct RoomPath {
    /// UUID主键
    pub id: Uuid,

    /// 房间ID（外键，关联rooms.roomid）
    #[serde(with = "crate::utils::roomid")]
    pub roomid: i64,

    /// 房间路径（唯一标识）
    pub roompath: String,

    /// 房间路径的哈希值
    pub roompath_hash: i64,

    /// 房间名称
    pub room_name: String,

    /// 数据来源类型
    pub source_type: String,

    /// 创建时间
    pub created_at: NaiveDateTime,

    /// 更新时间
    pub updated_at: NaiveDateTime,
}

/// 创建新RoomPath的DTO
#[derive(Debug, Clone, Insertable, Deserialize)]
#[diesel(table_name = room_paths)]
pub struct NewRoomPath {
    /// 房间ID
    #[serde(deserialize_with = "crate::utils::roomid::deserialize")]
    pub roomid: i64,

    /// 房间路径
    pub roompath: String,

    /// 房间路径的哈希值
    pub roompath_hash: i64,

    /// 房间名称
    pub room_name: String,

    /// 数据来源类型（默认"api_sync"）
    #[serde(default = "default_source_type")]
    pub source_type: String,
}

fn default_source_type() -> String {
    "api_sync".to_string()
}

impl NewRoomPath {
    /// 创建新的RoomPath实例
    pub fn new(roomid: i64, roompath: String, roompath_hash: i64, room_name: String) -> Self {
        Self {
            roomid,
            roompath,
            roompath_hash,
            room_name,
            source_type: default_source_type(),
        }
    }
}
