//! 电费历史记录模型

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::infrastructure::database::schema::electricity_history;

/// 电费历史记录实体
///
/// 记录房间的电费历史数据，用于统计分析
#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = electricity_history)]
pub struct ElectricityHistory {
    /// 唯一标识符
    pub id: Uuid,
    /// 房间ID
    pub roomid: i32,
    /// 电费值
    pub electricity_fee: f32,
    /// 记录时间（业务时间）
    pub recorded_at: NaiveDateTime,
    /// 创建时间（系统时间）
    pub created_at: NaiveDateTime,
}

/// 新建电费历史记录
#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = electricity_history)]
pub struct NewElectricityHistory {
    /// 房间ID
    pub roomid: i32,
    /// 电费值
    pub electricity_fee: f32,
    /// 记录时间
    pub recorded_at: NaiveDateTime,
}

impl NewElectricityHistory {
    /// 创建新的历史记录
    ///
    /// # 参数
    /// - `roomid`: 房间ID
    /// - `electricity_fee`: 电费值
    /// - `recorded_at`: 记录时间
    pub fn new(roomid: i32, electricity_fee: f32, recorded_at: NaiveDateTime) -> Self {
        Self {
            roomid,
            electricity_fee,
            recorded_at,
        }
    }
}
