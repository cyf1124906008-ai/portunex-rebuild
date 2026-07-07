//! 上游 provider 池仓库。
use crate::entities::providers::Providers;
use sqlx::PgPool;
pub struct ProviderRepo;
impl ProviderRepo {
    pub async fn list(pool: &PgPool) -> sqlx::Result<Vec<Providers>> {
        sqlx::query_as::<_, Providers>("SELECT * FROM providers WHERE deleted_at IS NULL ORDER BY id").fetch_all(pool).await
    }
    pub async fn create(pool: &PgPool, id: i64, kind: &str, name: &str, base_url: Option<&str>, weight: i32) -> sqlx::Result<()> {
        sqlx::query("INSERT INTO providers (id, kind, name, base_url, weight, healthy, created_at, updated_at) VALUES ($1,$2,$3,$4,$5, true, now(), now())")
            .bind(id).bind(kind).bind(name).bind(base_url).bind(weight).execute(pool).await?;
        Ok(())
    }
    pub async fn soft_delete(pool: &PgPool, id: i64) -> sqlx::Result<u64> {
        let r = sqlx::query("UPDATE providers SET deleted_at = now() WHERE id = $1 AND deleted_at IS NULL").bind(id).execute(pool).await?;
        Ok(r.rows_affected())
    }
    /// 调度:选一个该 kind 的健康、未冷却的 provider(按 weight 优先,随机 tiebreak)。
    /// TODO: 接入 sticky + P2C + spill(见配置的 ROUTING__*)。
    pub async fn pick_healthy_by_kind(pool: &PgPool, kind: &str) -> sqlx::Result<Option<Providers>> {
        sqlx::query_as::<_, Providers>(
            "SELECT * FROM providers WHERE kind = $1 AND healthy = true AND deleted_at IS NULL \
             AND (cooldown_until IS NULL OR cooldown_until < now()) \
             ORDER BY weight DESC NULLS LAST, random() LIMIT 1")
            .bind(kind).fetch_optional(pool).await
    }
}
