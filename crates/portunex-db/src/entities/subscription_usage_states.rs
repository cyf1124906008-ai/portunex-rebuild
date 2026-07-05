//! entities/subscription_usage_states.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubscriptionUsageStates {
    pub id: i64,
    pub subscription_id: i64,
    pub window_type: String,
    pub window_start: chrono::DateTime<chrono::Utc>,
    pub window_end: chrono::DateTime<chrono::Utc>,
    pub used_amount: Option<rust_decimal::Decimal>,
    pub limit_amount: Option<rust_decimal::Decimal>,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
