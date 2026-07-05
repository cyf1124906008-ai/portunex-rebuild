//! 兑换码 + 兑换记录仓库。
use crate::entities::redemption_codes::RedemptionCodes;
use sqlx::PgPool;

pub struct RedemptionCodeRepo;
impl RedemptionCodeRepo {
    pub async fn create(
        pool: &PgPool, id: i64, code: &str, name: &str, reward_type: &str,
        reward_value: &serde_json::Value, max_uses: Option<i32>, created_by: i64,
    ) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO redemption_codes \
             (id, code, name, reward_type, reward_value, max_uses, used_count, created_by, created_at, updated_at) \
             VALUES ($1,$2,$3,$4,$5,$6,0,$7, now(), now())",
        )
        .bind(id).bind(code).bind(name).bind(reward_type).bind(reward_value).bind(max_uses).bind(created_by)
        .execute(pool).await?;
        Ok(())
    }
    pub async fn find_active(pool: &PgPool, code: &str) -> sqlx::Result<Option<RedemptionCodes>> {
        sqlx::query_as::<_, RedemptionCodes>(
            "SELECT * FROM redemption_codes WHERE code = $1 AND deleted_at IS NULL \
             AND (valid_from IS NULL OR valid_from <= now()) \
             AND (valid_until IS NULL OR valid_until >= now())",
        )
        .bind(code).fetch_optional(pool).await
    }
    /// 原子自增 used_count(受 max_uses 约束);返回是否成功占用一次。
    pub async fn try_consume(pool: &PgPool, id: i64) -> sqlx::Result<bool> {
        let r = sqlx::query(
            "UPDATE redemption_codes SET used_count = used_count + 1, updated_at = now() \
             WHERE id = $1 AND (max_uses IS NULL OR used_count < max_uses)",
        )
        .bind(id).execute(pool).await?;
        Ok(r.rows_affected() == 1)
    }
}

pub struct RedemptionRecordRepo;
impl RedemptionRecordRepo {
    pub async fn has_redeemed(pool: &PgPool, code_id: i64, user_id: i64) -> sqlx::Result<bool> {
        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM redemption_records WHERE code_id = $1 AND user_id = $2",
        )
        .bind(code_id).bind(user_id).fetch_one(pool).await?;
        Ok(n > 0)
    }
    pub async fn insert(
        pool: &PgPool, id: i64, code_id: i64, user_id: i64, reward_type: &str,
        snapshot: &serde_json::Value,
    ) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO redemption_records \
             (id, code_id, user_id, reward_type, reward_snapshot, redeemed_at, created_at) \
             VALUES ($1,$2,$3,$4,$5, now(), now())",
        )
        .bind(id).bind(code_id).bind(user_id).bind(reward_type).bind(snapshot)
        .execute(pool).await?;
        Ok(())
    }
}
