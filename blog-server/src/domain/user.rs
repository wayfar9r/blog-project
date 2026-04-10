use std::fmt::Display;

use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub created_at: chrono::DateTime<Utc>,
}

impl Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "User {{ id: {}, username: {}, email: {}, created_at: {} }}",
            self.id, self.username, self.email, self.created_at
        )
    }
}

impl User {
    pub fn new(
        id: i64,
        username: String,
        email: String,
        password_hash: String,
        created_at: chrono::DateTime<Utc>,
    ) -> Self {
        User {
            id,
            username,
            email,
            password_hash,
            created_at,
        }
    }
}
