use axum::async_trait;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use tokio::sync::RwLock;
use tracing::error;
use uuid::Uuid;

use crate::ctx::Context;

#[derive(Clone)]
pub(crate) struct Session(Arc<RwLock<Inner>>);

struct Inner {
  user_id: Uuid,
}

#[derive(Default)]
pub(crate) struct SessionStore {
  sessions: RwLock<HashMap<Uuid, Session>>,
}

impl SessionStore {
  pub(crate) async fn push(&self, user_id: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    let mut sessions = self.sessions.write().await;
    sessions.insert(id, Session(Arc::new(RwLock::new(Inner { user_id }))));
    id
  }

  pub(crate) async fn load(&self, id: &Uuid) -> Option<Session> {
    self.sessions.read().await.get(id).cloned()
  }
}

impl Session {
  pub(crate) async fn user_id(&self) -> Uuid {
    self.0.read().await.user_id
  }
}

#[async_trait]
impl FromRequestParts<Context> for Session {
  type Rejection = StatusCode;

  async fn from_request_parts(parts: &mut Parts, state: &Context) -> Result<Self, Self::Rejection> {
    let jar = CookieJar::from_headers(&parts.headers);
    let cookie = jar.get("session_id").ok_or(StatusCode::UNAUTHORIZED)?;
    let session_id = Uuid::from_str(cookie.value()).map_err(|err| {
      error!("cannot deserialize session cookie {}", err);
      StatusCode::UNAUTHORIZED
    })?;

    let data = state
      .session_store
      .load(&session_id)
      .await
      .ok_or(StatusCode::UNAUTHORIZED)?
      .clone();

    Ok(data)
  }
}
