use axum::http::StatusCode;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use crate::config::AppConfig;

use super::super::domain::{Claims, TokenKind};

pub enum ResolvedCredential {
    User(Claims),
}

pub fn resolve_credential(
    token: &str,
    config: &AppConfig,
    required_token_kind: TokenKind,
) -> Result<ResolvedCredential, StatusCode> {
    super::ensure_jwt_crypto_provider();

    let secret = config.jwt.secret.as_bytes();
    let mut validation = Validation::new(Algorithm::HS256);
    validation.leeway = 60;
    validation.set_required_spec_claims(&["exp", "iat"]);

    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    if token_data.claims.token_kind != required_token_kind {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(ResolvedCredential::User(token_data.claims))
}
