use std::fmt::Write;
use std::sync::Arc;

use async_trait::async_trait;
use hickory_server::authority::{
  AnyRecords, AuthLookup, Authority, LookupError, LookupOptions, LookupRecords, LookupResult,
  MessageRequest, UpdateResult, ZoneType,
};
use hickory_server::proto::op::ResponseCode;
use hickory_server::proto::rr::domain::Label;
use hickory_server::proto::rr::{LowerName, Name, RData, Record, RecordSet, RecordType};
use hickory_server::server::RequestInfo;
use sea_orm::prelude::Uuid;
use tokio::try_join;

use crate::service::ZoneService;

pub(crate) struct ZoneAuthority {
  zone_service: Arc<ZoneService>,
  zone_id: Uuid,
  origin: LowerName,
  labels: usize,
}

impl ZoneAuthority {
  pub(crate) fn new(zone_service: Arc<ZoneService>, zone_id: Uuid, origin: LowerName) -> Self {
    Self {
      zone_service,
      zone_id,
      labels: Name::from(origin.clone()).iter().len(),
      origin,
    }
  }
}

#[async_trait]
impl Authority for ZoneAuthority {
  type Lookup = AuthLookup;

  fn zone_type(&self) -> ZoneType {
    ZoneType::Primary
  }

  fn is_axfr_allowed(&self) -> bool {
    todo!()
  }

  async fn update(&self, _update: &MessageRequest) -> UpdateResult<bool> {
    Err(ResponseCode::NotImp)
  }

  fn origin(&self) -> &LowerName {
    &self.origin
  }

  async fn lookup(
    &self,
    name: &LowerName,
    query_type: RecordType,
    lookup_options: LookupOptions,
  ) -> Result<Self::Lookup, LookupError> {
    let host = {
      let mut host = String::new();

      let name = Name::from(name);
      let name = name.into_iter().rev().skip(self.labels);
      let mut first = true;
      for label in name {
        if first {
          first = false;
        } else {
          host.write_char('.').unwrap();
        }
        let name = Label::from_raw_bytes(label).unwrap();
        name.write_ascii(&mut host).unwrap();
      }

      if host.is_empty() {
        host.write_char('@').unwrap();
      }

      host
    };

    // Collect the records from each rr_set
    let (result, additionals): (LookupResult<LookupRecords>, Option<LookupRecords>) =
      match query_type {
        RecordType::AXFR | RecordType::ANY => {
          let result = AnyRecords::new(
            lookup_options,
            self
              .zone_service
              .lookup_any(self.zone_id)
              .await
              .unwrap()
              .into_iter()
              .map(Arc::new)
              .collect(),
            query_type,
            name.clone(),
          );
          (Ok(LookupRecords::AnyRecords(result)), None)
        }
        _ => {
          // perform the lookup
          let answer = self
            .zone_service
            .lookup(
              self.zone_id,
              &Name::from(&self.origin),
              name,
              &host,
              query_type,
            )
            .await
            .unwrap();

          let additional = match answer.as_ref().and_then(|a| maybe_next_name(a, query_type)) {
            Some((search_name, search_type)) => {
              if !self.origin.zone_of(&search_name) {
                None
              } else {
                self
                  .zone_service
                  .additional_search(
                    self.zone_id,
                    &Name::from(&self.origin),
                    name,
                    query_type,
                    search_name,
                    search_type,
                    lookup_options,
                  )
                  .await
              }
            }
            None => None,
          };

          let answer = answer.map_or(Err(LookupError::from(ResponseCode::NXDomain)), |rr_set| {
            Ok(LookupRecords::new(lookup_options, Arc::new(rr_set)))
          });

          let additionals = additional.map(|a| LookupRecords::many(lookup_options, a));

          (answer, additionals)
        }
      };

    let result = match result {
      Err(LookupError::ResponseCode(ResponseCode::NXDomain)) => {
        // TODO: evaluate this query, should this be done?
        return if self
          .zone_service
          .name_exists(self.zone_id, &host)
          .await
          .unwrap()
        {
          Err(LookupError::NameExists)
        } else {
          let code = if self.origin().zone_of(name) {
            ResponseCode::NXDomain
          } else {
            ResponseCode::Refused
          };
          Err(LookupError::from(code))
        };
      }
      Err(e) => return Err(e),
      o => o,
    };

    result.map(|answers| {
      if let Some(x) = answers.iter().next() {
        if x.record_type() == RecordType::SOA {
          return AuthLookup::SOA(answers);
        }
      }

      AuthLookup::answers(answers, additionals)
    })
  }

  async fn search(
    &self,
    request_info: RequestInfo<'_>,
    lookup_options: LookupOptions,
  ) -> Result<Self::Lookup, LookupError> {
    let lookup_name = request_info.query.name();
    let record_type: RecordType = request_info.query.query_type();

    // perform the actual lookup
    match record_type {
      RecordType::AXFR => {
        let (start_soa, end_soa, records) = try_join!(
          self.soa_secure(lookup_options),
          self.soa(),
          self.lookup(lookup_name, record_type, lookup_options),
        )?;
        Ok(match start_soa {
          l @ AuthLookup::Empty => l,
          start_soa => AuthLookup::AXFR {
            start_soa: match start_soa {
              AuthLookup::SOA(soa) => soa,
              _ => panic!("abc"),
            },
            records: records.unwrap_records(),
            end_soa: match end_soa {
              AuthLookup::SOA(soa) => soa,
              _ => panic!("abc"),
            },
          },
        })
      }
      // A standard Lookup path
      _ => self.lookup(lookup_name, record_type, lookup_options).await,
    }
  }

  async fn get_nsec_records(
    &self,
    _name: &LowerName,
    _lookup_options: LookupOptions,
  ) -> Result<Self::Lookup, LookupError> {
    todo!()
  }
}

/// Gets the next search name, and returns the RecordType that it originated from
fn maybe_next_name(
  record_set: &RecordSet,
  query_type: RecordType,
) -> Option<(LowerName, RecordType)> {
  match (record_set.record_type(), query_type) {
    (t @ RecordType::NS, RecordType::NS) => record_set
      .records_without_rrsigs()
      .next()
      .and_then(Record::data)
      .and_then(RData::as_ns)
      .map(|ns| LowerName::from(&ns.0))
      .map(|name| (name, t)),
    // CNAME will continue to additional processing for any query type
    (t @ RecordType::CNAME, _) => record_set
      .records_without_rrsigs()
      .next()
      .and_then(Record::data)
      .and_then(RData::as_cname)
      .map(|cname| LowerName::from(&cname.0))
      .map(|name| (name, t)),
    (t @ RecordType::MX, RecordType::MX) => record_set
      .records_without_rrsigs()
      .next()
      .and_then(Record::data)
      .and_then(RData::as_mx)
      .map(|mx| mx.exchange().clone())
      .map(LowerName::from)
      .map(|name| (name, t)),
    (t @ RecordType::SRV, RecordType::SRV) => record_set
      .records_without_rrsigs()
      .next()
      .and_then(Record::data)
      .and_then(RData::as_srv)
      .map(|srv| srv.target().clone())
      .map(LowerName::from)
      .map(|name| (name, t)),
    // other additional collectors can be added here can be added here
    _ => None,
  }
}
