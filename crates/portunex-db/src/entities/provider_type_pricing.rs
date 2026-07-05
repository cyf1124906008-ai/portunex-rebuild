//! entities/provider_type_pricing.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProviderTypePricing {
    pub id: i64,
    pub kind: String,
    pub auth_type: Option<String>,
    pub model_id: Option<String>,
    pub input_per_1k: Option<rust_decimal::Decimal>,
    pub output_per_1k: Option<rust_decimal::Decimal>,
    pub cache_create_per_1k: Option<rust_decimal::Decimal>,
    pub cache_read_per_1k: Option<rust_decimal::Decimal>,
    pub cache_1h_per_1k: Option<rust_decimal::Decimal>,
    pub multiplier: Option<rust_decimal::Decimal>,
    pub effective_from: Option<chrono::DateTime<chrono::Utc>>,
    pub effective_to: Option<chrono::DateTime<chrono::Utc>>,
    pub priority: Option<i32>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
