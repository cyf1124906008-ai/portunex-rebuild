//! entities/orders.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Orders {
    pub id: i64,
    pub order_no: Option<String>,
    pub user_id: Option<i64>,
    pub channel_id: Option<i64>,
    pub product_type: Option<String>,
    pub product_info: Option<serde_json::Value>,
    pub amount: Option<rust_decimal::Decimal>,
    pub amount_cents: Option<i64>,
    pub status: Option<String>,
    pub payment_url: Option<String>,
    pub payment_meta: Option<serde_json::Value>,
    pub settled: Option<bool>,
    pub settled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub paid_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
