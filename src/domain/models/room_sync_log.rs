//! 房间同步日志模型

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::infrastructure::database::schema::room_sync_log;

/// 同步日志实体
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = room_sync_log)]
pub struct RoomSyncLog {
    pub id: Uuid,
    pub sync_type: String,
    pub started_at: NaiveDateTime,
    pub completed_at: Option<NaiveDateTime>,
    pub status: String,
    pub stats: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub created_at: NaiveDateTime,
}

/// 创建同步日志的DTO
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = room_sync_log)]
pub struct NewRoomSyncLog {
    pub id: Option<Uuid>, // 可选ID，允许手动设置或自动生成
    pub sync_type: String,
    pub started_at: NaiveDateTime,
    pub status: String,
}

/// 更新同步日志的DTO
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = room_sync_log)]
pub struct UpdateRoomSyncLog {
    pub completed_at: Option<NaiveDateTime>,
    pub status: Option<String>,
    pub stats: Option<serde_json::Value>,
    pub error_message: Option<String>,
}
