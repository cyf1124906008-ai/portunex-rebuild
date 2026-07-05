//! 用户面路由(需 Bearer)。
use axum::{routing::{get, post}, Router};
use crate::handlers::{user, api_key};
use crate::state::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/me", get(user::me))
        .route("/points/stats", get(user::points_stats))
        .route("/usage", get(user::usage_list))
        .route("/api-keys", get(api_key::list).post(api_key::create))
        .route("/api-keys/{id}/deactivate", post(api_key::deactivate))
}
