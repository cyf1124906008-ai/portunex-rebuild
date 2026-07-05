//! 配置系统 —— 还原自原实例的 `PORTUNEX__<SECTION>__<KEY>` 环境变量层级。
//! 用 `config` crate,分隔符 `__`,前缀 `PORTUNEX`。默认值取自原 docker-compose 观测。
//! 注:仅覆盖已观测到的字段;深层功能开关按需补充。

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub db: DbConfig,
    #[serde(default)]
    pub ids: IdsConfig,
    #[serde(default)]
    pub routing: RoutingConfig,
    #[serde(default)]
    pub redis: RedisConfig,
    #[serde(default)]
    pub clickhouse: ClickhouseConfig,
    #[serde(default)]
    pub oidc: OidcConfig,
    #[serde(default)]
    pub anthropic: AnthropicConfig,
    #[serde(default)]
    pub openai: OpenAiConfig,
    #[serde(default)]
    pub mail: MailConfig,
    #[serde(default)]
    pub captcha: CaptchaConfig,
    #[serde(default)]
    pub media: MediaConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "d_host")]
    pub host: String,
    #[serde(default = "d_port")]
    pub port: u16,
    #[serde(default = "d_log_dir")]
    pub log_dir: String,
    #[serde(default = "d_instance")]
    pub instance_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DbConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct IdsConfig {
    #[serde(default = "d_worker")]
    pub worker_id: u16,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RoutingConfig {
    #[serde(default = "d_sticky_ttl")]
    pub sticky_ttl_hours: u32,
    #[serde(default)]
    pub load_aware_spill: String,      // 逗号分隔的 provider-kind
    #[serde(default)]
    pub load_aware_placement: String,
    #[serde(default)]
    pub even_no_sticky: String,
    #[serde(default = "d_spill_wait")]
    pub concurrency_spill_wait_ms: u64,
    #[serde(default)]
    pub affine_wait_ms: u64,
    #[serde(default)]
    pub sticky_follow_spill: bool,
    #[serde(default = "d_p2c")]
    pub p2c_choices: u8,
    #[serde(default)]
    pub flood_sentinel_session_threshold: u32,
    #[serde(default = "d_headroom")]
    pub concurrency_headroom_reserve: u32,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct RedisConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub password: String,
    #[serde(default = "d_pool")]
    pub pool_size: u32,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct ClickhouseConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub database: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub retention_days: u32,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OidcConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub issuer: String,
    #[serde(default)]
    pub key_id: String,
    #[serde(default = "d_ac_ttl")]
    pub access_token_ttl_secs: u64,
    #[serde(default = "d_rt_ttl")]
    pub refresh_token_ttl_secs: u64,
    #[serde(default = "d_code_ttl")]
    pub auth_code_ttl_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AnthropicConfig {
    #[serde(default)]
    pub downgrade_1h_cache_to_5m: bool,
    #[serde(default = "d_true")]
    pub expand_tool_references: bool,
    #[serde(default = "d_ws_idle")]
    pub ws_idle_ttl_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OpenAiConfig {
    #[serde(default)]
    pub ws_openai_beta: String,
    #[serde(default = "d_ws_idle")]
    pub ws_idle_ttl_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MailConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub sender_email: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct CaptchaConfig {
    #[serde(default)]
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct MediaConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "d_media_backend")]
    pub backend: String,
    #[serde(default = "d_media_dir")]
    pub storage_dir: String,
}

fn d_host() -> String { "0.0.0.0".into() }
fn d_port() -> u16 { 8080 }
fn d_log_dir() -> String { "/app/logs".into() }
fn d_instance() -> String { "blue".into() }
fn d_worker() -> u16 { 1 }
fn d_sticky_ttl() -> u32 { 24 }
fn d_spill_wait() -> u64 { 500 }
fn d_p2c() -> u8 { 2 }
fn d_headroom() -> u32 { 2 }
fn d_pool() -> u32 { 4 }
fn d_ac_ttl() -> u64 { 3600 }
fn d_rt_ttl() -> u64 { 86400 }
fn d_code_ttl() -> u64 { 600 }
fn d_true() -> bool { true }
fn d_ws_idle() -> u64 { 30 }
fn d_media_backend() -> String { "local".into() }
fn d_media_dir() -> String { "/app/logs/media".into() }

impl Settings {
    /// 从环境变量加载:`PORTUNEX__SERVER__PORT=8080` → `settings.server.port`。
    pub fn load() -> anyhow::Result<Self> {
        let s = config::Config::builder()
            .add_source(
                config::Environment::with_prefix("PORTUNEX")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()?;
        Ok(s.try_deserialize()?)
    }
}
