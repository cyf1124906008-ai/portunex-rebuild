//! entities/oidc_consents.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OidcConsents {
    pub id: i64,
    pub user_id: i64,
    pub client_id: String,
    pub scopes: String,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub revoked_at: Option<chrono::DateTime<chrono::Utc>>,
}
