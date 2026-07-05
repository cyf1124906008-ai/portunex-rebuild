//! entities/redemption_records.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RedemptionRecords {
    pub id: i64,
    pub code_id: i64,
    pub user_id: i64,
    pub reward_type: String,
    pub reward_snapshot: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub redeemed_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
