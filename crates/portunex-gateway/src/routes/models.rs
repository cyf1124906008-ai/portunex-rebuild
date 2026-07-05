use axum::{routing::get, Router};
use crate::handlers::models;
use crate::state::AppState;
pub fn routes() -> Router<AppState> {
    Router::new().route("/v1/models", get(models::list))
}
