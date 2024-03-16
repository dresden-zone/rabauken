use std::sync::Arc;

use crate::service::UserService;
use crate::session::SessionStore;

#[derive(Clone)]
pub(crate) struct Context {
  pub(crate) session_store: Arc<SessionStore>,
  pub(crate) user_service: Arc<UserService>,
}
