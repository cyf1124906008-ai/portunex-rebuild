//! 管理后台 handler(均需 AdminUser)。
use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::middleware::auth::AdminUser;
use crate::state::AppState;
use crate::util;
use portunex_db::repositories::redemption::RedemptionCodeRepo;
use portunex_db::repositories::provider::ProviderRepo;
use portunex_db::repositories::model_alias::ModelAliasRepo;

type ApiErr = (StatusCode, Json<Value>);
fn db_err(e: sqlx::Error) -> ApiErr { (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":"db","detail":e.to_string()}))) }

pub async fn list_users(_a: AdminUser, State(st): State<AppState>) -> Result<Json<Value>, ApiErr> {
    let rows = sqlx::query_as::<_, (i64, Option<String>, Option<String>)>(
        "SELECT id, email, role FROM users WHERE deleted_at IS NULL ORDER BY id LIMIT 200")
        .fetch_all(st.pool()).await.map_err(db_err)?;
    let data: Vec<Value> = rows.iter().map(|(id,email,role)| json!({"id":id,"email":email,"role":role})).collect();
    Ok(Json(json!({ "data": data })))
}

#[derive(Deserialize)]
pub struct CreateCodeReq {
    #[serde(default)] pub name: String,
    #[serde(default)] pub points: i64,
    pub max_uses: Option<i32>,
    pub code: Option<String>,
}
pub async fn create_redemption_code(a: AdminUser, State(st): State<AppState>, Json(req): Json<CreateCodeReq>) -> Result<Json<Value>, ApiErr> {
    let code = req.code.unwrap_or_else(|| format!("RC-{}", util::random_token(12)));
    let id = util::new_id();
    RedemptionCodeRepo::create(st.pool(), id, &code, &req.name, "points",
        &json!({"points": req.points}), req.max_uses, a.user_id).await.map_err(db_err)?;
    Ok(Json(json!({ "id": id, "code": code, "points": req.points, "max_uses": req.max_uses })))
}

pub async fn list_providers(_a: AdminUser, State(st): State<AppState>) -> Result<Json<Value>, ApiErr> {
    let ps = ProviderRepo::list(st.pool()).await.map_err(db_err)?;
    let data: Vec<Value> = ps.iter().map(|p| json!({
        "id":p.id,"kind":p.kind,"name":p.name,"base_url":p.base_url,"weight":p.weight,"healthy":p.healthy
    })).collect();
    Ok(Json(json!({ "data": data })))
}
#[derive(Deserialize)]
pub struct CreateProviderReq { pub kind: String, pub name: String, pub base_url: Option<String>, #[serde(default="one")] pub weight: i32 }
fn one() -> i32 { 1 }
pub async fn create_provider(_a: AdminUser, State(st): State<AppState>, Json(req): Json<CreateProviderReq>) -> Result<Json<Value>, ApiErr> {
    let id = util::new_id();
    ProviderRepo::create(st.pool(), id, &req.kind, &req.name, req.base_url.as_deref(), req.weight).await.map_err(db_err)?;
    Ok(Json(json!({ "id": id, "kind": req.kind, "name": req.name })))
}
pub async fn delete_provider(_a: AdminUser, State(st): State<AppState>, Path(id): Path<i64>) -> Result<Json<Value>, ApiErr> {
    let n = ProviderRepo::soft_delete(st.pool(), id).await.map_err(db_err)?;
    if n == 0 { return Err((StatusCode::NOT_FOUND, Json(json!({"error":"not_found"})))); }
    Ok(Json(json!({ "ok": true })))
}

pub async fn list_model_aliases(_a: AdminUser, State(st): State<AppState>) -> Result<Json<Value>, ApiErr> {
    let al = ModelAliasRepo::list(st.pool()).await.map_err(db_err)?;
    let data: Vec<Value> = al.iter().map(|a| json!({
        "id":a.id,"alias":a.alias,"kind":a.kind,"upstream_model":a.upstream_model,"priority":a.priority
    })).collect();
    Ok(Json(json!({ "data": data })))
}
#[derive(Deserialize)]
pub struct CreateAliasReq { pub alias: String, pub kind: String, pub upstream_model: Option<String>, #[serde(default)] pub priority: i32 }
pub async fn create_model_alias(_a: AdminUser, State(st): State<AppState>, Json(req): Json<CreateAliasReq>) -> Result<Json<Value>, ApiErr> {
    let id = util::new_id();
    ModelAliasRepo::create(st.pool(), id, &req.alias, &req.kind, req.upstream_model.as_deref(), req.priority).await.map_err(db_err)?;
    Ok(Json(json!({ "id": id, "alias": req.alias })))
}
pub async fn delete_model_alias(_a: AdminUser, State(st): State<AppState>, Path(id): Path<i64>) -> Result<Json<Value>, ApiErr> {
    let n = ModelAliasRepo::delete(st.pool(), id).await.map_err(db_err)?;
    if n == 0 { return Err((StatusCode::NOT_FOUND, Json(json!({"error":"not_found"})))); }
    Ok(Json(json!({ "ok": true })))
}
