//! entities/oidc_tokens.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OidcTokens {
    pub id: i64,
    pub token_hash: String,
    pub token_type: String,
    pub client_id: String,
    pub user_id: Option<i64>,
    pub scopes: String,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub parent_id: Option<i64>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
