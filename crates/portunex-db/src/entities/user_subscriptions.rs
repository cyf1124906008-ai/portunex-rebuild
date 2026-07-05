//! entities/user_subscriptions.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserSubscriptions {
    pub id: i64,
    pub user_id: i64,
    pub plan_id: i64,
    pub order_id: Option<i64>,
    pub is_special: bool,
    pub level: i32,
    pub status: String,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub daily_limit: Option<rust_decimal::Decimal>,
    pub weekly_limit: Option<rust_decimal::Decimal>,
    pub monthly_limit: Option<rust_decimal::Decimal>,
    pub meta_json: Option<serde_json::Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
