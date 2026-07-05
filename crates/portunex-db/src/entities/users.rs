//! entities/users.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Users {
    pub id: i64,
    pub email: Option<String>,
    pub password_phc: Option<String>,
    pub points: Option<rust_decimal::Decimal>,
    pub role: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub can_purchase_subscription: bool,
    pub daily_recharge_limit: rust_decimal::Decimal,
}
