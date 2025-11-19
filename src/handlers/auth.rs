//! 认证处理器
//! 
//! 处理用户认证相关的HTTP请求

use axum::{
    extract::State,
    http::StatusCode,
    Extension,
    Json,
};
use chrono::Utc;
use jsonwebtoken::{encode, decode, EncodingKey, DecodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::config::AppConfig;
use crate::domain::services::VerificationCodeService;
use crate::errors::{AppError, Result};
use crate::infrastructure::repositories::UserRepository;
use crate::middleware::auth::Claims;
use crate::state::AppState;

/// 发送验证码请求
#[derive(Debug, Deserialize, Validate)]
pub struct SendVerificationCodeRequest {
    /// QQ号（5-20位数字）
    #[validate(length(min = 5, max = 20, message = "QQ号长度必须在5-20字符之间"))]
    pub qq_number: String,
}

/// 验证并登录请求
#[derive(Debug, Deserialize, Validate)]
pub struct VerifyAndLoginRequest {
    /// QQ号
    #[validate(length(min = 5, max = 20, message = "QQ号长度必须在5-20字符之间"))]
    pub qq_number: String,
    
    /// 验证码（6位数字）
    #[validate(length(equal = 6, message = "验证码必须为6位数字"))]
    pub code: String,
}

/// 刷新Token请求
#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    /// 旧的刷新Token
    pub refresh_token: String,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// 访问Token（短期有效）
    pub access_token: String,
    
    /// 刷新Token（长期有效）
    pub refresh_token: String,
    
    /// Token类型
    pub token_type: String,
    
    /// 过期时间（秒）
    pub expires_in: u64,
    
    /// 用户信息
    pub user: UserInfo,
}

/// 用户信息
#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: String,
    pub qq_number: String,
    pub role: String,
    pub is_active: bool,
}

/// 发送验证码
/// 
/// POST /auth/send-verification-code
pub async fn send_verification_code(
    State(state): State<AppState>,
    Json(req): Json<SendVerificationCodeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::Internal(format!("验证失败: {}", e)))?;

    tracing::info!(
        qq_number = %req.qq_number,
        "收到发送验证码请求"
    );

    // 创建QQ客户端
    let qq_client = crate::infrastructure::QQClient::new(AppConfig::global().qq_bot.clone())
        .map_err(|e| AppError::Internal(format!("QQ客户端初始化失败: {}", e)))?;

    // 创建验证码服务
    let verification_service = VerificationCodeService::new(
        state.redis_pool.clone(),
        qq_client,
        AppConfig::global().verification.clone(),
    );

    // 发送验证码
    let _code = verification_service
        .send_and_store(&req.qq_number)
        .await?;

    tracing::info!(
        qq_number = %req.qq_number,
        "验证码发送成功"
    );

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "验证码已发送",
            "qq_number": req.qq_number
        })),
    ))
}

/// 验证并登录
/// 
/// POST /auth/verify-and-login
pub async fn verify_and_login(
    State(state): State<AppState>,
    Json(req): Json<VerifyAndLoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>)> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::Internal(format!("验证失败: {}", e)))?;

    tracing::info!(
        qq_number = %req.qq_number,
        "收到登录请求"
    );

    // 创建QQ客户端
    let qq_client = crate::infrastructure::QQClient::new(AppConfig::global().qq_bot.clone())
        .map_err(|e| AppError::Internal(format!("QQ客户端初始化失败: {}", e)))?;

    // 创建验证码服务
    let verification_service = VerificationCodeService::new(
        state.redis_pool.clone(),
        qq_client,
        AppConfig::global().verification.clone(),
    );

    // 验证验证码
    let is_valid = verification_service
        .verify_code(&req.qq_number, &req.code)
        .await?;

    if !is_valid {
        tracing::warn!(
            qq_number = %req.qq_number,
            "验证码验证失败"
        );
        return Err(AppError::Unauthorized("验证码无效或已过期".to_string()));
    }

    // 创建用户仓储
    let user_repo = UserRepository::new(state.db_pool.clone());

    // 创建或查找用户
    let user = user_repo
        .create_or_find(&req.qq_number, "user")
        .await?;

    // 检查用户是否激活
    if !user.is_active {
        tracing::warn!(
            qq_number = %req.qq_number,
            user_id = %user.id,
            "用户已被停用"
        );
        return Err(AppError::Unauthorized("用户已被停用".to_string()));
    }

    // 生成JWT Token
    let config = AppConfig::global();
    let now = Utc::now().timestamp() as usize;
    let expiration = (config.jwt.expiration_hours * 3600) as usize;

    // 访问Token Claims（普通用户JWT，不包含role）
    let access_claims = Claims {
        sub: user.qq_number.clone(),
        user_id: user.id.to_string(),
        exp: now + expiration,
        iat: now,
    };

    // 刷新Token Claims（7天有效期，不包含role）
    let refresh_claims = Claims {
        sub: user.qq_number.clone(),
        user_id: user.id.to_string(),
        exp: now + (7 * 24 * 3600),
        iat: now,
    };

    let secret = config.jwt.secret.as_bytes();

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| AppError::Internal(format!("Token生成失败: {}", e)))?;

    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| AppError::Internal(format!("Token生成失败: {}", e)))?;

    tracing::info!(
        qq_number = %req.qq_number,
        user_id = %user.id,
        role = %user.role,
        "用户登录成功"
    );

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: expiration as u64,
            user: UserInfo {
                id: user.id.to_string(),
                qq_number: user.qq_number,
                role: user.role,
                is_active: user.is_active,
            },
        }),
    ))
}

