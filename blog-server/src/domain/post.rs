use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
    pub(crate) id: i64,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) author_id: i64,
    pub(crate) created_at: chrono::DateTime<Utc>,
    pub(crate) updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePostRequest {
    pub title: String,
    pub content: String,
}

impl Post {
    pub fn new() -> Self {
        Post::default()
    }
}
