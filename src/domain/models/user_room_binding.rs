//! 用户-房间绑定领域模型

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

/// 用户-房间绑定实体
#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::infrastructure::database::schema::user_room_bindings)]
pub struct UserRoomBinding {
    /// 绑定ID (UUID)
    pub id: Uuid,
    
    /// 用户ID (外键)
    pub user_id: Uuid,
    
    /// 房间ID (外键)
    pub roomid: i32,
    
    /// 是否启用通知
    pub notification_enabled: bool,
    
    /// 创建时间
    pub created_at: NaiveDateTime,
    
    /// 更新时间
    pub updated_at: NaiveDateTime,
}

/// 新建用户-房间绑定DTO
#[derive(Debug, Insertable, Deserialize, Validate)]
#[diesel(table_name = crate::infrastructure::database::schema::user_room_bindings)]
pub struct NewUserRoomBinding {
    /// 用户ID
    pub user_id: Uuid,
    
    /// 房间ID
    pub roomid: i32,
    
    /// 是否启用通知 (默认: true)
    #[serde(default = "default_true")]
    pub notification_enabled: bool,
}

/// 更新通知开关DTO
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = crate::infrastructure::database::schema::user_room_bindings)]
pub struct UpdateNotificationEnabled {
    /// 通知开关
    pub notification_enabled: bool,
}

/// 默认值为 true
fn default_true() -> bool {
    true
}

/// 用户房间绑定列表响应DTO
#[derive(Debug, Serialize)]
pub struct UserRoomBindingWithRoomInfo {
    /// 绑定信息
    #[serde(flatten)]
    pub binding: UserRoomBinding,
    
    /// 房间名称（可选，联表查询时填充）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_name: Option<String>,
    
    /// 当前电费（可选，联表查询时填充）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub electricity_fee: Option<f32>,
    
    /// 电费阈值（可选，联表查询时填充）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_binding_defaults() {
        let binding = NewUserRoomBinding {
            user_id: Uuid::new_v4(),
            roomid: 101,
            notification_enabled: default_true(),
        };
        
        assert!(binding.notification_enabled);
    }

    #[test]
    fn test_binding_validation() {
        let binding = NewUserRoomBinding {
            user_id: Uuid::new_v4(),
            roomid: 101,
            notification_enabled: true,
        };
        
        assert!(binding.validate().is_ok());
    }
}
