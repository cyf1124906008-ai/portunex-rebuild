//! 鉴权提取器:AuthUser(会话)/ AdminUser(role=admin)/ ApiKeyUser(平台 API Key)。
use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use crate::state::AppState;
use portunex_db::repositories::api_key::ApiKeyRepo;

pub struct AuthUser { pub user_id: i64 }
pub struct AdminUser { pub user_id: i64 }
pub struct ApiKeyUser { pub user_id: i64 }

fn bearer(parts: &Parts) -> Option<String> {
    parts.headers.get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .map(|s| s.to_string())
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(parts: &mut Parts, st: &AppState) -> Result<Self, Self::Rejection> {
        let token = bearer(parts).ok_or((StatusCode::UNAUTHORIZED, "missing_token"))?;
        let uid: Option<i64> = sqlx::query_scalar(
            "SELECT user_id FROM auth_sessions WHERE token = $1 AND expires_at > now() AND deleted_at IS NULL")
            .bind(token).fetch_optional(st.pool()).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db"))?;
        uid.map(|user_id| AuthUser { user_id }).ok_or((StatusCode::UNAUTHORIZED, "invalid_token"))
    }
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(parts: &mut Parts, st: &AppState) -> Result<Self, Self::Rejection> {
        let auth = AuthUser::from_request_parts(parts, st).await?;
        let role: Option<String> = sqlx::query_scalar("SELECT role FROM users WHERE id = $1")
            .bind(auth.user_id).fetch_optional(st.pool()).await.map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db"))?;
        if role.as_deref() == Some("admin") { Ok(AdminUser { user_id: auth.user_id }) }
        else { Err((StatusCode::FORBIDDEN, "forbidden")) }
    }
}

impl FromRequestParts<AppState> for ApiKeyUser {
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(parts: &mut Parts, st: &AppState) -> Result<Self, Self::Rejection> {
        let key = bearer(parts).ok_or((StatusCode::UNAUTHORIZED, "missing_api_key"))?;
        let uid = ApiKeyRepo::find_active_user(st.pool(), &key).await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db"))?;
        uid.map(|user_id| ApiKeyUser { user_id }).ok_or((StatusCode::UNAUTHORIZED, "invalid_api_key"))
    }
}
