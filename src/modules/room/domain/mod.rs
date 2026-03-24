use uuid::Uuid;

use crate::{
    errors::{AppError, Result},
    middleware::auth::UserContext,
};

#[derive(Debug, Clone)]
pub struct RoomActor {
    pub is_admin: bool,
    pub user_id: Option<Uuid>,
}

impl RoomActor {
    pub fn from_user_context(user_ctx: &UserContext) -> Result<Self> {
        let user_id = match &user_ctx.user_id {
            Some(value) => Some(
                Uuid::parse_str(value)
                    .map_err(|_| AppError::Internal("无效的用户ID格式".to_string()))?,
            ),
            None => None,
        };

        Ok(Self {
            is_admin: user_ctx.is_admin,
            user_id,
        })
    }
}
