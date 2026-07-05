//! 认证 handler:注册 / 登录(argon2 + auth_sessions 会话 token)。
use axum::{extract::State, http::StatusCode, Json};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::state::AppState;
use crate::util;
use portunex_db::repositories::user::UserRepo;

type ApiErr = (StatusCode, Json<Value>);
fn err(code: StatusCode, msg: &str) -> ApiErr {
    (code, Json(json!({ "error": msg })))
}
fn db_err(e: sqlx::Error) -> ApiErr {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "db", "detail": e.to_string() })))
}

#[derive(Deserialize)]
pub struct RegisterReq { pub email: String, pub password: String }

#[derive(Deserialize)]
pub struct LoginReq { pub email: String, pub password: String }

pub async fn register(State(st): State<AppState>, Json(req): Json<RegisterReq>) -> Result<Json<Value>, ApiErr> {
    let pool = st.pool();
    if UserRepo::find_by_email(pool, &req.email).await.map_err(db_err)?.is_some() {
        return Err(err(StatusCode::CONFLICT, "email_exists"));
    }
    let phc = util::hash_password(&req.password).map_err(|_| err(StatusCode::INTERNAL_SERVER_ERROR, "hash_failed"))?;
    let id = util::new_id();
    UserRepo::insert(pool, id, &req.email, &phc, "user").await.map_err(db_err)?;
    Ok(Json(json!({ "id": id, "email": req.email, "role": "user" })))
}

pub async fn login(State(st): State<AppState>, Json(req): Json<LoginReq>) -> Result<Json<Value>, ApiErr> {
    let pool = st.pool();
    let user = UserRepo::find_by_email(pool, &req.email).await.map_err(db_err)?
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "invalid_credentials"))?;
    let phc = user.password_phc.as_deref()
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "invalid_credentials"))?;
    if !util::verify_password(&req.password, phc) {
        return Err(err(StatusCode::UNAUTHORIZED, "invalid_credentials"));
    }
    let token = util::random_token(48);
    let sid = util::new_id();
    let expires = chrono::Utc::now() + chrono::Duration::days(7);
    sqlx::query("INSERT INTO auth_sessions (id, user_id, token, expires_at) VALUES ($1, $2, $3, $4)")
        .bind(sid).bind(user.id).bind(&token).bind(expires)
        .execute(pool).await.map_err(db_err)?;
    Ok(Json(json!({
        "token": token,
        "user": { "id": user.id, "email": user.email, "role": user.role }
    })))
}
