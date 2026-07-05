//! entities/subscription_plans.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubscriptionPlans {
    pub id: i64,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub is_special: Option<bool>,
    pub level: Option<i32>,
    pub duration_days: i32,
    pub daily_limit: Option<rust_decimal::Decimal>,
    pub weekly_limit: Option<rust_decimal::Decimal>,
    pub monthly_limit: Option<rust_decimal::Decimal>,
    pub price: rust_decimal::Decimal,
    pub price_cents: i64,
    pub active: Option<bool>,
    pub sort_order: Option<i32>,
    pub meta_json: Option<serde_json::Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
