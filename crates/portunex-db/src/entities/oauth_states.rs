//! entities/oauth_states.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OauthStates {
    pub id: i64,
    pub state: Option<String>,
    pub code_verifier: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub purpose: Option<String>,
    pub binding_user_id: Option<i64>,
}
