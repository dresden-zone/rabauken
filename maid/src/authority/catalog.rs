use std::sync::Arc;
use trust_dns_server::authority::{
  AuthorityObject, LookupError, LookupObject, LookupOptions, MessageRequest, UpdateResult, ZoneType,
};
use trust_dns_server::proto::rr::{LowerName, RecordType};
use trust_dns_server::server::RequestInfo;

use crate::service::ZoneService;

pub(crate) struct CatalogAuthority {
  zone_service: Arc<ZoneService>,
}

impl CatalogAuthority {
  pub(crate) fn new(zone_service: Arc<ZoneService>) -> Self {
    Self { zone_service }
  }
}

#[async_trait::async_trait]
impl AuthorityObject for CatalogAuthority {
  fn box_clone(&self) -> Box<dyn AuthorityObject> {
    todo!()
  }

  fn zone_type(&self) -> ZoneType {
    todo!()
  }

  fn is_axfr_allowed(&self) -> bool {
    todo!()
  }

  async fn update(&self, update: &MessageRequest) -> UpdateResult<bool> {
    todo!()
  }

  fn origin(&self) -> &LowerName {
    todo!()
  }

  async fn lookup(
    &self,
    name: &LowerName,
    rtype: RecordType,
    lookup_options: LookupOptions,
  ) -> Result<Box<dyn LookupObject>, LookupError> {
    todo!()
  }

  async fn search(
    &self,
    request_info: RequestInfo<'_>,
    lookup_options: LookupOptions,
  ) -> Result<Box<dyn LookupObject>, LookupError> {
    todo!()
  }

  async fn get_nsec_records(
    &self,
    name: &LowerName,
    lookup_options: LookupOptions,
  ) -> Result<Box<dyn LookupObject>, LookupError> {
    todo!()
  }
}
