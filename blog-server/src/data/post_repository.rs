use std::{fmt::Display, sync::Arc};

use crate::{data::CreatePostData, domain::post::Post, presentation};
use sqlx::{Postgres, postgres::PgPool};

impl From<presentation::dto::UpdatePostRequest> for Vec<Field> {
    fn from(value: presentation::dto::UpdatePostRequest) -> Self {
        let mut v = Vec::new();
        if let Some(content) = value.content {
            v.push(Field::Content(content));
        }
        if let Some(title) = value.title {
            v.push(Field::Title(title));
        }
        v
    }
}

pub enum Field {
    Id(u32),
    Title(String),
    Content(String),
    AuthorId(u32),
    CreatedAt(chrono::DateTime<chrono::Utc>),
    UpdatedAt(chrono::DateTime<chrono::Utc>),
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Field::Id(_) => "id",
                Field::AuthorId(_) => "author_id",
                Field::Content(_) => "content",
                Field::Title(_) => "title",
                Field::CreatedAt(_) => "created_at",
                Field::UpdatedAt(_) => "updated_at",
            }
        )
    }
}

pub struct PostRepository {
    pool: Arc<PgPool>,
}

pub enum Order {
    Asc,
    Desc,
}

impl Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Order::Asc => "ASC",
                Order::Desc => "DESC",
            }
        )
    }
}

impl PostRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        PostRepository { pool }
    }

    pub async fn list(
        &self,
        offset: u32,
        limit: u8,
        order: Order,
    ) -> Result<Vec<Post>, sqlx::Error> {
        sqlx::query_as!(
            Post,
            "SELECT * FROM posts ORDER BY $1 OFFSET $2 LIMIT $3;",
            order.to_string(),
            offset as i64,
            limit as i64
        )
        .fetch_all(&*self.pool)
        .await
    }

    pub async fn count(&self) -> Result<u32, sqlx::Error> {
        sqlx::query_scalar!("SELECT COUNT(*) FROM posts;")
            .fetch_one(&*self.pool)
            .await
            .map(|r| {
                r.ok_or(sqlx::Error::ColumnNotFound(
                    "couldn't get count from posts".into(),
                ))
            })?
            .map(|count| count as u32)
    }
}

impl PostRepository {
    pub async fn create(&self, fields: CreatePostData) -> Result<Post, sqlx::Error> {
        sqlx::query_as!(
            Post,
            "INSERT INTO posts (title, content, author_id) VALUES ($1, $2, $3) RETURNING *;",
            fields.title,
            fields.content,
            fields.author as i64
        )
        .fetch_one(&*self.pool)
        .await
    }
    pub async fn find(&self, id: u32) -> Result<Post, sqlx::Error> {
        sqlx::query_as!(
            Post,
            "SELECT * from posts WHERE id = $1 LIMIT 1;",
            id as i64
        )
        .fetch_one(&*self.pool)
        .await
    }
    pub async fn delete(&self, id: u32) -> Result<(), sqlx::Error> {
        self.find(id).await?;
        sqlx::query!("DELETE FROM posts WHERE id = $1;", id as i32)
            .execute(&*self.pool)
            .await
            .map(|_r| ())
    }
    pub async fn update(&self, id: u32, fields: Vec<Field>) -> Result<Post, sqlx::Error> {
        if fields.is_empty() {
            return Err(sqlx::Error::InvalidArgument("empty fields".into()));
        }
        let mut builder = sqlx::QueryBuilder::<Postgres>::new("UPDATE posts SET ");
        let mut separated_builder = builder.separated(',');
        for field in fields {
            separated_builder.push(format!("{}=", field));
            match field {
                Field::Title(title) => {
                    separated_builder.push_bind_unseparated(title);
                }
                Field::Content(content) => {
                    separated_builder.push_bind_unseparated(content);
                }
                Field::AuthorId(id) => {
                    separated_builder.push_bind_unseparated(id as i64);
                }
                f => {
                    return Err(sqlx::Error::InvalidArgument(format!(
                        "field {} is not supported",
                        f
                    )));
                }
            };
        }
        builder.push(", updated_at = NOW() WHERE id =");
        builder.push_bind(id as i64);
        builder.push(" RETURNING *;");
        let query = builder.build_query_as();
        query.fetch_one(&*self.pool).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::user_repository::UserRepository;
    use crate::tools::test::{TestDbManager, create_test_db_pool};

    #[actix_web::test]
    #[ignore = "one-thread-test"]
    pub async fn should_crud_posts() {
        let db_pool = Arc::new(create_test_db_pool().await);
        let db_manager = TestDbManager::new().await;
        let post_rep = PostRepository::new(Arc::clone(&db_pool));
        let user_rep = UserRepository::new(Arc::clone(&db_pool));
        db_manager.create().await;
        let user = user_rep
            .create(presentation::dto::RegisterRequest {
                username: "User 1".into(),
                email: "User1@mail.ru".into(),
                password: "lksjfskjfsdf?".into(),
            })
            .await
            .unwrap();
        let post = post_rep
            .create(
                (
                    presentation::dto::CreatePostRequest {
                        title: "Post 1".into(),
                        content: "Post 1 content".into(),
                    },
                    user.id as u32,
                )
                    .into(),
            )
            .await
            .unwrap();
        let post = post_rep.find(post.id as u32).await.unwrap();
        assert_eq!(post.id as u32, post.id as u32);
        post_rep
            .update(post.id as u32, vec![Field::Title("New title".to_string())])
            .await
            .unwrap();
        db_manager.drop().await;
    }
}
