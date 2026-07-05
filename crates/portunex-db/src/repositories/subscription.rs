//! 订阅套餐 + 用户订阅仓库。
use crate::entities::subscription_plans::SubscriptionPlans;
use crate::entities::user_subscriptions::UserSubscriptions;
use rust_decimal::Decimal;
use sqlx::PgPool;

pub struct SubscriptionPlanRepo;
impl SubscriptionPlanRepo {
    pub async fn list_active(pool: &PgPool) -> sqlx::Result<Vec<SubscriptionPlans>> {
        sqlx::query_as::<_, SubscriptionPlans>(
            "SELECT * FROM subscription_plans WHERE active = true ORDER BY sort_order NULLS LAST, level")
            .fetch_all(pool).await
    }
    pub async fn find(pool: &PgPool, id: i64) -> sqlx::Result<Option<SubscriptionPlans>> {
        sqlx::query_as::<_, SubscriptionPlans>("SELECT * FROM subscription_plans WHERE id = $1")
            .bind(id).fetch_optional(pool).await
    }
    pub async fn create(pool: &PgPool, id: i64, name: &str, code: &str, price: Decimal, duration_days: i32, level: i32) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO subscription_plans (id, name, code, price, price_cents, duration_days, level, active, is_special) \
             VALUES ($1,$2,$3,$4, ($4*100)::bigint, $5,$6, true, false)")
            .bind(id).bind(name).bind(code).bind(price).bind(duration_days).bind(level)
            .execute(pool).await?;
        Ok(())
    }
}

pub struct UserSubscriptionRepo;
impl UserSubscriptionRepo {
    pub async fn create(pool: &PgPool, id: i64, user_id: i64, plan_id: i64, level: i32, duration_days: i32) -> sqlx::Result<()> {
        sqlx::query(
            "INSERT INTO user_subscriptions (id, user_id, plan_id, is_special, level, status, started_at, expires_at, created_at) \
             VALUES ($1,$2,$3, false, $4,'active', now(), now() + make_interval(days => $5), now())")
            .bind(id).bind(user_id).bind(plan_id).bind(level).bind(duration_days)
            .execute(pool).await?;
        Ok(())
    }
    pub async fn list_active(pool: &PgPool, user_id: i64) -> sqlx::Result<Vec<UserSubscriptions>> {
        sqlx::query_as::<_, UserSubscriptions>(
            "SELECT * FROM user_subscriptions WHERE user_id = $1 AND status='active' AND expires_at > now() ORDER BY expires_at DESC")
            .bind(user_id).fetch_all(pool).await
    }
}
