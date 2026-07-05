use axum::{routing::{get, post}, Router};
use crate::handlers::subscription;
use crate::state::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/subscriptions", get(subscription::list).post(subscription::subscribe))
        .route("/admin/subscription-plans", post(subscription::admin_create_plan))
}
