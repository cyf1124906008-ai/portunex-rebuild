//! 订阅:列出套餐/我的订阅、用积分购买、管理员建套餐。
use axum::{extract::State, http::StatusCode, Json};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::middleware::auth::{AuthUser, AdminUser};
use crate::state::AppState;
use crate::util;
use portunex_db::repositories::user::UserRepo;
use portunex_db::repositories::points::PointsRepo;
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

pub async fn subscribe(user: AuthUser, State(st): State<AppState>, Json(req): Json<SubscribeReq>) -> Result<Json<Value>, ApiErr> {
    let pool = st.pool();
    let plan = SubscriptionPlanRepo::find(pool, req.plan_id).await.map_err(db_err)?
        .ok_or(err(StatusCode::NOT_FOUND, "plan_not_found"))?;
    let price = plan.price;
    let level = plan.level.unwrap_or(0);
    let days = plan.duration_days;
    let u = UserRepo::find_by_id(pool, user.user_id).await.map_err(db_err)?
        .ok_or(err(StatusCode::NOT_FOUND, "user_not_found"))?;
    if u.points.unwrap_or_default() < price {
        return Err(err(StatusCode::PAYMENT_REQUIRED, "insufficient_points"));
    }
    // 扣积分
    let bal = PointsRepo::add(pool, util::new_id(), user.user_id, -price, "subscribe",
        &format!("subscribe:plan:{}", plan.id)).await.map_err(db_err)?;
    let sid = util::new_id();
    UserSubscriptionRepo::create(pool, sid, user.user_id, plan.id, level, days).await.map_err(db_err)?;
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
