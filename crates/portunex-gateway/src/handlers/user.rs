//! 用户面 handler:/me。
use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use portunex_db::repositories::user::UserRepo;

pub async fn me(user: AuthUser, State(st): State<AppState>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let u = UserRepo::find_by_id(st.pool(), user.user_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"db","detail":e.to_string()}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"error":"not_found"}))))?;
    Ok(Json(json!({
        "id": u.id, "email": u.email, "role": u.role, "points": u.points
    })))
}
