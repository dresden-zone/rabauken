use std::ops::DerefMut;
use std::str::FromStr;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use bb8_redis::bb8::Pool;
use bb8_redis::redis::AsyncCommands;
use bb8_redis::RedisConnectionManager;
use redis_derive::{FromRedisValue, ToRedisArgs};
use tracing::error;
use uuid::Uuid;

pub const ROLE_NONE: i16 = 0b0000000000000000;
pub const ROLE_ADMIN: i16 = 0b0000000000000001;
pub const ROLE_DNS: i16 = 0b0000000000000010;

pub trait SessionContext {
  fn session_store(&self) -> &SessionStore;
}

#[derive(Clone)]
pub struct SessionStore {
  redis: Pool<RedisConnectionManager>,
}

pub struct Session<const ROLES: i16 = ROLE_NONE> {
  pub user_id: Uuid,
  pub roles: i16,
}

impl SessionStore {
  pub fn new(redis: Pool<RedisConnectionManager>) -> Self {
    Self { redis }
  }

  pub async fn push(&self, user_id: Uuid, roles: i16) -> anyhow::Result<Uuid> {
    let mut conn = self.redis.get().await?;
    let session_id = Uuid::new_v4();
    redis::cmd("HSET")
      .arg(session_id)
      .arg(Inner { user_id, roles })
      .query_async(conn.deref_mut())
      .await?;

    Ok(session_id)
  }

  pub async fn lookup(&self, session_id: Uuid) -> anyhow::Result<Option<(Uuid, i16)>> {
    let mut conn = self.redis.get().await?;
    let inner = conn.hgetall::<_, Option<Inner>>(session_id).await?;
    Ok(inner.map(|session| (session.user_id, session.roles)))
  }
}

#[axum::async_trait]
impl<C: SessionContext + Sync, const ROLES: i16> FromRequestParts<C> for Session<ROLES> {
  type Rejection = StatusCode;

  async fn from_request_parts(parts: &mut Parts, ctx: &C) -> Result<Self, Self::Rejection> {
    let jar = CookieJar::from_headers(&parts.headers);
    let cookie = jar.get("session_id").ok_or(StatusCode::UNAUTHORIZED)?;
    let session_id = Uuid::from_str(cookie.value()).map_err(|err| {
      error!("cannot deserialize session cookie {}", err);
      StatusCode::UNAUTHORIZED
    })?;

    let session = ctx
      .session_store()
      .lookup(session_id)
      .await
      .map_err(|err| {
        error!("cannot lookup session id {}", err);
        StatusCode::UNAUTHORIZED
      })?;

    let (user_id, user_roles) = session.ok_or(StatusCode::UNAUTHORIZED)?;

    if user_roles & ROLES != ROLES {
      return Err(StatusCode::FORBIDDEN);
    }

    Ok(Self {
      user_id,
      roles: user_roles,
    })
  }
}

#[derive(ToRedisArgs, FromRedisValue)]
struct Inner {
  user_id: Uuid,
  roles: i16,
}
