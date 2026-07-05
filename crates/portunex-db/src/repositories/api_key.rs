//! API Key 仓库。
use crate::entities::api_keys::ApiKeys;
use sqlx::PgPool;

pub struct ApiKeyRepo;

impl ApiKeyRepo {
    pub async fn create(
        pool: &PgPool, id: i64, user_id: i64, key_text: &str, prefix: &str, name: &str,
    ) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO api_keys (id, user_id, key_text, prefix, active, name, settings) \
             VALUES ($1, $2, $3, $4, true, $5, '{}'::jsonb)",
        )
        .bind(id).bind(user_id).bind(key_text).bind(prefix).bind(name)
        .execute(pool).await?;
        Ok(())
    }

    pub async fn list_by_user(pool: &PgPool, user_id: i64) -> sqlx::Result<Vec<ApiKeys>> {
        sqlx::query_as::<_, ApiKeys>(
            "SELECT * FROM api_keys WHERE user_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(user_id).fetch_all(pool).await
    }

    pub async fn deactivate(pool: &PgPool, user_id: i64, key_id: i64) -> sqlx::Result<u64> {
        let r = sqlx::query(
            "UPDATE api_keys SET active = false WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL",
        )
        .bind(key_id).bind(user_id).execute(pool).await?;
        Ok(r.rows_affected())
    }

    /// 供网关鉴权:凭 key 找到 active 用户。
    pub async fn find_active_user(pool: &PgPool, key_text: &str) -> sqlx::Result<Option<i64>> {
        sqlx::query_scalar(
            "SELECT user_id FROM api_keys WHERE key_text = $1 AND active = true AND deleted_at IS NULL",
        )
        .bind(key_text).fetch_optional(pool).await
    }
}
