//! 订阅:列出套餐/我的订阅、用积分购买(单事务,失败回滚)、管理员建套餐。
use axum::{extract::State, http::StatusCode, Json};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::middleware::auth::{AuthUser, AdminUser};
use crate::state::AppState;
use crate::util;
use portunex_db::repositories::subscription::{SubscriptionPlanRepo, UserSubscriptionRepo};

type ApiErr = (StatusCode, Json<Value>);
fn err(c: StatusCode, m: &str) -> ApiErr { (c, Json(json!({"error": m}))) }
fn db_err(e: sqlx::Error) -> ApiErr { (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"db","detail":e.to_string()}))) }

pub async fn list(user: AuthUser, State(st): State<AppState>) -> Result<Json<Value>, ApiErr> {
    let plans = SubscriptionPlanRepo::list_active(st.pool()).await.map_err(db_err)?;
    let mine = UserSubscriptionRepo::list_active(st.pool(), user.user_id).await.map_err(db_err)?;
    let plans_j: Vec<Value> = plans.iter().map(|p| json!({
        "id": p.id, "name": p.name, "code": p.code, "price": p.price, "duration_days": p.duration_days, "level": p.level
    })).collect();
    let mine_j: Vec<Value> = mine.iter().map(|s| json!({
        "id": s.id, "plan_id": s.plan_id, "level": s.level, "status": s.status, "expires_at": s.expires_at
    })).collect();
    Ok(Json(json!({ "plans": plans_j, "active": mine_j })))
}

#[derive(Deserialize)]
pub struct SubscribeReq { pub plan_id: i64 }

/// 用积分购买订阅 —— 扣分 + 建订阅在**单个事务**内,任一步失败整体回滚(不丢积分)。
pub async fn subscribe(user: AuthUser, State(st): State<AppState>, Json(req): Json<SubscribeReq>) -> Result<Json<Value>, ApiErr> {
    let pool = st.pool();
    let plan = SubscriptionPlanRepo::find(pool, req.plan_id).await.map_err(db_err)?
        .ok_or(err(StatusCode::NOT_FOUND, "plan_not_found"))?;
    let price = plan.price;
    let level = plan.level.unwrap_or(0);
    let days = plan.duration_days;

    // 前置检查:已有活跃订阅则直接 409(扣分前拦截,避免依赖事务回滚)
    let existing = UserSubscriptionRepo::list_active(pool, user.user_id).await.map_err(db_err)?;
    if !existing.is_empty() {
        return Err(err(StatusCode::CONFLICT, "already_subscribed"));
    }

    let mut tx = pool.begin().await.map_err(db_err)?;
    let cur: Decimal = sqlx::query_scalar("SELECT COALESCE(points,0) FROM users WHERE id = $1 FOR UPDATE")
        .bind(user.user_id).fetch_one(&mut *tx).await.map_err(db_err)?;
    if cur < price {
        return Err(err(StatusCode::PAYMENT_REQUIRED, "insufficient_points"));
    }
    let bal: Decimal = sqlx::query_scalar(
        "UPDATE users SET points = COALESCE(points,0) - $1, updated_at = now() WHERE id = $2 RETURNING points")
        .bind(price).bind(user.user_id).fetch_one(&mut *tx).await.map_err(db_err)?;
    sqlx::query(
        "INSERT INTO points_details (id, user_id, kind, delta, balance_after, reason, created_at) \
         VALUES ($1,$2,'subscribe',$3,$4,$5, now())")
        .bind(util::new_id()).bind(user.user_id).bind(-price).bind(bal)
        .bind(format!("subscribe:plan:{}", plan.id)).execute(&mut *tx).await.map_err(db_err)?;
    let sid = util::new_id();
    match sqlx::query(
        "INSERT INTO user_subscriptions (id, user_id, plan_id, is_special, level, status, started_at, expires_at, created_at) \
         VALUES ($1,$2,$3, false, $4,'active', now(), now() + make_interval(days => $5), now())")
        .bind(sid).bind(user.user_id).bind(plan.id).bind(level).bind(days)
        .execute(&mut *tx).await
    {
        Ok(_) => { tx.commit().await.map_err(db_err)?; }
        Err(e) => {
            // 显式回滚:恢复本事务里扣掉的积分
            let _ = tx.rollback().await;
            // 23505 = unique_violation -> 已有活跃订阅
            if e.as_database_error().and_then(|d| d.code()).as_deref() == Some("23505") {
                return Err(err(StatusCode::CONFLICT, "already_subscribed"));
            }
            return Err(db_err(e));
        }
    }
    Ok(Json(json!({ "ok": true, "subscription_id": sid, "balance": bal, "expires_in_days": days })))
}

#[derive(Deserialize)]
pub struct CreatePlanReq {
    pub name: String, pub code: String,
    #[serde(default)] pub price: i64,
    #[serde(default = "d30")] pub duration_days: i32,
    #[serde(default)] pub level: i32,
}
fn d30() -> i32 { 30 }
pub async fn admin_create_plan(_a: AdminUser, State(st): State<AppState>, Json(req): Json<CreatePlanReq>) -> Result<Json<Value>, ApiErr> {
    let id = util::new_id();
    SubscriptionPlanRepo::create(st.pool(), id, &req.name, &req.code, Decimal::from(req.price), req.duration_days, req.level)
        .await.map_err(db_err)?;
    Ok(Json(json!({ "id": id, "name": req.name, "code": req.code, "price": req.price })))
}
