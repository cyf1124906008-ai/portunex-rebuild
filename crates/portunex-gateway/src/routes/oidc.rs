use axum::{routing::{get, post}, Router};
use crate::handlers::oidc;
use crate::state::AppState;
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/.well-known/openid-configuration", get(oidc::discovery))
        .route("/oidc/jwks", get(oidc::jwks))
        .route("/oidc/token", post(oidc::token))
        .route("/oidc/userinfo", get(oidc::userinfo))
}
