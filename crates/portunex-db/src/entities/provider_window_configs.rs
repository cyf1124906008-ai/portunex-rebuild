//! entities/provider_window_configs.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ProviderWindowConfigs {
    pub id: i64,
    pub provider_kind: Option<String>,
    pub window_type: Option<String>,
    pub window_seconds: Option<i64>,
    pub stat_type: Option<String>,
    pub limit_value: Option<i64>,
    pub model_pattern: Option<String>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
