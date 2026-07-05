//! 计费:兑换码 -> 积分。
use axum::{extract::State, http::StatusCode, Json};
use rust_decimal::Decimal;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use crate::util;
use portunex_db::repositories::redemption::{RedemptionCodeRepo, RedemptionRecordRepo};
use portunex_db::repositories::points::PointsRepo;

type ApiErr = (StatusCode, Json<Value>);
fn err(c: StatusCode, m: &str) -> ApiErr { (c, Json(json!({"error": m}))) }
fn db_err(e: sqlx::Error) -> ApiErr { (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"db","detail":e.to_string()}))) }

#[derive(Deserialize)]
pub struct RedeemReq { pub code: String }

pub async fn redeem(user: AuthUser, State(st): State<AppState>, Json(req): Json<RedeemReq>) -> Result<Json<Value>, ApiErr> {
    let pool = st.pool();
    let code = RedemptionCodeRepo::find_active(pool, &req.code).await.map_err(db_err)?
        .ok_or(err(StatusCode::NOT_FOUND, "code_not_found"))?;
    if RedemptionRecordRepo::has_redeemed(pool, code.id, user.user_id).await.map_err(db_err)? {
        return Err(err(StatusCode::CONFLICT, "already_redeemed"));
    }
    if !RedemptionCodeRepo::try_consume(pool, code.id).await.map_err(db_err)? {
        return Err(err(StatusCode::GONE, "code_exhausted"));
    }
    let mut balance: Option<Decimal> = None;
    if code.reward_type == "points" {
        let pts = code.reward_value.get("points").and_then(|v| v.as_i64()).unwrap_or(0);
        let bal = PointsRepo::add(pool, util::new_id(), user.user_id, Decimal::from(pts), "redeem",
            &format!("redeem:{}", code.code)).await.map_err(db_err)?;
        balance = Some(bal);
    }
    RedemptionRecordRepo::insert(pool, util::new_id(), code.id, user.user_id, &code.reward_type, &code.reward_value)
        .await.map_err(db_err)?;
    Ok(Json(json!({ "ok": true, "reward_type": code.reward_type, "balance": balance })))
}
