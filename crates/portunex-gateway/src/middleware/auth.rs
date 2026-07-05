//! 鉴权提取器:Authorization: Bearer <token> -> 校验 auth_sessions -> AuthUser。
use axum::extract::FromRequestParts;
use axum::http::{request::Parts, StatusCode};
use crate::state::AppState;

pub struct AuthUser {
    pub user_id: i64,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, st: &AppState) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or((StatusCode::UNAUTHORIZED, "missing_token"))?
            .to_string();

        let uid: Option<i64> = sqlx::query_scalar(
            "SELECT user_id FROM auth_sessions \
             WHERE token = $1 AND expires_at > now() AND deleted_at IS NULL",
        )
        .bind(token)
        .fetch_optional(st.pool())
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "db"))?;

        uid.map(|user_id| AuthUser { user_id })
            .ok_or((StatusCode::UNAUTHORIZED, "invalid_token"))
    }
}
