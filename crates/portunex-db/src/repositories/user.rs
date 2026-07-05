//! 用户仓库。
use crate::entities::users::Users;
use sqlx::PgPool;

pub struct UserRepo;

impl UserRepo {
    pub async fn count(pool: &PgPool) -> sqlx::Result<i64> {
        sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(pool).await
    }

    pub async fn find_by_email(pool: &PgPool, email: &str) -> sqlx::Result<Option<Users>> {
        sqlx::query_as::<_, Users>(
            "SELECT * FROM users WHERE email = $1 AND deleted_at IS NULL",
        )
        .bind(email)
        .fetch_optional(pool)
        .await
    }

    pub async fn find_by_id(pool: &PgPool, id: i64) -> sqlx::Result<Option<Users>> {
        sqlx::query_as::<_, Users>("SELECT * FROM users WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn insert(
        pool: &PgPool,
        id: i64,
        email: &str,
        password_phc: &str,
        role: &str,
    ) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO users (id, email, password_phc, role, points) VALUES ($1, $2, $3, $4, 0)",
        )
        .bind(id)
        .bind(email)
        .bind(password_phc)
        .bind(role)
        .execute(pool)
        .await?;
        Ok(())
    }
}
