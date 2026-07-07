use axum::{routing::post, Router};
use crate::handlers::messages;
use crate::state::AppState;
pub fn routes() -> Router<AppState> {
    Router::new().route("/messages", post(messages::messages))
}
