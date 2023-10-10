use std::collections::HashSet;
use std::fmt::Write;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use sea_orm::prelude::{Expr, Uuid};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use trust_dns_server::authority::LookupOptions;
use trust_dns_server::proto::rr::domain::Label;
use trust_dns_server::proto::rr::RecordType::SOA;
use trust_dns_server::proto::rr::{rdata, LowerName, Name, RData, Record, RecordSet, RecordType};

use entity::{record, record_a, record_aaaa, record_cname, record_mx, record_ns, record_txt, zone};

pub(crate) struct ZoneService {
  db: Arc<DatabaseConnection>,
}

impl ZoneService {
  pub(crate) fn new(db: Arc<DatabaseConnection>) -> Self {
    Self { db }
  }

  pub(crate) async fn soa(&self, zone_id: Uuid, original: &Name) -> anyhow::Result<Option<Record>> {
    let zone = zone::Entity::find_by_id(zone_id)
      .one(self.db.as_ref())
      .await?;

    let zone = match zone {
      Some(zone) => zone,
      None => return Err(anyhow!("zone by id not found")),
    };

    let mut name = Name::from_ascii(zone.name)?;
    name.set_fqdn(true);

    if &name != original {
      Ok(None)
    } else {
      Ok(Some(Record::from_rdata(
        name,
        31,
        RData::SOA(rdata::SOA::new(
          Name::from_ascii("ns.dns.dresden.zone.")?,
          Name::from_ascii("dns.dresden.zone")?,
          1,
          zone.refresh,
          zone.retry,
          zone.expire,
          zone.minimum as u32,
        )),
      )))
    }
  }

  pub(crate) async fn lookup_any(&self, zone_id: Uuid) -> Vec<RecordSet> {
    todo!()
  }

  pub(crate) async fn lookup(
    &self,
    zone_id: Uuid,
    origin: &Name,
    original: &LowerName,
    host: &str,
    r#type: RecordType,
  ) -> anyhow::Result<Option<RecordSet>> {
    let name = original.into();
    let mut set = RecordSet::new(&name, r#type, 0);

    if r#type == SOA {
      return match self.soa(zone_id, &name).await? {
        None => Ok(None),
        Some(record) => {
          set.insert(record, 1);
          Ok(Some(set))
        }
      };
    }

    let query = record::Entity::find().inner_join(zone::Entity).filter(
      Expr::col((zone::Entity, zone::Column::Id))
        .eq(Expr::val(zone_id))
        .and(Expr::col((record::Entity, record::Column::Name)).eq(host)),
    );

    let mut set = RecordSet::new(&name, r#type, 0);

    {
      let query = query.clone();
      match r#type {
        RecordType::A => query
          .inner_join(record_a::Entity)
          .select_also(record_a::Entity)
          .all(self.db.as_ref())
          .await?
          .into_iter()
          .for_each(|(record, a)| {
            set.insert(
              Record::from_rdata(
                name.clone(),
                record.ttl as u32,
                RData::A(rdata::A(Ipv4Addr::from_str(&a.unwrap().address).unwrap())),
              ),
              2,
            );
          }),
        RecordType::AAAA => query
          .inner_join(record_aaaa::Entity)
          .select_also(record_aaaa::Entity)
          .all(self.db.as_ref())
          .await?
          .into_iter()
          .for_each(|(record, aaaa)| {
            set.insert(
              Record::from_rdata(
                name.clone(),
                record.ttl as u32,
                RData::AAAA(rdata::AAAA(
                  Ipv6Addr::from_str(&aaaa.unwrap().address).unwrap(),
                )),
              ),
              2,
            );
          }),
        RecordType::MX => query
          .inner_join(record_mx::Entity)
          .select_also(record_mx::Entity)
          .all(self.db.as_ref())
          .await?
          .into_iter()
          .for_each(|(record, mx)| {
            let a = mx.unwrap();
            set.insert(
              Record::from_rdata(
                name.clone(),
                record.ttl as u32,
                RData::MX(rdata::MX::new(
                  a.preference as u16,
                  Name::from_ascii(a.exchange).unwrap(),
                )),
              ),
              2,
            );
          }),
        RecordType::NS => query
          .inner_join(record_ns::Entity)
          .select_also(record_ns::Entity)
          .all(self.db.as_ref())
          .await?
          .into_iter()
          .for_each(|(record, ns)| {
            set.insert(
              Record::from_rdata(
                name.clone(),
                record.ttl as u32,
                RData::NS(rdata::NS(Name::from_ascii(ns.unwrap().target).unwrap())),
              ),
              2,
            );
          }),
        RecordType::CNAME => query
          .inner_join(record_cname::Entity)
          .select_also(record_cname::Entity)
          .all(self.db.as_ref())
          .await?
          .into_iter()
          .for_each(|(record, cname)| {
            set.insert(
              Record::from_rdata(
                name.clone(),
                record.ttl as u32,
                RData::CNAME(rdata::CNAME(
                  Name::from_ascii(cname.unwrap().target).unwrap(),
                )),
              ),
              2,
            );
          }),
        RecordType::TXT => query
          .inner_join(record_txt::Entity)
          .select_also(record_txt::Entity)
          .all(self.db.as_ref())
          .await?
          .into_iter()
          .for_each(|(record, txt)| {
            set.insert(
              Record::from_rdata(
                name.clone(),
                record.ttl as u32,
                RData::TXT(rdata::TXT::new(vec![txt.unwrap().content])),
              ),
              2,
            );
          }),
        _ => todo!(),
      }
    }

    Ok(if set.is_empty() {
      let mut set = RecordSet::new(&name, RecordType::CNAME, 0);

      query
        .inner_join(record_cname::Entity)
        .select_also(record_cname::Entity)
        .all(self.db.as_ref())
        .await?
        .into_iter()
        .for_each(|(record, cname)| {
          let target = cname.unwrap().target;

          let n = if target.ends_with('@') {
            let n = Name::from_ascii(&target[..target.len() - 1]).unwrap();
            n.append_name(origin).unwrap()
          } else {
            Name::from_ascii(target).unwrap()
          };

          set.insert(
            Record::from_rdata(
              name.clone(),
              record.ttl as u32,
              RData::CNAME(rdata::CNAME(n)),
            ),
            2,
          );
        });

      Some(set)
    } else {
      Some(set)
    })
  }

