//! Room领域模型

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::infrastructure::database::schema::rooms;

/// Room实体
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = rooms)]
pub struct Room {
    /// UUID主键
    pub id: Uuid,

    /// 房间ID（唯一业务标识）
    #[serde(with = "crate::utils::roomid")]
    pub roomid: i64,

    /// 当前电费
    pub electricity_fee: f32,

    /// 发送标志（超过阈值时自动设置为true）
    pub send_flag: bool,

    /// 电费阈值
    pub threshold: f32,

    /// 房间名称
    pub room_name: String,

    /// 创建时间
    pub created_at: NaiveDateTime,

    /// 更新时间
    pub updated_at: NaiveDateTime,

    // === 同步相关字段 ===
    /// 主要房间路径（唯一标识）
    pub primary_roompath: String,

    /// 主要房间路径的哈希值（用于快速查询）
    pub primary_roompath_hash: i64,

    /// 是否有额外的路径（优化查询标志）
    pub has_additional_paths: bool,

    /// 是否激活（用于软删除）
    pub is_active: bool,

    /// 数据来源类型（manual/api_sync/crawler等）
    pub source_type: String,

    /// 外部系统ID（可选）
    pub external_id: Option<String>,

    /// 最后同步时间
    pub last_synced_at: Option<NaiveDateTime>,

    /// 最后恢复时间（电费恢复到阈值以上的时间）
    ///
    /// 用于通知防抖逻辑，记录房间电费恢复到阈值以上的时间点。
    /// 当房间电费恢复后，需要等待防抖观察期才能重置通知状态。
    /// 服务器重启后可从数据库恢复此状态。
    pub last_recovered_at: Option<NaiveDateTime>,
}

/// 创建新房间的DTO
#[derive(Debug, Clone, Insertable, Deserialize, Validate)]
#[diesel(table_name = rooms)]
pub struct NewRoom {
    /// 房间ID
    #[serde(deserialize_with = "crate::utils::roomid::deserialize")]
    pub roomid: i64,

    /// 初始电费（默认0.0）
    #[serde(default)]
    pub electricity_fee: f32,

    /// 电费阈值
    #[validate(range(min = 0.0, message = "阈值不能为负数"))]
    pub threshold: f32,

    /// 房间名称
    #[validate(length(min = 1, max = 64, message = "房间名称长度必须在1-64字符之间"))]
    pub room_name: String,

    // === 同步相关字段 ===
    /// 主要房间路径（必填）
    #[validate(length(min = 1, max = 255, message = "房间路径长度必须在1-255字符之间"))]
    pub primary_roompath: String,

    /// 主要房间路径的哈希值
    pub primary_roompath_hash: i64,

    /// 是否有额外的路径（默认false）
    #[serde(default)]
    pub has_additional_paths: bool,

    /// 是否激活（默认true）
    #[serde(default = "default_true")]
    pub is_active: bool,

    /// 数据来源类型（默认"manual"）
    #[serde(default = "default_source_type")]
    pub source_type: String,

    /// 外部系统ID（可选）
    #[serde(default)]
    pub external_id: Option<String>,

    /// 最后同步时间（创建时不需要）
    #[serde(default)]
    pub last_synced_at: Option<NaiveDateTime>,

    /// 最后恢复时间（创建时不需要）
    #[serde(default)]
    pub last_recovered_at: Option<NaiveDateTime>,
}

// === 默认值函数 ===

fn default_true() -> bool {
    true
}

fn default_source_type() -> String {
    "manual".to_string()
}

/// 更新电费的DTO
#[derive(Debug, Clone, AsChangeset, Deserialize)]
#[diesel(table_name = rooms)]
pub struct UpdateElectricityFee {
    /// 新的电费值
    pub electricity_fee: f32,
}

/// 更新阈值的DTO
#[derive(Debug, Clone, AsChangeset, Deserialize, Validate)]
#[diesel(table_name = rooms)]
pub struct UpdateThreshold {
    /// 新的阈值
    #[validate(range(min = 0.0, message = "阈值不能为负数"))]
    pub threshold: f32,
}

/// 重置send_flag的DTO
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = rooms)]
pub struct ResetSendFlag {
    /// 重置为false
    pub send_flag: bool,
}

impl ResetSendFlag {
    /// 创建重置send_flag的实例
    pub fn new() -> Self {
        Self { send_flag: false }
    }
}

impl Default for ResetSendFlag {
    fn default() -> Self {
        Self::new()
    }
}

/// 更新最后恢复时间DTO
///
/// 用于在房间电费恢复到阈值以上时更新恢复时间，
/// 实现防抖状态的持久化存储。
#[derive(Debug, Clone, AsChangeset)]
#[diesel(table_name = rooms)]
pub struct UpdateLastRecovered {
    /// 最后恢复时间
    pub last_recovered_at: Option<NaiveDateTime>,
}
