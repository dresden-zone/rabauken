use std::str::FromStr;
use std::sync::Arc;

use sea_orm::prelude::Uuid;
use trust_dns_server::authority::{
  AuthLookup, Authority, AuthorityObject, LookupError, LookupObject, LookupOptions, LookupRecords,
  MessageRequest, UpdateResult, ZoneType,
};
use trust_dns_server::proto::rr::{LowerName, RecordSet, RecordType};
use trust_dns_server::server::RequestInfo;

use crate::service::ZoneService;

pub(crate) struct CatalogAuthority {
  zone_service: Arc<ZoneService>,
  zone_id: Uuid,
  origin: LowerName,
}

impl CatalogAuthority {
  pub(crate) fn new(zone_service: Arc<ZoneService>, zone_id: Uuid, origin: LowerName) -> Self {
    Self {
      zone_service,
      zone_id,
      origin,
    }
  }
}

#[async_trait::async_trait]
impl Authority for CatalogAuthority {
  type Lookup = AuthLookup;

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
  ) -> Result<Self::Lookup, LookupError> {
    todo!()
  }

  async fn search(
    &self,
    request: RequestInfo<'_>,
    lookup_options: LookupOptions,
  ) -> Result<Self::Lookup, LookupError> {
    let x = request.query.name().to_string();
    let y = self.origin.to_string();
    let option = x.strip_suffix(&y).unwrap();
    let option = option.strip_suffix(".").unwrap();

    let y = b(&self.origin, request.query.name());

    let records = self
      .zone_service
      .lookup(
        self.zone_id,
        &LowerName::from_str(option).unwrap(),
        request.query.original().query_type(),
      )
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
    name: &LowerName,
    lookup_options: LookupOptions,
  ) -> Result<Self::Lookup, LookupError> {
    todo!()
  }
}

fn b(zone: &LowerName, name: &LowerName) -> String {
  let zone = zone.to_string();
  let name = name.to_string();

  name
    .strip_suffix(&zone)
    .unwrap()
    .strip_suffix(".")
    .unwrap()
    .to_string()
}
