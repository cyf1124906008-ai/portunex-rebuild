//! 用户面:/me、/points/stats、/usage。
use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use portunex_db::repositories::user::UserRepo;
use portunex_db::repositories::usage::UsageRepo;

type ApiErr = (StatusCode, Json<Value>);
fn db_err(e: sqlx::Error) -> ApiErr { (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"db","detail":e.to_string()}))) }

pub async fn me(user: AuthUser, State(st): State<AppState>) -> Result<Json<Value>, ApiErr> {
    let u = UserRepo::find_by_id(st.pool(), user.user_id).await.map_err(db_err)?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"error":"not_found"}))))?;
    Ok(Json(json!({ "id": u.id, "email": u.email, "role": u.role, "points": u.points })))
}

pub async fn points_stats(user: AuthUser, State(st): State<AppState>) -> Result<Json<Value>, ApiErr> {
    let u = UserRepo::find_by_id(st.pool(), user.user_id).await.map_err(db_err)?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"error":"not_found"}))))?;
    Ok(Json(json!({ "balance": u.points, "daily_recharge_limit": u.daily_recharge_limit })))
}

pub async fn usage_list(user: AuthUser, State(st): State<AppState>) -> Result<Json<Value>, ApiErr> {
    let rows = UsageRepo::list_by_user(st.pool(), user.user_id, 50).await.map_err(db_err)?;
    let data: Vec<Value> = rows.iter().map(|r| json!({
        "id": r.id, "model": r.model, "provider": r.provider, "trace_id": r.trace_id, "started_at": r.started_at
    })).collect();
    Ok(Json(json!({ "data": data })))
}
