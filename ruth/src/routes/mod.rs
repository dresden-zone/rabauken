use axum::routing::{get, post};
use axum::Router;

use crate::ctx::Context;

mod auth;

pub(super) fn router() -> Router<Context> {
  Router::new()
    .route("/api/auth/me", get(auth::me))
    .route("/api/auth/register", post(auth::register))
    .route("/api/auth/password", post(auth::password_login))
}
