use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::{
    config::AppConfig,
    middleware::auth::UserContext,
    modules::auth::{application::resolve_actor, domain::Actor},
};

pub async fn auth_middleware(mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    if !auth_header.starts_with("Bearer ") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let token = &auth_header[7..];
    let actor = resolve_actor(token, AppConfig::global())?;

    if let Some(claims) = actor.claims().cloned() {
        req.extensions_mut().insert(claims);
    }
    req.extensions_mut().insert(UserContext::from(&actor));
    req.extensions_mut().insert(actor.clone());

    tracing::info!(
        actor_type = actor.actor_type(),
        method = %req.method(),
        path = %req.uri().path(),
        "认证通过"
    );

    Ok(next.run(req).await)
}

pub async fn require_admin(req: Request, next: Next) -> Result<Response, StatusCode> {
    if req.extensions().get::<Actor>().is_some_and(Actor::is_admin) {
        return Ok(next.run(req).await);
    }

    tracing::warn!("非管理员用户尝试访问管理员资源被拒绝");
    Err(StatusCode::FORBIDDEN)
}

pub async fn require_user(req: Request, next: Next) -> Result<Response, StatusCode> {
    if req
        .extensions()
        .get::<Actor>()
        .is_some_and(Actor::is_authenticated)
    {
        return Ok(next.run(req).await);
    }

    tracing::warn!("未认证用户尝试访问受保护资源被拒绝");
    Err(StatusCode::UNAUTHORIZED)
}
