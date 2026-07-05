//! 用量记录仓库(读)。
use crate::entities::usage_records::UsageRecords;
use sqlx::PgPool;
pub struct UsageRepo;
impl UsageRepo {
    pub async fn list_by_user(pool: &PgPool, user_id: i64, limit: i64) -> sqlx::Result<Vec<UsageRecords>> {
        sqlx::query_as::<_, UsageRecords>(
            "SELECT * FROM usage_records WHERE user_id = $1 ORDER BY started_at DESC LIMIT $2")
            .bind(user_id).bind(limit).fetch_all(pool).await
    }
}
