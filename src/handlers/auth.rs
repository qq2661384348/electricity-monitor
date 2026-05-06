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

use crate::config::AppConfig;
use crate::domain::models::{LOGIN_PROVIDER_EMAIL, LOGIN_PROVIDER_QQ};
use crate::domain::services::VerificationCodeService;
use crate::errors::{AppError, Result};
use crate::infrastructure::repositories::UserRepository;
use crate::modules::auth::{
    infrastructure::{resolve_credential, ResolvedCredential},
    Claims, TokenKind,
};
use crate::state::AppState;
use crate::utils::validation::{normalize_email_address, QQ_NUMBER_REGEX};

const REFRESH_COOKIE_NAME: &str = "refresh_token";

/// 发送验证码请求
#[derive(Debug, Deserialize)]
pub struct SendVerificationCodeRequest {
    /// 登录模式。缺省为 qq，用于兼容旧前端和旧调用方。
    pub login_mode: Option<LoginMode>,

    /// 统一登录标识：QQ 号或邮箱地址。
    pub identifier: Option<String>,

    /// 兼容字段：QQ号（5-20位数字）
    pub qq_number: Option<String>,

    /// 兼容字段：邮箱地址
    pub email: Option<String>,

    /// 验证码Token（必须先通过 /api/captcha/verify 获取）
    pub captcha_token: Option<String>,
}

/// 验证并登录请求
#[derive(Debug, Deserialize)]
pub struct VerifyAndLoginRequest {
    /// 登录模式。缺省为 qq，用于兼容旧前端和旧调用方。
    pub login_mode: Option<LoginMode>,

    /// 统一登录标识：QQ 号或邮箱地址。
    pub identifier: Option<String>,

    /// 兼容字段：QQ号
    pub qq_number: Option<String>,

    /// 兼容字段：邮箱地址
    pub email: Option<String>,

    /// 验证码（长度由 verification.code_length 配置控制）
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
    pub login_mode: String,
    pub identifier: String,
    pub qq_number: Option<String>,
    pub email: Option<String>,
    pub role: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LoginMode {
    Qq,
    Email,
}

impl LoginMode {
    fn as_provider(self) -> &'static str {
        match self {
            Self::Qq => LOGIN_PROVIDER_QQ,
            Self::Email => LOGIN_PROVIDER_EMAIL,
        }
    }
}

#[derive(Debug, Clone)]
struct LoginIdentity {
    mode: LoginMode,
    identifier: String,
    qq_number: Option<String>,
    email: Option<String>,
}

impl LoginIdentity {
    fn provider(&self) -> &'static str {
        self.mode.as_provider()
    }

    fn from_parts(
        login_mode: Option<LoginMode>,
        identifier: Option<&str>,
        qq_number: Option<&str>,
        email: Option<&str>,
    ) -> Result<Self> {
        let mode = login_mode.unwrap_or(LoginMode::Qq);

        match mode {
            LoginMode::Qq => {
                let qq_number = identifier
                    .or(qq_number)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| AppError::Unauthorized("请输入QQ号".to_string()))?;

                if !QQ_NUMBER_REGEX.is_match(qq_number) {
                    return Err(AppError::Unauthorized("QQ号必须为5-20位数字".to_string()));
                }

                Ok(Self {
                    mode,
                    identifier: qq_number.to_string(),
                    qq_number: Some(qq_number.to_string()),
                    email: None,
                })
            }
            LoginMode::Email => {
                let email = identifier
                    .or(email)
                    .and_then(normalize_email_address)
                    .ok_or_else(|| AppError::Unauthorized("请输入有效邮箱地址".to_string()))?;

                Ok(Self {
                    mode,
                    identifier: email.clone(),
                    qq_number: None,
                    email: Some(email),
                })
            }
        }
    }
}

impl SendVerificationCodeRequest {
    fn identity(&self) -> Result<LoginIdentity> {
        LoginIdentity::from_parts(
            self.login_mode,
            self.identifier.as_deref(),
            self.qq_number.as_deref(),
            self.email.as_deref(),
        )
    }
}

impl VerifyAndLoginRequest {
    fn identity(&self) -> Result<LoginIdentity> {
        LoginIdentity::from_parts(
            self.login_mode,
            self.identifier.as_deref(),
            self.qq_number.as_deref(),
            self.email.as_deref(),
        )
    }
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
        sub: user.identity_subject(),
        user_id: user.id.to_string(),
        role: user.role.clone(),
        token_kind: TokenKind::Access,
        exp: now + access_expiration,
        iat: now,
    };

    let refresh_claims = Claims {
        sub: user.identity_subject(),
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
        user: build_user_info(user),
    }
}

