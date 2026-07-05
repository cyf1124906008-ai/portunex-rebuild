//! entities/points_details.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct PointsDetails {
    pub id: i64,
    pub user_id: Option<i64>,
    pub usage_id: Option<i64>,
    pub kind: Option<String>,
    pub delta: rust_decimal::Decimal,
    pub balance_after: Option<rust_decimal::Decimal>,
    pub reason: Option<String>,
    pub pricing_snapshot: Option<serde_json::Value>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
