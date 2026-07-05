//! entities/providers.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Providers {
    pub id: i64,
    pub kind: Option<String>,
    pub name: Option<String>,
    pub base_url: Option<String>,
    pub http_proxy: Option<String>,
    pub socks5_proxy: Option<String>,
    pub weight: Option<i32>,
    pub healthy: Option<bool>,
    pub last_error: Option<serde_json::Value>,
    pub last_checked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rpm_limit: Option<i32>,
    pub rps_limit: Option<i32>,
    pub max_concurrent: Option<i32>,
    pub available_from: Option<String>,
    pub available_until: Option<String>,
    pub cooldown_until: Option<chrono::DateTime<chrono::Utc>>,
}
