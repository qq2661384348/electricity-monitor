//! 用户领域模型

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 用户实体
#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::infrastructure::database::schema::users)]
pub struct User {
    /// 用户ID (UUID)
    pub id: Uuid,

    /// QQ号（唯一）
    pub qq_number: String,

    /// 用户角色 (admin/user)
    pub role: String,

    /// 是否激活
    pub is_active: bool,

    /// 创建时间
    pub created_at: NaiveDateTime,

    /// 更新时间
    pub updated_at: NaiveDateTime,
}

/// 新建用户DTO
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::infrastructure::database::schema::users)]
pub struct NewUser {
    /// QQ号
    pub qq_number: String,

    /// 用户角色 (默认: "user")
    #[serde(default = "default_user_role")]
    pub role: String,

    /// 是否激活 (默认: true)
    #[serde(default = "default_true")]
    pub is_active: bool,
}

/// 更新用户角色DTO
#[derive(Debug, AsChangeset, Deserialize)]
#[diesel(table_name = crate::infrastructure::database::schema::users)]
pub struct UpdateUserRole {
    /// 新角色
    pub role: String,
}

/// 默认角色为 "user"
fn default_user_role() -> String {
    "user".to_string()
}

/// 默认值为 true
fn default_true() -> bool {
    true
}

impl User {
    /// 检查用户是否为管理员
    pub fn is_admin(&self) -> bool {
        self.role == "admin"
    }

    /// 检查用户是否为普通用户
    pub fn is_user(&self) -> bool {
        self.role == "user"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_admin() {
        let admin = User {
            id: Uuid::new_v4(),
            qq_number: "123456".to_string(),
            role: "admin".to_string(),
            is_active: true,
            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
        };

        assert!(admin.is_admin());
        assert!(!admin.is_user());
    }

    #[test]
    fn test_is_user() {
        let user = User {
            id: Uuid::new_v4(),
            qq_number: "123456".to_string(),
            role: "user".to_string(),
            is_active: true,
            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
        };

        assert!(!user.is_admin());
        assert!(user.is_user());
    }

    #[test]
    fn test_new_user_defaults() {
        let new_user = NewUser {
            qq_number: "123456".to_string(),
            role: default_user_role(),
            is_active: default_true(),
        };

        assert_eq!(new_user.role, "user");
        assert!(new_user.is_active);
    }
}
