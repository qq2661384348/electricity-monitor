//! JWT认证中间件

use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;

/// JWT Claims
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,        // 用户ID
    pub exp: usize,         // 过期时间
    pub iat: usize,         // 签发时间
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

    // 验证token
    let config = AppConfig::global();
    let secret = config.jwt.secret.as_bytes();
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // 将claims添加到请求扩展中，后续处理器可以使用
    req.extensions_mut().insert(token_data.claims);

    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};

    #[test]
    fn test_jwt_encode_decode() {
        let secret = b"test-secret";
        
        let claims = Claims {
            sub: "user123".to_string(),
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

        // 解码
        let decoded = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(secret),
            &Validation::default(),
        )
        .unwrap();

        assert_eq!(decoded.claims.sub, "user123");
    }
}
