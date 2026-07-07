//! Portunex 网关主程序 —— 重建版 Phase 1。
//! 已实现:配置加载、Postgres 连接、sqlx 迁移、首启播种 admin、/health /ready、路由骨架。
//! 待填(见 RECONSTRUCTION_BLUEPRINT.md):各 handler/route 真实逻辑、upstream、translators、
//! 调度算法、OIDC 签发、计费、WebSocket 上游池等。

mod config;
mod handlers;
mod middleware;
mod routes;
mod services;
mod server;
mod state;
mod util;

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::config::Settings;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_env("RUST_LOG").unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let settings = Settings::load().context("加载配置失败(检查 PORTUNEX__* 环境变量)")?;
    tracing::info!("Starting Portunex Server on {}:{}", settings.server.host, settings.server.port);
    tracing::info!("Instance ID: {}", settings.server.instance_id);

    // ── 数据库 ──
    let pool = PgPoolOptions::new()
        .max_connections(16)
        .connect(&settings.db.url)
        .await
        .context("连接 Postgres 失败")?;

    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .context("数据库迁移失败")?;

    // ── 首启播种管理员(还原原行为:随机密码,打印到日志)──
    seed_admin(&pool).await?;

    let state = AppState::new(settings.clone(), pool);

    // ── 起服务 ──
    let app = server::build_router(state.clone());
    let addr = format!("{}:{}", settings.server.host, settings.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await.context("绑定端口失败")?;
    tracing::info!("Starting server on {}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

/// 首次启动(users 表为空)时创建 admin@portunex.local,随机密码打印到日志。
/// 与原实例行为一致(argon2id 存储)。
async fn seed_admin(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .unwrap_or(0);
    if count > 0 {
        return Ok(());
    }
    tracing::info!("Initializing admin account...");

    use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
    use argon2::Argon2;
    use rand::Rng;

    // 16 位随机密码(与观测到的形如 "pMLesN0Kxz95IClY" 一致)
    const ALPHA: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    let password: String = (0..16).map(|_| ALPHA[rng.gen_range(0..ALPHA.len())] as char).collect();

    let salt = SaltString::generate(&mut OsRng);
    let phc = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("argon2 hash 失败: {e}"))?
        .to_string();

    sqlx::query(
        "INSERT INTO users (id, email, password_phc, role, points) \
         VALUES ($1, $2, $3, 'admin', 0)",
    )
    .bind(next_snowflake_id())
    .bind("admin@portunex.local")
    .bind(&phc)
    .execute(pool)
    .await?;

    tracing::info!("==================================================");
    tracing::info!("系统首次启动，已自动创建管理员账号：");
    tracing::info!("  邮箱: admin@portunex.local");
    tracing::info!("  密码: {}", password);
    tracing::info!("请妥善保管以上信息，建议首次登录后立即修改密码。");
    tracing::info!("==================================================");
    Ok(())
}

/// 占位雪花 ID。原系统用 PORTUNEX__IDS__WORKER_ID 做分片,此处简化。
/// TODO: 还原真实雪花算法(worker_id + 时间戳 + 序列)。
fn next_snowflake_id() -> i64 {
    use rand::Rng;
    rand::thread_rng().gen_range(1_000_000_000_000_000..9_000_000_000_000_000)
}
