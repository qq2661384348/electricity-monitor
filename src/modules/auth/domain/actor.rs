use super::Claims;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Actor {
    User(Claims),
}

impl Actor {
    pub fn is_admin(&self) -> bool {
        matches!(self, Self::User(claims) if claims.role == "admin")
    }

    pub fn is_authenticated(&self) -> bool {
        true
    }

    pub fn user_id(&self) -> Option<&str> {
        Some(match self {
            Self::User(claims) => &claims.user_id,
        })
    }

    pub fn claims(&self) -> Option<&Claims> {
        Some(match self {
            Self::User(claims) => claims,
        })
    }

    pub fn actor_type(&self) -> &'static str {
        match self {
            Self::User(claims) if claims.role == "admin" => "admin",
            Self::User(_) => "user",
        }
    }
}
