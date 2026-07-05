//! entities/subscription_consume_details.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubscriptionConsumeDetails {
    pub id: i64,
    pub subscription_id: i64,
    pub usage_id: Option<i64>,
    pub amount: rust_decimal::Decimal,
    pub reason: Option<String>,
    pub daily_state_id: Option<i64>,
    pub weekly_state_id: Option<i64>,
    pub monthly_state_id: Option<i64>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub pricing_snapshot: Option<serde_json::Value>,
}
