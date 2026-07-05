//! entities/api_keys.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiKeys {
    pub id: i64,
    pub user_id: Option<i64>,
    pub key_text: Option<String>,
    pub prefix: Option<String>,
    pub active: Option<bool>,
    pub settings: Option<serde_json::Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rotated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub name: Option<String>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
}