  pub(crate) async fn additional_search(
    &self,
    zone_id: Uuid,
    origin: &Name,
    original_name: &LowerName,
    original_query_type: RecordType,
    next_name: LowerName,
    _search_type: RecordType,
    lookup_options: LookupOptions,
  ) -> Option<Vec<Arc<RecordSet>>> {
    let mut additionals: Vec<Arc<RecordSet>> = vec![];

    // if it's a CNAME or other forwarding record, we'll be adding additional records based on the query_type
    let mut query_types_arr = [original_query_type; 2];
    let query_types: &[RecordType] = match original_query_type {
      RecordType::ANAME | RecordType::NS | RecordType::MX | RecordType::SRV => {
        query_types_arr = [RecordType::A, RecordType::AAAA];
        &query_types_arr[..]
      }
      _ => &query_types_arr[..1],
    };

    for query_type in query_types {
      // loop and collect any additional records to send

      // Track the names we've looked up for this query type.
      let mut names = HashSet::new();

      // If we're just going to repeat the same query then bail out.
      if query_type == &original_query_type {
        names.insert(original_name.clone());
      }

      let mut next_name = Some(next_name.clone());
      while let Some(search) = next_name.take() {
        // If we've already looked up this name then bail out.
        if names.contains(&Name::from(search.clone())) {
          break;
        }

        let host = {
          let mut host = String::new();

          let name = Name::from(search.clone());
          let name = name.into_iter().rev().skip(origin.iter().len());
          let mut first = true;
          for label in name {
            if first {
              first = false;
            } else {
              host.write_char('.').unwrap();
            }
            let mut name = Label::from_raw_bytes(label).unwrap();
            name.write_ascii(&mut host).unwrap();
          }

          if host.is_empty() {
            host.write_char('@').unwrap();
          }

          host
        };

        let additional = self
          .lookup(zone_id, origin, &search, &host, *query_type)
          .await
          .unwrap();
        names.insert(search);

        if let Some(additional) = additional {
          let x = Arc::new(additional);
          // assuming no crazy long chains...
          if !additionals.contains(&x) {
            additionals.push(x.clone());
          }

          next_name = maybe_next_name(&x.clone(), *query_type).map(|(name, _search_type)| name);
        }
      }
    }

    if !additionals.is_empty() {
      Some(additionals)
    } else {
      None
    }
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
