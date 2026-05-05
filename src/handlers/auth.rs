//! 认证处理器
//!
//! 处理用户认证相关的HTTP请求

use axum::{
    extract::State,
    http::{
        header::{self, HeaderMap, HeaderValue},
        StatusCode,
    },
    response::IntoResponse,
    Extension, Json,
};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::config::AppConfig;
use crate::domain::services::VerificationCodeService;
use crate::errors::{AppError, Result};
use crate::infrastructure::repositories::UserRepository;
use crate::modules::auth::{
    infrastructure::{resolve_credential, ResolvedCredential},
    Claims, TokenKind,
};
use crate::state::AppState;

const REFRESH_COOKIE_NAME: &str = "refresh_token";

/// 发送验证码请求
#[derive(Debug, Deserialize, Validate)]
pub struct SendVerificationCodeRequest {
    /// QQ号（5-20位数字）
    #[validate(length(min = 5, max = 20, message = "QQ号长度必须在5-20字符之间"))]
    pub qq_number: String,

    /// 验证码Token（必须先通过 /api/captcha/verify 获取）
    pub captcha_token: Option<String>,
}

/// 验证并登录请求
#[derive(Debug, Deserialize, Validate)]
pub struct VerifyAndLoginRequest {
    /// QQ号
    #[validate(length(min = 5, max = 20, message = "QQ号长度必须在5-20字符之间"))]
    pub qq_number: String,

    /// 验证码（长度由 verification.code_length 配置控制）
    #[validate(length(min = 1, max = 20, message = "验证码长度必须在1-20字符之间"))]
    pub code: String,
}

/// 登录响应
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    /// 访问Token（短期有效）
    pub access_token: String,

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

fn has_admin_override(config: &AppConfig, qq_number: &str) -> bool {
    let admin_qq = config.admin.default_qq_number.trim();
    !admin_qq.is_empty() && !admin_qq.starts_with("CHANGE-THIS") && qq_number == admin_qq
}

fn cookie_same_site_label(config: &AppConfig) -> Result<&'static str> {
    match config
        .auth
        .refresh_cookie_same_site
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "lax" => Ok("Lax"),
        "strict" => Ok("Strict"),
        "none" => Ok("None"),
        _ => Err(AppError::Internal(
            "refresh cookie SameSite 配置非法".to_string(),
        )),
    }
}

fn build_refresh_cookie_header(config: &AppConfig, refresh_token: &str) -> Result<HeaderValue> {
    let mut cookie = format!(
        "{REFRESH_COOKIE_NAME}={refresh_token}; HttpOnly; Path=/api/auth; SameSite={}; Max-Age={}",
        cookie_same_site_label(config)?,
        config.auth.refresh_expiration_hours * 3600
    );
    if config.auth.refresh_cookie_secure {
        cookie.push_str("; Secure");
    }

    HeaderValue::from_str(&cookie)
        .map_err(|error| AppError::Internal(format!("构造 refresh cookie 失败: {error}")))
}

fn build_clear_refresh_cookie_header(config: &AppConfig) -> Result<HeaderValue> {
    let mut cookie = format!(
        "{REFRESH_COOKIE_NAME}=; HttpOnly; Path=/api/auth; SameSite={}; Max-Age=0",
        cookie_same_site_label(config)?
    );
    if config.auth.refresh_cookie_secure {
        cookie.push_str("; Secure");
    }

    HeaderValue::from_str(&cookie)
        .map_err(|error| AppError::Internal(format!("构造清理 cookie 失败: {error}")))
}

fn refresh_token_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .and_then(|cookie_header| {
            cookie_header.split(';').map(str::trim).find_map(|segment| {
                segment
                    .strip_prefix(&format!("{REFRESH_COOKIE_NAME}="))
                    .map(ToOwned::to_owned)
            })
        })
}

fn issue_tokens(
    user: &crate::domain::models::User,
    config: &AppConfig,
) -> Result<(String, String, u64)> {
    crate::modules::auth::infrastructure::ensure_jwt_crypto_provider();

    let now = Utc::now().timestamp() as usize;
    let access_expiration = (config.jwt.expiration_hours * 3600) as usize;
    let refresh_expiration = (config.auth.refresh_expiration_hours * 3600) as usize;

    let access_claims = Claims {
        sub: user.qq_number.clone(),
        user_id: user.id.to_string(),
        role: user.role.clone(),
        token_kind: TokenKind::Access,
        exp: now + access_expiration,
        iat: now,
    };

    let refresh_claims = Claims {
        sub: user.qq_number.clone(),
        user_id: user.id.to_string(),
        role: user.role.clone(),
        token_kind: TokenKind::Refresh,
        exp: now + refresh_expiration,
        iat: now,
    };

    let secret = config.jwt.secret.as_bytes();
    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|error| AppError::Internal(format!("Token生成失败: {error}")))?;
    let refresh_token = encode(
        &Header::default(),
        &refresh_claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|error| AppError::Internal(format!("Token生成失败: {error}")))?;

    Ok((access_token, refresh_token, access_expiration as u64))
}

