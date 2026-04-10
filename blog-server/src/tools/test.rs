use sqlx::Postgres;

use crate::infrastructure::database;

pub async fn create_test_db_pool() -> sqlx::Pool<Postgres> {
    let conn_url = dotenvy::var("TEST_DATABASE_URL").unwrap();
    database::create_pool(&conn_url).await.unwrap()
}

pub struct TestDbManager {
    pool: sqlx::Pool<Postgres>,
}

impl TestDbManager {
    pub async fn new() -> TestDbManager {
        let pool = create_test_db_pool().await;
        TestDbManager { pool }
    }

    pub async fn create(&self) {
        database::run_migrations(&self.pool).await.unwrap();
    }

    pub async fn drop(&self) {
        sqlx::query!("DROP TABLE posts;")
            .execute(&self.pool)
            .await
            .unwrap();
        sqlx::query!("DROP TABLE users;")
            .execute(&self.pool)
            .await
            .unwrap();
        sqlx::query!("DROP TABLE _sqlx_migrations;")
            .execute(&self.pool)
            .await
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_web::test]
    async fn test_should_create_then_drop() {
        let db_manager = TestDbManager::new().await;
        db_manager.create().await;
        db_manager.drop().await;

    }
}
