//! 模型别名仓库。
use crate::entities::model_aliases::ModelAliases;
use sqlx::PgPool;
pub struct ModelAliasRepo;
impl ModelAliasRepo {
    pub async fn list(pool: &PgPool) -> sqlx::Result<Vec<ModelAliases>> {
        sqlx::query_as::<_, ModelAliases>("SELECT * FROM model_aliases ORDER BY priority NULLS LAST, alias").fetch_all(pool).await
    }
    pub async fn find_by_alias(pool: &PgPool, alias: &str) -> sqlx::Result<Option<ModelAliases>> {
        sqlx::query_as::<_, ModelAliases>("SELECT * FROM model_aliases WHERE alias = $1 LIMIT 1").bind(alias).fetch_optional(pool).await
    }
    pub async fn create(pool: &PgPool, id: i64, alias: &str, kind: &str, upstream_model: Option<&str>, priority: i32) -> sqlx::Result<()> {
        sqlx::query("INSERT INTO model_aliases (id, alias, kind, upstream_model, priority, created_at, updated_at) VALUES ($1,$2,$3,$4,$5, now(), now())")
            .bind(id).bind(alias).bind(kind).bind(upstream_model).bind(priority).execute(pool).await?;
        Ok(())
    }
    pub async fn delete(pool: &PgPool, id: i64) -> sqlx::Result<u64> {
        let r = sqlx::query("DELETE FROM model_aliases WHERE id = $1").bind(id).execute(pool).await?;
        Ok(r.rows_affected())
    }
}
