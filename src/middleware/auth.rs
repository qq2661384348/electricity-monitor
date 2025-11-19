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

/// 管理员标记（通过固定token识别）
#[derive(Debug, Clone)]
pub struct AdminMarker;

/// JWT Claims（普通用户）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// QQ号（主题标识）
    pub sub: String,
    
    /// 用户UUID（字符串格式）
    pub user_id: String,
    
    /// 过期时间（Unix时间戳）
    pub exp: usize,
    
    /// 签发时间（Unix时间戳）
    pub iat: usize,
}

/// 用户上下文（统一的认证信息）
#[derive(Debug, Clone)]
pub struct UserContext {
    /// 是否为管理员
    pub is_admin: bool,
    
    /// 用户ID（仅普通用户有效，管理员为None）
    pub user_id: Option<String>,
}

/// JWT认证中间件
/// 
/// 优先检查admin_token，如果匹配则标记为管理员
/// 否则验证JWT token（普通用户）
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
    let config = AppConfig::global();

    // 优先检查是否为管理员固定token
    if token == config.jwt.admin_token {
        // 是管理员token，注入管理员上下文
        let user_ctx = UserContext {
            is_admin: true,
            user_id: None,
        };
        req.extensions_mut().insert(AdminMarker);  // 保留AdminMarker用于require_admin中间件
        req.extensions_mut().insert(user_ctx);
        
        // 记录管理员访问（不记录token值，防止泄露）
        tracing::info!(
            method = %req.method(),
            path = %req.uri().path(),
            "管理员访问"
        );
        
        return Ok(next.run(req).await);
    }

    // 不是管理员token，验证JWT（普通用户）
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

    // 注入普通用户上下文
    let user_ctx = UserContext {
        is_admin: false,
        user_id: Some(token_data.claims.user_id.clone()),
    };
    req.extensions_mut().insert(token_data.claims.clone());  // 保留Claims用于兼容性
    req.extensions_mut().insert(user_ctx);

    Ok(next.run(req).await)
}

/// 要求管理员权限的中间件
/// 
/// # 使用
/// 必须在auth_middleware之后使用
pub async fn require_admin(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 检查是否有AdminMarker（管理员固定token）
    if req.extensions().get::<AdminMarker>().is_some() {
        return Ok(next.run(req).await);
    }

    // 没有管理员权限，拒绝访问
    tracing::warn!("非管理员用户尝试访问管理员资源被拒绝");
    Err(StatusCode::FORBIDDEN)
}

/// 要求用户认证的中间件（包括admin和普通用户）
/// 
/// # 使用
/// 必须在auth_middleware之后使用
pub async fn require_user(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 检查是否有AdminMarker或Claims（管理员或普通用户都允许）
    let has_admin = req.extensions().get::<AdminMarker>().is_some();
    let has_user = req.extensions().get::<Claims>().is_some();

    if !has_admin && !has_user {
        tracing::warn!("未认证用户尝试访问受保护资源被拒绝");
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}

/// 辅助函数：检查用户是否拥有绑定资源
/// 
/// # 参数
/// - `claims`: JWT Claims（普通用户）
/// - `binding_user_id`: 绑定的user_id
/// 
/// # 返回
/// - `true`: 用户拥有该资源
/// - `false`: 用户无权访问该资源
/// 
/// # 注意
/// 此函数仅用于普通用户的所有权检查
/// 管理员应在调用此函数前通过AdminMarker判断并直接放行
pub fn check_binding_ownership(claims: &Claims, binding_user_id: &str) -> bool {
    // 普通用户只能访问自己的资源
    claims.user_id == binding_user_id
}

/// 辅助函数：判断请求是否为管理员
/// 
/// # 参数
/// - `req_extensions`: 请求扩展
/// 
/// # 返回
/// - `true`: 请求来自管理员
/// - `false`: 请求来自普通用户或未认证
pub fn is_admin(req_extensions: &axum::http::Extensions) -> bool {
    req_extensions.get::<AdminMarker>().is_some()
}

/// 辅助函数：获取请求中的Claims（普通用户）
/// 
/// # 参数
/// - `req_extensions`: 请求扩展
/// 
/// # 返回
/// - `Some(Claims)`: 普通用户的Claims
/// - `None`: 请求来自管理员或未认证
pub fn get_user_claims(req_extensions: &axum::http::Extensions) -> Option<&Claims> {
    req_extensions.get::<Claims>()
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
        assert_eq!(decoded.claims.user_id, "550e8400-e29b-41d4-a716-446655440000");
    }
}
