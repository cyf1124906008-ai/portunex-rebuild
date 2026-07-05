//! entities/provider_credentials.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProviderCredentials {
    pub id: i64,
    pub provider_id: Option<i64>,
    pub auth_type: Option<String>,
    pub secret: Option<String>,
    pub meta_json: Option<serde_json::Value>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
