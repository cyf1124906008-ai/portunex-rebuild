//! entities/usage_records.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct UsageRecords {
    pub id: i64,
    pub user_id: Option<i64>,
    pub api_key_id: Option<i64>,
    pub provider_id: Option<i64>,
    pub provider: Option<String>,
    pub facade: Option<String>,
    pub model: Option<String>,
    pub mode: Option<String>,
    pub request_id_upstream: Option<String>,
    pub trace_id: Option<String>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub first_response_at: Option<chrono::DateTime<chrono::Utc>>,
    pub first_output_at: Option<chrono::DateTime<chrono::Utc>>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: Option<String>,
    pub upstream_status: Option<String>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
    pub request_bytes: Option<i64>,
    pub response_bytes: Option<i64>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
