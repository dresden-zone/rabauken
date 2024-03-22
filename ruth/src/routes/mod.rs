use axum::routing::{get, post};
use axum::Router;

use crate::ctx::Context;

mod auth;

pub(super) fn router() -> Router<Context> {
  Router::new()
    .route("/api/auth/me", get(auth::me))
    .route("/api/auth/invite/:invite_id", get(auth::check_invite))
    .route("/api/auth/register/:invite_id", post(auth::register))
    .route("/api/auth/password", post(auth::password_login))
}
