//! 上游 provider 池仓库。
use crate::entities::providers::Providers;
use sqlx::PgPool;

pub struct ProviderRepo;
impl ProviderRepo {
    pub async fn list(pool: &PgPool) -> sqlx::Result<Vec<Providers>> {
        sqlx::query_as::<_, Providers>(
            "SELECT * FROM providers WHERE deleted_at IS NULL ORDER BY id",
        ).fetch_all(pool).await
    }
    pub async fn create(pool: &PgPool, id: i64, kind: &str, name: &str, base_url: Option<&str>, weight: i32) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO providers (id, kind, name, base_url, weight, healthy, created_at, updated_at) \
             VALUES ($1,$2,$3,$4,$5, true, now(), now())",
        )
        .bind(id).bind(kind).bind(name).bind(base_url).bind(weight)
        .execute(pool).await?;
        Ok(())
    }
    pub async fn soft_delete(pool: &PgPool, id: i64) -> sqlx::Result<u64> {
        let r = sqlx::query("UPDATE providers SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL")
            .bind(id).execute(pool).await?;
        Ok(r.rows_affected())
    }
}
