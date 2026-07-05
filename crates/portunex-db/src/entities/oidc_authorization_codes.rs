//! entities/oidc_authorization_codes.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OidcAuthorizationCodes {
    pub id: i64,
    pub code: String,
    pub client_id: String,
    pub user_id: i64,
    pub redirect_uri: String,
    pub scopes: String,
    pub nonce: Option<String>,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub consumed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
}
