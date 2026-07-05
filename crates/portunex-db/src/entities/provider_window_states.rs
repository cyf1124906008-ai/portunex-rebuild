//! entities/provider_window_states.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProviderWindowStates {
    pub id: i64,
    pub config_id: Option<i64>,
    pub provider_id: Option<i64>,
    pub window_start: Option<chrono::DateTime<chrono::Utc>>,
    pub window_end: Option<chrono::DateTime<chrono::Utc>>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
    pub request_count: Option<i64>,
    pub last_request_at: Option<chrono::DateTime<chrono::Utc>>,
    pub utilization: Option<f64>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}
