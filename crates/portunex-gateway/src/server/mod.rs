//! HTTP 服务装配:CORS、探活、以及按原实例观测到的全部端点接线。
//! 探活响应与原实例逐字节一致;业务端点目前返回 501(待按 BLUEPRINT 实现)。

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde_json::json;
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};

use crate::state::AppState;

static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

pub fn build_router(state: AppState) -> Router {
    START.get_or_init(Instant::now);

    // 原实例对所有响应带 Access-Control-Allow-*: *(供跨域前端调用)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // ── 探活(响应与原实例一致)──
        .route("/health", get(health))
        .route("/ready", get(ready))
        // ── OpenAI 兼容 ──
        .route("/v1/models", get(not_impl))
        .route("/v1/chat/completions", post(not_impl))
        .route("/v1/responses", post(not_impl))
        .route("/responses", post(not_impl))
        // ── Anthropic 兼容 ──
        .route("/messages", post(not_impl))
        .route("/messages/count_tokens", post(not_impl))
        // ── Gemini ──
        .route("/v1beta/{*rest}", post(not_impl))
        // ── 鉴权 / 用户面 ──(auth/register、auth/login 已实现)
        .merge(crate::routes::auth::routes())
        .route("/api-keys", get(not_impl).post(not_impl))
        .route("/subscriptions", get(not_impl))
        .route("/usage", get(not_impl))
        .route("/points/stats", get(not_impl))
        // ── 上游账号接入(OAuth)──
        .route("/oauth/claude/start", post(not_impl))
        .route("/oauth/claude/refresh", post(not_impl))
        .route("/oauth/openai/start", post(not_impl))
        .route("/oauth/openai/import", post(not_impl))
        .route("/oauth/gemini/start", post(not_impl))
        .route("/oauth/antigravity/start", post(not_impl))
        // ── OIDC 提供方 ──
        .route("/.well-known/openid-configuration", get(not_impl))
        .route("/oidc/jwks", get(not_impl))
        .route("/oidc/authorize", get(not_impl))
        .route("/oidc/token", post(not_impl))
        .route("/oidc/userinfo", get(not_impl))
        .route("/oidc/revoke", post(not_impl))
        // ── 管理后台(还原自 routes/admin/*)──
        .route("/admin/users", get(not_impl))
        .route("/admin/api-keys", get(not_impl))
        .route("/admin/providers", get(not_impl))
        .route("/admin/model-aliases", get(not_impl))
        .route("/admin/orders", get(not_impl))
        .route("/admin/payment-channels", get(not_impl))
        .route("/admin/subscription-plans", get(not_impl))
        .route("/admin/redemption-codes", get(not_impl))
        .route("/admin/request-logs", get(not_impl))
        .route("/admin/stats", get(not_impl))
        .route("/admin/oidc-clients", get(not_impl))
        .layer(cors)
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    Json(json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "instance_id": std::env::var("PORTUNEX__SERVER__INSTANCE_ID").unwrap_or_else(|_| "blue".into()),
        "uptime_secs": uptime(),
    }))
}

async fn ready(axum::extract::State(state): axum::extract::State<AppState>) -> impl IntoResponse {
    // 探测数据库
    let db_ok = sqlx::query("SELECT 1").execute(state.pool()).await.is_ok();
    let cache = if state.settings().redis.enabled { "connected" } else { "disabled" };
    let body = json!({
        "status": if db_ok { "ready" } else { "degraded" },
        "version": env!("CARGO_PKG_VERSION"),
        "instance_id": state.settings().server.instance_id,
        "uptime_secs": uptime(),
        "database": if db_ok { "connected" } else { "error" },
        "cache": cache,
    });
    let code = if db_ok { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (code, Json(body))
}

async fn not_impl() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({ "error": "not_implemented", "detail": "重建骨架:该端点待按 BLUEPRINT 实现" })),
    )
}

fn uptime() -> u64 {
    START.get().map(|s| s.elapsed().as_secs()).unwrap_or(0)
}
