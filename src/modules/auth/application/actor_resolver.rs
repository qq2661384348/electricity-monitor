use axum::http::StatusCode;

use crate::config::AppConfig;

use super::super::{
    domain::Actor,
    infrastructure::{resolve_credential, ResolvedCredential},
};

pub fn resolve_actor(token: &str, config: &AppConfig) -> Result<Actor, StatusCode> {
    match resolve_credential(token, config)? {
        ResolvedCredential::User(claims) => Ok(Actor::User(claims)),
    }
}
