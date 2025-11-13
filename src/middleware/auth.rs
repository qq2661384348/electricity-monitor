//! JWT认证中间件

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;

/// JWT Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// QQ号（主题标识）
    pub sub: String,
    
    /// 用户UUID（字符串格式）
    pub user_id: String,
    
    /// 用户角色 (admin/user)
    pub role: String,
    
    /// 过期时间（Unix时间戳）
    pub exp: usize,
    
    /// 签发时间（Unix时间戳）
    pub iat: usize,
}

/// JWT认证中间件
pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 获取Authorization头
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 验证Bearer token格式
    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..]; // 去掉"Bearer "前缀

    // 验证token（使用增强的验证规则）
    let config = AppConfig::global();
    let secret = config.jwt.secret.as_bytes();
    
    // 创建严格的JWT验证规则
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 60; // 60秒时钟容差，容忍服务器时间偏差
    validation.set_required_spec_claims(&["exp", "iat"]); // 要求包含过期和签发时间
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &validation,
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 将claims添加到请求扩展中，后续处理器可以使用
    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}

/// 要求管理员角色的中间件
/// 
/// # 使用
/// 必须在auth_middleware之后使用
pub async fn require_admin(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 从请求扩展中获取Claims
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 验证角色
    if claims.role != "admin" {
        tracing::warn!(
            user_id = %claims.user_id,
            role = %claims.role,
            "用户尝试访问管理员资源被拒绝"
        );
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// 要求用户角色的中间件（包括admin）
/// 
/// # 使用
/// 必须在auth_middleware之后使用
pub async fn require_user(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 从请求扩展中获取Claims
    let claims = req
        .extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // 验证角色（user或admin都可以）
    if claims.role != "user" && claims.role != "admin" {
        tracing::warn!(
            user_id = %claims.user_id,
            role = %claims.role,
            "用户角色无效"
        );
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(next.run(req).await)
}

/// 辅助函数：检查用户是否拥有绑定资源
/// 
/// # 参数
/// - `claims`: JWT Claims
/// - `binding_user_id`: 绑定的user_id
/// 
/// # 返回
/// - `true`: 用户是管理员或拥有该资源
/// - `false`: 用户无权访问该资源
pub fn check_binding_ownership(claims: &Claims, binding_user_id: &str) -> bool {
    // 管理员可以访问所有资源
    if claims.role == "admin" {
        return true;
    }
    
    // 普通用户只能访问自己的资源
    claims.user_id == binding_user_id
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    #[test]
    fn test_jwt_encode_decode() {
        let secret = b"test-secret";
        
        let claims = Claims {
            sub: "123456789".to_string(),
            user_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            role: "user".to_string(),
            exp: 9999999999,
            iat: 1234567890,
        };

        // 编码
        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret),
        )
        .unwrap();

        // 解码（使用与生产代码一致的验证规则）
        let mut validation = Validation::new(Algorithm::HS256);
        validation.leeway = 60;
        validation.set_required_spec_claims(&["exp", "iat"]);
        
        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret),
            &validation,
        )
        .unwrap();

        assert_eq!(decoded.claims.sub, "123456789");
        assert_eq!(decoded.claims.role, "user");
    }
}
