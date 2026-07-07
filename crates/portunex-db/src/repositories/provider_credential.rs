//! provider 凭据仓库(上游认证 secret)。
use sqlx::PgPool;
pub struct ProviderCredentialRepo;
impl ProviderCredentialRepo {
    /// 取该 provider 的一条凭据 -> (auth_type, secret)。
    pub async fn find_for_provider(pool: &PgPool, provider_id: i64) -> sqlx::Result<Option<(String, String)>> {
        let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
            "SELECT auth_type, secret FROM provider_credentials WHERE provider_id = $1 AND deleted_at IS NULL ORDER BY id LIMIT 1")
            .bind(provider_id).fetch_optional(pool).await?;
        Ok(row.map(|(a, s)| (a.unwrap_or_else(|| "api_key".into()), s.unwrap_or_default())))
    }
}
