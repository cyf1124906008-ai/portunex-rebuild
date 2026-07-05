//! /auth 路由。
use axum::{routing::post, Router};
use crate::handlers::auth;
use crate::state::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
}
