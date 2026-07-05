//! 积分仓库:加/减积分 + 明细台账(事务)。
use rust_decimal::Decimal;
use sqlx::PgPool;

pub struct PointsRepo;
impl PointsRepo {
    /// 原子加积分:更新 users.points 并写 points_details,返回新余额。
    pub async fn add(pool: &PgPool, id: i64, user_id: i64, delta: Decimal, kind: &str, reason: &str) -> sqlx::Result<Decimal> {
        let mut tx = pool.begin().await?;
        let new_balance: Decimal = sqlx::query_scalar(
            "UPDATE users SET points = points + $1, updated_at = now() WHERE id = $2 RETURNING points",
        )
        .bind(delta).bind(user_id).fetch_one(&mut *tx).await?;
        sqlx::query(
            "INSERT INTO points_details (id, user_id, kind, delta, balance_after, reason, created_at) \
             VALUES ($1,$2,$3,$4,$5,$6, now())",
        )
        .bind(id).bind(user_id).bind(kind).bind(delta).bind(new_balance).bind(reason)
        .execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(new_balance)
    }
}
