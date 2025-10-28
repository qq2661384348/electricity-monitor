//! Room领域模型

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

use crate::infrastructure::database::schema::rooms;

/// Room实体
#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = rooms)]
pub struct Room {
    /// UUID主键
    pub id: Uuid,
    
    /// 房间ID（可能重复的整数ID）
    pub roomid: i32,
    
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
}

/// 创建新房间的DTO
#[derive(Debug, Clone, Insertable, Deserialize, Validate)]
#[diesel(table_name = rooms)]
pub struct NewRoom {
    /// 房间ID
    pub roomid: i32,
    
    /// 初始电费（默认0.0）
    #[serde(default)]
    pub electricity_fee: f32,
    
    /// 电费阈值
    #[validate(range(min = 0.0, message = "阈值不能为负数"))]
    pub threshold: f32,
    
    /// 房间名称
    #[validate(length(min = 1, max = 64, message = "房间名称长度必须在1-64字符之间"))]
    pub room_name: String,
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
