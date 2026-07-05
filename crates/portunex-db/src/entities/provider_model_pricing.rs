//! entities/provider_model_pricing.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProviderModelPricing {
    pub id: i64,
    pub provider_id: Option<i64>,
    pub model_id: Option<String>,
    pub input_per_1k: Option<rust_decimal::Decimal>,
    pub output_per_1k: Option<rust_decimal::Decimal>,
    pub cache_create_per_1k: Option<rust_decimal::Decimal>,
    pub cache_read_per_1k: Option<rust_decimal::Decimal>,
    pub cache_1h_per_1k: Option<rust_decimal::Decimal>,
    pub effective_from: Option<chrono::DateTime<chrono::Utc>>,
    pub effective_to: Option<chrono::DateTime<chrono::Utc>>,
    pub priority: Option<i32>,
}
