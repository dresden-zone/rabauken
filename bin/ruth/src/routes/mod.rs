use axum::routing::{get, post};
use axum::Router;

use crate::ctx::Context;

mod auth;

pub(super) fn router() -> Router<Context> {
  Router::new()
    .route("/api/auth/v1/me", get(auth::me))
    .route("/api/auth/v1/invite/:invite_id", get(auth::check_invite))
    .route("/api/auth/v1/register/:invite_id", post(auth::register))
    .route("/api/auth/v1/password", post(auth::password_login))
}
