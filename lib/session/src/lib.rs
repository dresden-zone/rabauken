use std::str::FromStr;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use bb8_redis::bb8::Pool;
use bb8_redis::redis::AsyncCommands;
use bb8_redis::RedisConnectionManager;
use tracing::error;
use uuid::Uuid;

pub trait SessionContext {
  fn session_store(&self) -> &SessionStore;
}

#[derive(Clone)]
pub struct SessionStore {
  redis: Pool<RedisConnectionManager>,
}

pub struct Session {
  pub user_id: Uuid,
}

impl SessionStore {
  pub fn new(redis: Pool<RedisConnectionManager>) -> Self {
    Self { redis }
  }

  pub async fn push(&self, user_id: Uuid) -> anyhow::Result<Uuid> {
    let mut conn = self.redis.get().await?;
    let session_id = Uuid::new_v4();
    conn.set(session_id, user_id).await?;

    Ok(session_id)
  }

  pub async fn lookup(&self, session_id: Uuid) -> anyhow::Result<Option<Uuid>> {
    let mut conn = self.redis.get().await?;
    let user_id = conn.get::<_, Option<Uuid>>(session_id).await?;
    Ok(user_id)
  }
}

#[axum::async_trait]
impl<C: SessionContext + Sync> FromRequestParts<C> for Session {
  type Rejection = StatusCode;

  async fn from_request_parts(parts: &mut Parts, ctx: &C) -> Result<Self, Self::Rejection> {
    let jar = CookieJar::from_headers(&parts.headers);
    let cookie = jar.get("session_id").ok_or(StatusCode::UNAUTHORIZED)?;
    let session_id = Uuid::from_str(cookie.value()).map_err(|err| {
      error!("cannot deserialize session cookie {}", err);
      StatusCode::UNAUTHORIZED
    })?;

    let user_id = ctx
      .session_store()
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
