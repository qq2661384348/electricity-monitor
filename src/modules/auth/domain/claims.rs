use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Claims {
    pub sub: String,
    pub user_id: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}
