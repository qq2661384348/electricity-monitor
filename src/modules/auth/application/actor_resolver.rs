use axum::http::StatusCode;

use crate::config::AppConfig;

use super::super::{
    domain::{Actor, TokenKind},
    infrastructure::{resolve_credential, ResolvedCredential},
};

pub fn resolve_actor(token: &str, config: &AppConfig) -> Result<Actor, StatusCode> {
    match resolve_credential(token, config, TokenKind::Access)? {
        ResolvedCredential::User(claims) => Ok(Actor::User(claims)),
    }
}
