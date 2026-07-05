//! HTTP 服务装配。已实现的合并真实路由;未实现的保留 501 桩。
use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use serde_json::json;
use std::time::Instant;
use tower_http::cors::{Any, CorsLayer};
use crate::state::AppState;

static START: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

pub fn build_router(state: AppState) -> Router {
    START.get_or_init(Instant::now);
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);

    Router::new()
        // 探活
        .route("/health", axum::routing::get(health))
        .route("/ready", axum::routing::get(ready))
        // 已实现的真实路由
        .merge(crate::routes::auth::routes())     // /auth/register /auth/login
        .merge(crate::routes::user::routes())     // /me /api-keys*
        .merge(crate::routes::billing::routes())  // /redeem
        .merge(crate::routes::admin::routes())    // /admin/*
        .merge(crate::routes::models::routes())   // /v1/models
        .merge(crate::routes::subscription::routes()) // /subscriptions /admin/subscription-plans
        // 未实现(需上游凭据或后续实现)—— 501 桩
        .route("/v1/chat/completions", post(not_impl))
        .route("/v1/responses", post(not_impl))
        .route("/responses", post(not_impl))
        .route("/messages", post(not_impl))
        .route("/messages/count_tokens", post(not_impl))
        .route("/v1beta/{*rest}", post(not_impl))
        .route("/oauth/claude/start", post(not_impl))
        .route("/oauth/openai/start", post(not_impl))
        .route("/oauth/gemini/start", post(not_impl))
        .route("/oauth/antigravity/start", post(not_impl))
        .route("/.well-known/openid-configuration", axum::routing::get(not_impl))
        .route("/oidc/token", post(not_impl))
        .layer(cors)
        .with_state(state)
}

async fn health() -> impl IntoResponse {
    Json(json!({
        "status": "healthy", "version": env!("CARGO_PKG_VERSION"),
        "instance_id": std::env::var("PORTUNEX__SERVER__INSTANCE_ID").unwrap_or_else(|_| "blue".into()),
        "uptime_secs": uptime(),
    }))
}
async fn ready(axum::extract::State(state): axum::extract::State<AppState>) -> impl IntoResponse {
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
    (StatusCode::NOT_IMPLEMENTED, Json(json!({"error":"not_implemented"})))
}
fn uptime() -> u64 { START.get().map(|s| s.elapsed().as_secs()).unwrap_or(0) }
