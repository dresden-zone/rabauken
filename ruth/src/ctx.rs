use std::sync::Arc;

use session::{SessionContext, SessionStore};

use crate::service::UserService;

#[derive(Clone)]
pub(crate) struct Context {
  pub(crate) user_service: Arc<UserService>,
  pub(crate) session_store: SessionStore,
}

impl SessionContext for Context {
  fn session_store(&self) -> &SessionStore {
    &self.session_store
  }
}
