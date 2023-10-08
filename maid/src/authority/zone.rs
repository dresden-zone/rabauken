use std::sync::Arc;

use trust_dns_server::authority::{
  AuthLookup, Authority, LookupError, LookupOptions, LookupRecords, MessageRequest, UpdateResult,
  ZoneType,
};
use trust_dns_server::proto::rr::{LowerName, RecordSet, RecordType};
use trust_dns_server::server::RequestInfo;

use crate::service::ZoneService;

pub(crate) struct ZoneAuthority {
  zone_service: Arc<ZoneService>,
}

impl ZoneAuthority {
  pub(crate) fn new(zone_service: Arc<ZoneService>) -> Self {
    Self { zone_service }
  }
}

#[async_trait::async_trait]
impl Authority for ZoneAuthority {
  type Lookup = AuthLookup;

  fn zone_type(&self) -> ZoneType {
    ZoneType::Primary
  }

  fn is_axfr_allowed(&self) -> bool {
    todo!()
  }

  async fn update(&self, _update: &MessageRequest) -> UpdateResult<bool> {
    todo!()
  }

  fn origin(&self) -> &LowerName {
    todo!()
  }

  async fn lookup(
    &self,
    _name: &LowerName,
    _rtype: RecordType,
    _lookup_options: LookupOptions,
  ) -> Result<Self::Lookup, LookupError> {
    todo!()
  }

  async fn search(
    &self,
    request: RequestInfo<'_>,
    lookup_options: LookupOptions,
  ) -> Result<Self::Lookup, LookupError> {
    let records = self
      .zone_service
      .lookup(request.query.name(), request.query.original().query_type())
      .await
      .unwrap();

    let mut set = RecordSet::new(
      request.query.original().name(),
      request.query.original().query_type(),
      123,
    );

    for x in records {
      set.new_record(x.data().unwrap());
    }

    Ok(AuthLookup::Records {
      answers: LookupRecords::Records {
        lookup_options,
        records: Arc::new(set),
      },
      additionals: None,
    })
  }

  async fn get_nsec_records(
    &self,
    _name: &LowerName,
    _lookup_options: LookupOptions,
  ) -> Result<Self::Lookup, LookupError> {
    todo!()
  }
}