fn build_user_info(user: crate::domain::models::User) -> UserInfo {
    let identifier = user.identifier();
    UserInfo {
        id: user.id.to_string(),
        login_mode: user.login_provider,
        identifier,
        qq_number: user.qq_number,
        email: user.email,
        role: user.role,
        is_active: user.is_active,
    }
}

/// 发送验证码
///
/// POST /auth/send-verification-code
pub async fn send_verification_code(
    State(state): State<AppState>,
    Json(req): Json<SendVerificationCodeRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>)> {
    let identity = req.identity()?;

    tracing::info!(
        login_provider = identity.provider(),
        identifier = %identity.identifier,
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
            login_provider = identity.provider(),
            identifier = %identity.identifier,
            "验证码token无效或已过期"
        );
        return Err(AppError::Unauthorized(
            "验证码token无效或已过期".to_string(),
        ));
    }

    tracing::info!(
        login_provider = identity.provider(),
        identifier = %identity.identifier,
        "验证码token验证成功"
    );

    let verification_service = VerificationCodeService::new(
        state.redis_pool.clone(),
        AppConfig::global().verification.clone(),
    );

    match identity.mode {
        LoginMode::Qq => {
            let qq_number = identity
                .qq_number
                .as_deref()
                .ok_or_else(|| AppError::Internal("QQ 登录标识缺失".to_string()))?;
            let qq_client =
                crate::infrastructure::QQClient::new(AppConfig::global().qq_bot.clone())
                    .map_err(|e| AppError::Internal(format!("QQ客户端初始化失败: {}", e)))?;
            let _code = verification_service
                .send_and_store_qq(&qq_client, qq_number)
                .await?;
        }
        LoginMode::Email => {
            let email = identity
                .email
                .as_deref()
                .ok_or_else(|| AppError::Internal("邮箱登录标识缺失".to_string()))?;
            let email_sender = state
                .email_sender
                .as_deref()
                .ok_or_else(|| AppError::Config("邮件发送未配置，无法使用邮箱登录".to_string()))?;
            let _code = verification_service
                .send_and_store_email(email_sender, email)
                .await?;
        }
    }

    tracing::info!(
        login_provider = identity.provider(),
        identifier = %identity.identifier,
        "验证码发送成功"
    );

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "message": "验证码已发送",
            "login_mode": identity.provider(),
            "identifier": identity.identifier,
            "qq_number": identity.qq_number,
            "email": identity.email
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
    let identity = req.identity()?;

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
        login_provider = identity.provider(),
        identifier = %identity.identifier,
        "收到登录请求"
    );

    let verification_service = VerificationCodeService::new(
        state.redis_pool.clone(),
        AppConfig::global().verification.clone(),
    );

    let is_valid = verification_service
        .verify_code_for(identity.provider(), &identity.identifier, &req.code)
        .await?;

    if !is_valid {
        tracing::warn!(
            login_provider = identity.provider(),
            identifier = %identity.identifier,
            "验证码验证失败"
        );
        return Err(AppError::Unauthorized("验证码无效或已过期".to_string()));
    }

    // 创建用户仓储
    let user_repo = UserRepository::new(state.db_pool.clone());

    let user = match identity.mode {
        LoginMode::Qq => {
            let qq_number = identity
                .qq_number
                .as_deref()
                .ok_or_else(|| AppError::Internal("QQ 登录标识缺失".to_string()))?;
            let desired_role = if has_admin_override(config, qq_number) {
                "admin"
            } else {
                "user"
            };

            user_repo
                .ensure_role(
                    user_repo.create_or_find_qq(qq_number, desired_role).await?,
                    desired_role,
                )
                .await?
        }
        LoginMode::Email => {
            let email = identity
                .email
                .as_deref()
                .ok_or_else(|| AppError::Internal("邮箱登录标识缺失".to_string()))?;

            // 本轮明确不启用邮箱管理员提升；即使未来预留配置入口，当前邮箱登录也只能落 user。
            user_repo
                .ensure_role(user_repo.create_or_find_email(email, "user").await?, "user")
                .await?
        }
    };

    // 检查用户是否激活
    if !user.is_active {
        tracing::warn!(
            login_provider = identity.provider(),
            identifier = %identity.identifier,
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
        login_provider = identity.provider(),
        identifier = %identity.identifier,
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

    Ok(Json(build_user_info(user)))
}
