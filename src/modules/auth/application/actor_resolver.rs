use axum::http::StatusCode;
use uuid::Uuid;

use crate::{config::AppConfig, infrastructure::repositories::UserRepository, state::AppState};

use super::super::{
    domain::{Actor, Claims, TokenKind},
    infrastructure::{resolve_credential, ResolvedCredential},
};

pub async fn resolve_actor(
    token: &str,
    config: &AppConfig,
    state: &AppState,
) -> Result<Actor, StatusCode> {
    match resolve_credential(token, config, TokenKind::Access)? {
        ResolvedCredential::User(claims) => resolve_user_actor(claims, state).await,
    }
}

async fn resolve_user_actor(claims: Claims, state: &AppState) -> Result<Actor, StatusCode> {
    let user_id = Uuid::parse_str(&claims.user_id).map_err(|_| StatusCode::UNAUTHORIZED)?;
    let user_repo = UserRepository::new(state.db_pool.clone());
    let user = user_repo
        .find_by_id(user_id)
        .await
        .map_err(|error| {
            tracing::error!(
                user_id = %claims.user_id,
                error = %error,
                "认证中间件重新校验用户失败"
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !user.is_active {
        tracing::warn!(user_id = %user.id, "已停用用户使用旧 access token 被拒绝");
        return Err(StatusCode::UNAUTHORIZED);
    }

    if user.identity_subject() != claims.sub {
        tracing::warn!(
            user_id = %user.id,
            token_subject = %claims.sub,
            current_subject = %user.identity_subject(),
            "access token subject 与当前用户身份不一致"
        );
        return Err(StatusCode::UNAUTHORIZED);
    }

    // JWT 只证明 token 未过期；授权角色必须使用数据库中的当前值，确保降权立即生效。
    Ok(Actor::User(Claims {
        sub: claims.sub,
        user_id: claims.user_id,
        role: user.role,
        token_kind: claims.token_kind,
        exp: claims.exp,
        iat: claims.iat,
    }))
}
