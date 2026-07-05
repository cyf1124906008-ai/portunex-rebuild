//! entities/oidc_clients.rs — 由 information_schema 精确生成。
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OidcClients {
    pub id: i64,
    pub client_id: String,
    pub client_secret_hash: Option<String>,
    pub client_name: String,
    pub redirect_uris: String,
    pub allowed_scopes: String,
    pub grant_types: String,
    pub token_endpoint_auth_method: String,
    pub access_token_ttl_secs: Option<i32>,
    pub refresh_token_ttl_secs: Option<i32>,
    pub active: bool,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub deleted_at: Option<chrono::DateTime<chrono::Utc>>,
}
