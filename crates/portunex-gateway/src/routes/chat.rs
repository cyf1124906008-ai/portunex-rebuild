use axum::{routing::post, Router};
use crate::handlers::chat;
use crate::state::AppState;
pub fn routes() -> Router<AppState> {
    Router::new().route("/v1/chat/completions", post(chat::chat_completions))
}
