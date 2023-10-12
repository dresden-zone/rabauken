use std::collections::HashSet;
use std::fmt::Write;
use std::sync::Arc;

use anyhow::anyhow;
use sea_orm::{
  DatabaseConnection, EntityTrait, QueryFilter, Related, Select,
};
use sea_orm::prelude::{Expr, Uuid};
use trust_dns_server::authority::LookupOptions;
use trust_dns_server::proto::rr::{
  LowerName, Name, rdata, RData, Record, RecordData, RecordSet, RecordType,
};
use trust_dns_server::proto::rr::domain::Label;
use trust_dns_server::proto::rr::RecordType::SOA;

use entity::{record, record_a, record_aaaa, record_cname, record_mx, record_ns, record_txt, zone};
use entity::EntityError;

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
          10,
          10,
          10,
          10,
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

    let set = match r#type {
      record_type @ RecordType::A => {
        query_records::<record_a::Entity, _>(&self.db, zone_id, &name, record_type, host).await?
      }
      record_type @ RecordType::AAAA => {
        query_records::<record_aaaa::Entity, _>(&self.db, zone_id, &name, record_type, host).await?
      }
      record_type @ RecordType::MX => {
        query_records::<record_mx::Entity, _>(&self.db, zone_id, &name, record_type, host).await?
      }
      record_type @ RecordType::NS => {
        query_records::<record_ns::Entity, _>(&self.db, zone_id, &name, record_type, host).await?
      }
      record_type @ RecordType::CNAME => {
        query_records_raw::<record_cname::Entity, _>(
          &self.db,
          zone_id,
          &name,
          RecordType::CNAME,
          &host,
          |(record, model)| {
            let n = if model.target.ends_with('@') {
              let n = Name::from_ascii(&model.target[..model.target.len() - 1])?;
              n.append_name(origin)?
            } else {
              Name::from_ascii(model.target)?
            };
            Ok(Record::from_rdata(
              name.clone(),
              record.ttl as u32,
              RData::CNAME(rdata::CNAME(n)),
            ))
          },
        )
          .await?
      }
      record_type @ RecordType::TXT => {
        query_records::<record_txt::Entity, _>(&self.db, zone_id, &name, record_type, host).await?
      }
      _ => todo!(),
    };

    Ok(if set.is_empty() {
      let records = query_records_raw::<record_cname::Entity, _>(
        &self.db,
        zone_id,
        &name,
        RecordType::CNAME,
        &host,
        |(record, model)| {
          let n = if model.target.ends_with('@') {
            let n = Name::from_ascii(&model.target[..model.target.len() - 1])?;
            n.append_name(origin)?
          } else {
            Name::from_ascii(model.target)?
          };
          Ok(Record::from_rdata(
            name.clone(),
            record.ttl as u32,
            RData::CNAME(rdata::CNAME(n)),
          ))
        },
      )
        .await?;

      Some(records)
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
            let name = Label::from_raw_bytes(label).unwrap();
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

async fn query_records<E, M>(
  db: &DatabaseConnection,
  zone_id: Uuid,
  name: &Name,
  record_type: RecordType,
  host: &str,
) -> anyhow::Result<RecordSet>
  where
    E: EntityTrait<Model=M>,
    M: TryInto<RData, Error=EntityError>,
    record::Entity: Related<E>,
{
  query_records_raw(db, zone_id, name, record_type, host, |(record, model)| {
    Ok(Record::from_rdata(
      name.clone(),
      record.ttl as u32,
      model.try_into()?,
    ))
  })
    .await
}

async fn query_records_raw<E, M>(
  db: &DatabaseConnection,
  zone_id: Uuid,
  name: &Name,
  record_type: RecordType,
  host: &str,
  to_record: impl FnOnce((record::Model, E::Model)) -> anyhow::Result<Record<RData>> + Copy,
) -> anyhow::Result<RecordSet>
  where
    E: EntityTrait<Model=M>,
    M: TryInto<RData, Error=EntityError>,
    record::Entity: Related<E>,
{
  fn call(
    zone_id: Uuid,
    name: &Name,
    record_type: RecordType,
    host: &str,
  ) -> (RecordSet, Select<record::Entity>) {
    let set = RecordSet::new(name, record_type, 0);

    let query = record::Entity::find()
      .inner_join::<zone::Entity>(zone::Entity)
      .filter(
        Expr::col((zone::Entity, zone::Column::Id))
          .eq(Expr::val(zone_id))
          .and(Expr::col((zone::Entity, zone::Column::Verified)).eq(Expr::val(true)))
          .and(Expr::col((record::Entity, record::Column::Name)).eq(host)),
      );

    (set, query)
  }

  let (mut set, query) = call(zone_id, name, record_type, host);

  let records = query
    .inner_join(E::default())
    .select_also(E::default())
    .all(db)
    .await?;

  for (record, model) in records {
    // we are using an inner join, so this can never be none
    let model = model.unwrap();

    set.insert(to_record((record, model))?, 1);
  }

  Ok(set)
}
