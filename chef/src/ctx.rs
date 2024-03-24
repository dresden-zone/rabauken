use std::sync::Arc;

use session::{SessionContext, SessionStore};

use crate::service::{RecordService, ZoneService};

#[derive(Clone)]
pub(crate) struct Context {
  pub(crate) zone_service: Arc<ZoneService>,
  pub(crate) record_service: Arc<RecordService>,
  pub(crate) session_store: SessionStore,
}

impl SessionContext for Context {
  fn session_store(&self) -> &SessionStore {
    &self.session_store
  }
}
