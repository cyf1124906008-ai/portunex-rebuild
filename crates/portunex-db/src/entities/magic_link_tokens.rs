//! entities/magic_link_tokens.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MagicLinkTokens {
    pub id: i64,
    pub email: String,
    pub token: String,
    pub user_id: Option<i64>,
    pub is_new_user: Option<bool>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}
