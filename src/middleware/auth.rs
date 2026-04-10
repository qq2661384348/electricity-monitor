//! JWT认证中间件兼容层

pub use crate::modules::auth::{auth_middleware, require_admin, require_user, Actor, Claims};

#[derive(Debug, Clone)]
pub struct UserContext {
    pub is_admin: bool,
    pub user_id: Option<String>,
}

impl From<&Actor> for UserContext {
    fn from(actor: &Actor) -> Self {
        Self {
            is_admin: actor.is_admin(),
            user_id: actor.user_id().map(|value| value.to_string()),
        }
    }
}

/// 辅助函数：检查用户是否拥有绑定资源
pub fn check_binding_ownership(claims: &Claims, binding_user_id: &str) -> bool {
    claims.user_id == binding_user_id
}

/// 辅助函数：判断请求是否为管理员
pub fn is_admin(req_extensions: &axum::http::Extensions) -> bool {
    req_extensions
        .get::<Actor>()
        .is_some_and(crate::modules::auth::domain::Actor::is_admin)
}

/// 辅助函数：获取请求中的Claims（普通用户）
pub fn get_user_claims(req_extensions: &axum::http::Extensions) -> Option<&Claims> {
    req_extensions.get::<Claims>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
    use crate::modules::auth::TokenKind;

    #[test]
    fn test_jwt_encode_decode() {
        crate::modules::auth::infrastructure::ensure_jwt_crypto_provider();

        let secret = b"test-secret";

        let claims = Claims {
            sub: "123456789".to_string(),
            user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            role: "user".to_string(),
            token_kind: TokenKind::Access,
            exp: 9_999_999_999,
            iat: 1_234_567_890,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret),
        )
        .unwrap();

        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 60;
        validation.set_required_spec_claims(&["exp", "iat"]);

        let decoded =
            decode::<Claims>(&token, &DecodingKey::from_secret(secret), &validation).unwrap();

        assert_eq!(decoded.claims.sub, "123456789");
        assert_eq!(
            decoded.claims.user_id,
            "550e8400-e29b-41d4-a716-446655440000"
        );
        assert_eq!(decoded.claims.token_kind, TokenKind::Access);
    }
}
