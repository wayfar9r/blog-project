use crate::domain::user::User;
use crate::presentation;
use std::fmt::Display;
use std::sync::Arc;

use chrono::Utc;
use sqlx::Postgres;
use sqlx::postgres::PgPool;

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("Not found. {0}")]
    NotFound(String),
    #[error("Internal error. {0}")]
    InternalError(String),
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Field::Id(_) => "id",
                Field::Username(_) => "username",
                Field::Email(_) => "email",
                Field::Password(_) => "password",
                Field::CreatedAt(_) => "created_at",
            }
        )
    }
}

#[derive(Debug, Hash)]
pub enum Field {
    Id(u32),
    Username(String),
    Email(String),
    Password(String),
    CreatedAt(chrono::DateTime<Utc>),
}

pub struct UserRepository {
    pool: Arc<PgPool>,
}

impl UserRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        UserRepository { pool }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as!(User, "SELECT * FROM USERS where email = $1;", email)
            .fetch_one(&*self.pool)
            .await
    }

    pub async fn find_by_username(&self, username: &str) -> Result<User, sqlx::Error> {
        sqlx::query_as!(User, "SELECT * FROM USERS WHERE username = $1;", username)
            .fetch_one(&*self.pool)
            .await
    }

    pub async fn find_by_any(&self, fields: Vec<Field>) -> Result<User, sqlx::Error> {
        if fields.is_empty() {
            return Err(sqlx::Error::InvalidArgument("empty fields".into()));
        }
        let mut builder = sqlx::QueryBuilder::<Postgres>::new("SELECT * FROM users WHERE ");
        let mut separated_builder = builder.separated(" or ");
        for f in fields {
            separated_builder.push(format!("{}=", f));
            match f {
                Field::Email(email) => {
                    separated_builder.push_bind_unseparated(email);
                }
                Field::Username(username) => {
                    separated_builder.push_bind_unseparated(username);
                }
                _ => {
                    return Err(sqlx::Error::InvalidArgument(format!(
                        "field is not supported"
                    )));
                }
            }
        }
        let query = builder.build_query_as();
        query.fetch_one(&*self.pool).await
    }
}

impl UserRepository {
    pub async fn create(
        &self,
        fields: presentation::dto::RegisterRequest,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING *;",
            fields.username,
            fields.email,
            fields.password
        )
        .fetch_one(&*self.pool)
        .await
    }
    pub async fn delete(&self, id: u32) -> Result<(), sqlx::Error> {
        match sqlx::query!("DELETE FROM users WHERE id = $1;", id as i64)
            .execute(&*self.pool)
            .await
        {
            Ok(r) => {
                if r.rows_affected() > 0 {
                    Ok(())
                } else {
                    Err(sqlx::Error::RowNotFound)
                }
            }
            Err(e) => Err(e),
        }
    }
    pub async fn find(&self, id: u32) -> Result<User, sqlx::Error> {
        sqlx::query_as!(
            User,
            "SELECT id, username, email, password_hash, created_at FROM users WHERE id = $1;",
            id as i64
        )
        .fetch_one(&*self.pool)
        .await
    }
    pub async fn update(&self, _id: u32, _fields: Vec<Field>) -> Result<User, sqlx::Error> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::test::{TestDbManager, create_test_db_pool};

    #[actix_web::test]
    #[ignore = "one-thread-test"]
    async fn should_crud_users() {
        let db_manager = TestDbManager::new().await;
        db_manager.create().await;
        let db_pool = Arc::new(create_test_db_pool().await);
        let user_rep = UserRepository::new(db_pool);
        let created_user = user_rep
            .create(presentation::dto::RegisterRequest {
                username: "User 1".into(),
                email: "User1@email.ru".into(),
                password: "lskdfjkljskljsdf".into(),
            })
            .await
            .unwrap();
        let user = user_rep.find(created_user.id as u32).await.unwrap();
        assert_eq!(user.id, created_user.id as i64);
        let user2 = user_rep.find_by_email(&user.email).await.unwrap();
        assert_eq!(user, user2);
        user_rep.delete(user.id as u32).await.unwrap();
        db_manager.drop().await;
    }

    #[actix_web::test]
    #[ignore = "one-thread-test"]
    async fn should_find_by_any_field() {
        let db_manager = TestDbManager::new().await;
        db_manager.create().await;
        let db_pool = Arc::new(create_test_db_pool().await);
        let user_rep = UserRepository::new(db_pool);
        let _user = user_rep
            .create(presentation::dto::RegisterRequest {
                username: "User 1".into(),
                email: "User1@email.ru".into(),
                password: "lskdfjkljskljsdf".into(),
            })
            .await
            .unwrap();
        let _user = user_rep
            .find_by_any(vec![
                Field::Username("User 1".into()),
                Field::Email("dskjfsf".into()),
            ])
            .await
            .unwrap();
        db_manager.drop().await;
    }
}
