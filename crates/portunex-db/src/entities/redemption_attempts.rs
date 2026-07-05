//! entities/redemption_attempts.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct RedemptionAttempts {
    pub id: i64,
    pub ip_address: String,
    pub attempted_code: Option<String>,
    pub success: bool,
    pub attempted_at: chrono::DateTime<chrono::Utc>,
}