/// 刷新Token
/// 
/// POST /auth/refresh
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> Result<(StatusCode, Json<LoginResponse>)> {
    let config = AppConfig::global();
    let secret = config.jwt.secret.as_bytes();

    // 验证旧Token（使用增强的验证规则）
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.leeway = 60; // 60秒时钟容差
    validation.set_required_spec_claims(&["exp", "iat"]); // 必需字段
    
    let token_data = decode::<Claims>(
        &req.refresh_token,
        &DecodingKey::from_secret(secret),
        &validation,
    )
    .map_err(|_| AppError::Unauthorized("刷新Token无效或已过期".to_string()))?;

    let old_claims = token_data.claims;

    // 从数据库查询用户信息（获取最新的role和状态）
    let user_repo = UserRepository::new(state.db_pool.clone());
    let user_id = uuid::Uuid::parse_str(&old_claims.user_id)
        .map_err(|_| AppError::Internal("无效的用户ID格式".to_string()))?;
    
    let user = user_repo
        .find_by_id(user_id)
        .await?
        .ok_or(AppError::Unauthorized("用户不存在".to_string()))?;

    // 生成新Token
    let now = Utc::now().timestamp() as usize;
    let expiration = (config.jwt.expiration_hours * 3600) as usize;

    let new_access_claims = Claims {
        sub: old_claims.sub.clone(),
        user_id: old_claims.user_id.clone(),
        exp: now + expiration,
        iat: now,
    };

    let new_refresh_claims = Claims {
        sub: old_claims.sub.clone(),
        user_id: old_claims.user_id.clone(),
        exp: now + (7 * 24 * 3600),
        iat: now,
    };

    let access_token = encode(
        &Header::default(),
        &new_access_claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| AppError::Internal(format!("Token生成失败: {}", e)))?;

    let refresh_token = encode(
        &Header::default(),
        &new_refresh_claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|e| AppError::Internal(format!("Token生成失败: {}", e)))?;

    tracing::info!(
        user_id = %old_claims.user_id,
        "Token刷新成功"
    );

    Ok((
        StatusCode::OK,
        Json(LoginResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: expiration as u64,
            user: UserInfo {
                id: user.id.to_string(),
                qq_number: user.qq_number,
                role: user.role,
                is_active: user.is_active,
            },
        }),
    ))
}

/// 获取当前用户信息
/// 
/// GET /auth/me
/// 
/// 需要JWT认证或admin_token
pub async fn get_current_user(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<crate::middleware::auth::UserContext>,
) -> Result<Json<UserInfo>> {
    // 如果是管理员
    if user_ctx.is_admin {
        // 管理员使用固定QQ号
        let config = AppConfig::global();
        let admin_qq = &config.admin.default_qq_number;
        
        return Ok(Json(UserInfo {
            id: "00000000-0000-0000-0000-000000000000".to_string(), // 管理员固定UUID
            qq_number: admin_qq.clone(),
            role: "admin".to_string(),
            is_active: true,
        }));
    }
    
    // 普通用户：从UserContext中获取user_id
    let user_id_str = user_ctx.user_id
        .ok_or(AppError::Unauthorized("用户ID缺失".to_string()))?;
    
    let user_repo = UserRepository::new(state.db_pool.clone());

    // 从user_id解析UUID
    let user_id = uuid::Uuid::parse_str(&user_id_str)
        .map_err(|_| AppError::Internal("无效的用户ID格式".to_string()))?;

    // 查询用户
    let user = user_repo
        .find_by_id(user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(UserInfo {
        id: user.id.to_string(),
        qq_number: user.qq_number,
        role: user.role,
        is_active: user.is_active,
    }))
}
