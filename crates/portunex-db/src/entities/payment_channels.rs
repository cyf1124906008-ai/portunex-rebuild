//! entities/payment_channels.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PaymentChannels {
    pub id: i64,
    pub code: Option<String>,
    pub name: Option<String>,
    pub provider_type: Option<String>,
    pub config_json: Option<serde_json::Value>,
    pub priority: Option<i32>,
    pub active: Option<bool>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
