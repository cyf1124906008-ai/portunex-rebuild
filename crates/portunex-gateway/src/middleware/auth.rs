//! 鉴权提取器:AuthUser(会话)/ AdminUser(会话 + role=admin)。
use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use crate::state::AppState;

pub struct AuthUser { pub user_id: i64 }
pub struct AdminUser { pub user_id: i64 }

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(parts: &mut Parts, st: &AppState) -> Result<Self, Self::Rejection> {
        let token = parts.headers.get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or((StatusCode::UNAUTHORIZED, "missing_token"))?
            .to_string();
        let uid: Option<i64> = sqlx::query_scalar(
            "SELECT user_id FROM auth_sessions WHERE token = $1 AND expires_at > now() AND deleted_at IS NULL")
            .bind(token).fetch_optional(st.pool()).await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db"))?;
        uid.map(|user_id| AuthUser { user_id }).ok_or((StatusCode::UNAUTHORIZED, "invalid_token"))
    }
}

impl FromRequestParts<AppState> for AdminUser {
    type Rejection = (StatusCode, &'static str);
    async fn from_request_parts(parts: &mut Parts, st: &AppState) -> Result<Self, Self::Rejection> {
        let auth = AuthUser::from_request_parts(parts, st).await?;
        let role: Option<String> = sqlx::query_scalar("SELECT role FROM users WHERE id = $1")
            .bind(auth.user_id).fetch_optional(st.pool()).await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db"))?;
        if role.as_deref() == Some("admin") {
            Ok(AdminUser { user_id: auth.user_id })
        } else {
            Err((StatusCode::FORBIDDEN, "forbidden"))
        }
    }
}
