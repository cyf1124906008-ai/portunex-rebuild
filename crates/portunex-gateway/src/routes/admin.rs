//! /admin/* 路由(均需 AdminUser)。
use axum::{routing::{get, post, delete}, Router};
use crate::handlers::admin;
use crate::state::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/users", get(admin::list_users))
        .route("/admin/redemption-codes", post(admin::create_redemption_code))
        .route("/admin/providers", get(admin::list_providers).post(admin::create_provider))
        .route("/admin/providers/{id}", delete(admin::delete_provider))
        .route("/admin/model-aliases", get(admin::list_model_aliases).post(admin::create_model_alias))
        .route("/admin/model-aliases/{id}", delete(admin::delete_model_alias))
}
