//! API Key handler:创建 / 列表 / 停用(均需 Bearer 会话)。
use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use crate::util;
use portunex_db::repositories::api_key::ApiKeyRepo;

type ApiErr = (StatusCode, Json<Value>);
fn db_err(e: sqlx::Error) -> ApiErr {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"db","detail":e.to_string()})))
}

#[derive(Deserialize)]
pub struct CreateReq { #[serde(default)] pub name: String }

/// 创建后**仅此一次**返回完整 key。
pub async fn create(user: AuthUser, State(st): State<AppState>, Json(req): Json<CreateReq>) -> Result<Json<Value>, ApiErr> {
    let key = format!("sk-{}", util::random_token(40));
    let prefix = key.chars().take(11).collect::<String>();
    let id = util::new_id();
    let name = if req.name.is_empty() { "default".to_string() } else { req.name };
    ApiKeyRepo::create(st.pool(), id, user.user_id, &key, &prefix, &name).await.map_err(db_err)?;
    Ok(Json(json!({ "id": id, "name": name, "key": key, "prefix": prefix })))
}

/// 列表:不回传完整 key,只给 prefix/name/active。
pub async fn list(user: AuthUser, State(st): State<AppState>) -> Result<Json<Value>, ApiErr> {
    let keys = ApiKeyRepo::list_by_user(st.pool(), user.user_id).await.map_err(db_err)?;
    let items: Vec<Value> = keys.iter().map(|k| json!({
        "id": k.id, "name": k.name, "prefix": k.prefix, "active": k.active, "created_at": k.created_at
    })).collect();
    Ok(Json(json!({ "data": items })))
}

pub async fn deactivate(user: AuthUser, State(st): State<AppState>, Path(id): Path<i64>) -> Result<Json<Value>, ApiErr> {
    let n = ApiKeyRepo::deactivate(st.pool(), user.user_id, id).await.map_err(db_err)?;
    if n == 0 { return Err((StatusCode::NOT_FOUND, Json(json!({"error":"not_found"})))); }
    Ok(Json(json!({ "ok": true })))
}
