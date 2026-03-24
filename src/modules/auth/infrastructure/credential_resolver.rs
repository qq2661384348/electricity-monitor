use axum::http::StatusCode;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use crate::config::AppConfig;

use super::super::domain::Claims;

pub enum ResolvedCredential {
    User(Claims),
}

pub fn resolve_credential(
    token: &str,
    config: &AppConfig,
) -> Result<ResolvedCredential, StatusCode> {
    let secret = config.jwt.secret.as_bytes();
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 60;
    validation.set_required_spec_claims(&["exp", "iat"]);

    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(ResolvedCredential::User(token_data.claims))
}