fn build_login_response(
    user: crate::domain::models::User,
    access_token: String,
    expires_in: u64,
) -> LoginResponse {
    LoginResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in,
        user: UserInfo {
            id: user.id.to_string(),
            qq_number: user.qq_number,
            role: user.role,
            is_active: user.is_active,
        },
    }
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
        has_captcha_token = req.captcha_token.is_some(),
        "收到发送验证码请求"
    );

    let captcha_token = req
        .captcha_token
        .as_deref()
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .ok_or_else(|| AppError::Unauthorized("验证码token缺失".to_string()))?;

    let captcha_service =
        crate::domain::services::captcha_verification::CaptchaVerificationService::new(
            state.redis_pool.clone(),
        );

    let token_valid = captcha_service
        .verify_and_consume_token(captcha_token)
        .await?;

    if !token_valid {
        tracing::warn!(
            qq_number = %req.qq_number,
            "验证码token无效或已过期"
        );
        return Err(AppError::Unauthorized(
            "验证码token无效或已过期".to_string(),
        ));
    }

    tracing::info!(
        qq_number = %req.qq_number,
        "验证码token验证成功"
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
    let _code = verification_service.send_and_store(&req.qq_number).await?;

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
) -> Result<impl IntoResponse> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::Internal(format!("验证失败: {}", e)))?;

    let config = AppConfig::global();
    if req.code.len() != config.verification.code_length
        || !req.code.chars().all(|ch| ch.is_ascii_digit())
    {
        return Err(AppError::Unauthorized(format!(
            "验证码必须为{}位数字",
            config.verification.code_length
        )));
    }

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

    let desired_role = if has_admin_override(config, &req.qq_number) {
        "admin"
    } else {
        "user"
    };

    let user = user_repo
        .ensure_role(
            user_repo
                .create_or_find(&req.qq_number, desired_role)
                .await?,
            desired_role,
        )
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

    let (access_token, refresh_token, expires_in) = issue_tokens(&user, config)?;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        build_refresh_cookie_header(config, &refresh_token)?,
    );

    tracing::info!(
        qq_number = %req.qq_number,
        user_id = %user.id,
        role = %user.role,
        "用户登录成功"
    );

    Ok((
        StatusCode::OK,
        headers,
        Json(build_login_response(user, access_token, expires_in)),
    ))
}

/// 刷新Token
///
/// POST /auth/refresh
pub async fn refresh_token(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    let config = AppConfig::global();
    let refresh_token = refresh_token_from_headers(&headers)
        .ok_or(AppError::Unauthorized("缺少 refresh cookie".to_string()))?;
    let ResolvedCredential::User(old_claims) =
        resolve_credential(&refresh_token, config, TokenKind::Refresh)
            .map_err(|_| AppError::Unauthorized("刷新Token无效或已过期".to_string()))?;

    // 从数据库查询用户信息（获取最新的role和状态）
    let user_repo = UserRepository::new(state.db_pool.clone());
    let user_id = uuid::Uuid::parse_str(&old_claims.user_id)
        .map_err(|_| AppError::Internal("无效的用户ID格式".to_string()))?;

    let user = user_repo
        .find_by_id(user_id)
        .await?
        .ok_or(AppError::Unauthorized("用户不存在".to_string()))?;

    if !user.is_active {
        return Err(AppError::Unauthorized("用户已被停用".to_string()));
    }

    let (access_token, new_refresh_token, expires_in) = issue_tokens(&user, config)?;
    let mut response_headers = HeaderMap::new();
    response_headers.insert(
        header::SET_COOKIE,
        build_refresh_cookie_header(config, &new_refresh_token)?,
    );

    tracing::info!(
        user_id = %old_claims.user_id,
        "Token刷新成功"
    );

    Ok((
        StatusCode::OK,
        response_headers,
        Json(build_login_response(user, access_token, expires_in)),
    ))
}

/// 退出登录
///
/// POST /auth/logout
pub async fn logout() -> Result<impl IntoResponse> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        build_clear_refresh_cookie_header(AppConfig::global())?,
    );

    Ok((StatusCode::NO_CONTENT, headers))
}

/// 获取当前用户信息
///
/// GET /auth/me
///
/// 需要JWT认证
pub async fn get_current_user(
    State(state): State<AppState>,
    Extension(user_ctx): Extension<crate::middleware::auth::UserContext>,
) -> Result<Json<UserInfo>> {
    let user_id_str = user_ctx
        .user_id
        .ok_or(AppError::Unauthorized("用户ID缺失".to_string()))?;

    // 从user_id解析UUID
    let user_id = uuid::Uuid::parse_str(&user_id_str)
        .map_err(|_| AppError::Internal("无效的用户ID格式".to_string()))?;

    let user = state
        .cache_manager
        .get_user(user_id)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(UserInfo {
        id: user.id.to_string(),
        qq_number: user.qq_number,
        role: user.role,
        is_active: user.is_active,
    }))
}
