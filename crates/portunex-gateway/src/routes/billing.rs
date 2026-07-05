use axum::{routing::post, Router};
use crate::handlers::billing;
use crate::state::AppState;
pub fn routes() -> Router<AppState> {
    Router::new().route("/redeem", post(billing::redeem))
}
