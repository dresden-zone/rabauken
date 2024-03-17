use std::str::FromStr;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;
use tracing::error;
use uuid::Uuid;

#[async_trait::async_trait]
pub trait SessionContext {
  async fn session_store(&self) -> SessionStore;
}

pub struct SessionStore {
  client: redis::Client,
}

pub struct Session {
  pub user_id: Uuid,
}

impl SessionStore {
  async fn push(user_id: Uuid) -> Session {}

  async fn lookup(session_id: Uuid) -> Option<Session> {}
}

#[async_trait::async_trait]
impl<C: SessionContext + Sync> FromRequestParts<C> for Session {
  type Rejection = StatusCode;

  async fn from_request_parts(parts: &mut Parts, ctx: &C) -> Result<Self, Self::Rejection> {
    let jar = CookieJar::from_headers(&parts.headers);
    let cookie = jar.get("session_id").ok_or(StatusCode::UNAUTHORIZED)?;
    let session_id = Uuid::from_str(cookie.value()).map_err(|err| {
      error!("cannot deserialize session cookie {}", err);
      StatusCode::UNAUTHORIZED
    })?;

    let user_id: Option<Uuid> =
      ctx
        .session_store()
        .await
        .lookup(session_id)
        .await
        .map_err(|err| {
          error!("cannot lookup session id {}", err);
          StatusCode::UNAUTHORIZED
        })?;

    let user_id = user_id.ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(Session { user_id })
  }
}
