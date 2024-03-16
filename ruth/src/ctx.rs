use std::sync::Arc;

use crate::service::UserService;

#[derive(Clone)]
pub(crate) struct Context {
  pub(crate) user_service: Arc<UserService>,
}
