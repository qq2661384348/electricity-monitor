//! 用户领域模型

use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const LOGIN_PROVIDER_QQ: &str = "qq";
pub const LOGIN_PROVIDER_EMAIL: &str = "email";

/// 用户实体
#[derive(Debug, Clone, Queryable, Selectable, Serialize)]
#[diesel(table_name = crate::infrastructure::database::schema::users)]
pub struct User {
    /// 用户ID (UUID)
    pub id: Uuid,

    /// QQ号（QQ 登录账号唯一；邮箱登录账号为空）
    pub qq_number: Option<String>,

    /// 用户角色 (admin/user)
    pub role: String,

    /// 是否激活
    pub is_active: bool,

    /// 创建时间
    pub created_at: NaiveDateTime,

    /// 更新时间
    pub updated_at: NaiveDateTime,

    /// 登录渠道：qq / email
    pub login_provider: String,

    /// 邮箱地址（邮箱登录账号唯一；QQ 登录账号为空）
    pub email: Option<String>,
}

/// 新建用户DTO
#[derive(Debug, Insertable, Deserialize)]
#[diesel(table_name = crate::infrastructure::database::schema::users)]
pub struct NewUser {
    /// QQ号
    pub qq_number: Option<String>,

    /// 用户角色 (默认: "user")
    #[serde(default = "default_user_role")]
    pub role: String,

    /// 是否激活 (默认: true)
    #[serde(default = "default_true")]
    pub is_active: bool,

    /// 登录渠道：qq / email
    pub login_provider: String,

    /// 邮箱地址
    pub email: Option<String>,
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
    pub fn is_qq_login(&self) -> bool {
        self.login_provider == LOGIN_PROVIDER_QQ
    }

    pub fn is_email_login(&self) -> bool {
        self.login_provider == LOGIN_PROVIDER_EMAIL
    }

    /// 返回当前登录主体的稳定展示标识。
    pub fn identifier(&self) -> String {
        match self.login_provider.as_str() {
            LOGIN_PROVIDER_EMAIL => self.email.clone().unwrap_or_default(),
            _ => self.qq_number.clone().unwrap_or_default(),
        }
    }

    /// JWT subject 使用 provider 前缀，避免同值标识在不同登录渠道下混淆。
    pub fn identity_subject(&self) -> String {
        format!("{}:{}", self.login_provider, self.identifier())
    }

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
            qq_number: Some("123456".to_string()),
            role: "admin".to_string(),
            is_active: true,
            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            login_provider: LOGIN_PROVIDER_QQ.to_string(),
            email: None,
        };

        assert!(admin.is_admin());
        assert!(!admin.is_user());
    }

    #[test]
    fn test_is_user() {
        let user = User {
            id: Uuid::new_v4(),
            qq_number: Some("123456".to_string()),
            role: "user".to_string(),
            is_active: true,
            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            login_provider: LOGIN_PROVIDER_QQ.to_string(),
            email: None,
        };

        assert!(!user.is_admin());
        assert!(user.is_user());
    }

    #[test]
    fn test_new_user_defaults() {
        let new_user = NewUser {
            qq_number: Some("123456".to_string()),
            role: default_user_role(),
            is_active: default_true(),
            login_provider: LOGIN_PROVIDER_QQ.to_string(),
            email: None,
        };

        assert_eq!(new_user.role, "user");
        assert!(new_user.is_active);
    }

    #[test]
    fn test_email_identifier_subject() {
        let user = User {
            id: Uuid::new_v4(),
            qq_number: None,
            role: "user".to_string(),
            is_active: true,
            created_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            updated_at: chrono::DateTime::from_timestamp(0, 0).unwrap().naive_utc(),
            login_provider: LOGIN_PROVIDER_EMAIL.to_string(),
            email: Some("student@example.com".to_string()),
        };

        assert!(user.is_email_login());
        assert_eq!(user.identifier(), "student@example.com");
        assert_eq!(user.identity_subject(), "email:student@example.com");
    }
}
