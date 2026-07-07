//! OIDC 提供方端点:discovery / jwks / token(签发 id_token)。
//! 简化:token 直接给"当前会话用户"签 id_token(真实 OIDC 走 client_id+授权码交换,见 TODO)。
use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::middleware::auth::AuthUser;
use crate::services::oidc;
use crate::state::AppState;
use portunex_db::repositories::user::UserRepo;

fn issuer(st: &AppState) -> String {
    let cfg = &st.settings().oidc.issuer;
    if cfg.is_empty() { "http://localhost:8080".to_string() } else { cfg.clone() }
}
fn now_secs() -> usize {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as usize).unwrap_or(0)
}

pub async fn discovery(State(st): State<AppState>) -> Json<Value> {
    let iss = issuer(&st);
    Json(json!({
        "issuer": iss,
        "jwks_uri": format!("{iss}/oidc/jwks"),
        "authorization_endpoint": format!("{iss}/oidc/authorize"),
        "token_endpoint": format!("{iss}/oidc/token"),
        "userinfo_endpoint": format!("{iss}/oidc/userinfo"),
        "response_types_supported": ["code"],
        "subject_types_supported": ["public"],
        "id_token_signing_alg_values_supported": ["RS256"],
        "scopes_supported": ["openid", "email"]
    }))
}

pub async fn jwks() -> Json<Value> { Json(oidc::jwks()) }

pub async fn token(user: AuthUser, State(st): State<AppState>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let u = UserRepo::find_by_id(st.pool(), user.user_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":e.to_string()}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"error":"user_not_found"}))))?;
    let iss = issuer(&st);
    let ttl = st.settings().oidc.access_token_ttl_secs.max(1) as usize;
    let id_token = oidc::sign_id_token(&iss, user.user_id, "portunex", u.email, ttl, now_secs())
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":e.to_string()}))))?;
    Ok(Json(json!({ "id_token": id_token, "token_type": "Bearer", "expires_in": ttl })))
}

pub async fn userinfo(user: AuthUser, State(st): State<AppState>) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let u = UserRepo::find_by_id(st.pool(), user.user_id).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error":e.to_string()}))))?
        .ok_or((StatusCode::NOT_FOUND, Json(json!({"error":"user_not_found"}))))?;
    Ok(Json(json!({ "sub": u.id.to_string(), "email": u.email, "role": u.role })))
}
