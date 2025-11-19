//! 验证码路由
//! 
//! 定义验证码相关的API路由

use axum::{
    routing::post,
    Router,
};

use crate::handlers::captcha;
use crate::state::AppState;

/// 创建验证码路由
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/captcha/verify", post(captcha::verify_captcha))
}
