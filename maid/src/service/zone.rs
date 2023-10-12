use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::sync::Arc;

use anyhow::anyhow;
use sea_orm::prelude::{Expr, Uuid};
use sea_orm::{
  ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, Related, Select,
};
use time::macros::datetime;
use time::OffsetDateTime;
use trust_dns_server::authority::LookupOptions;
use trust_dns_server::proto::rr::domain::Label;
use trust_dns_server::proto::rr::{rdata, LowerName, Name, RData, Record, RecordSet, RecordType};

use entity::EntityError;
use entity::{record, record_a, record_aaaa, record_cname, record_mx, record_ns, record_txt, zone};

// Thu Oct 12 2023 00:00:00 GMT+0000
const EPOCH: OffsetDateTime = datetime!(2023-10-12 00:00:00 UTC);

pub(crate) struct ZoneService {
  db: Arc<DatabaseConnection>,
}

impl ZoneService {
  pub(crate) fn new(db: Arc<DatabaseConnection>) -> Self {
    Self { db }
  }

  async fn records_serial(&self, zone_id: Uuid) -> anyhow::Result<Option<OffsetDateTime>> {
    Ok(
      record::Entity::find()
        .inner_join(zone::Entity)
        .filter(
          record::Column::ZoneId
            .eq(zone_id)
            .and(Expr::col((zone::Entity, zone::Column::Verified)).eq(Expr::val(true))),
        )
        .select_only()
        .expr(record::Column::Updated.max())
        .into_tuple()
        .one(self.db.as_ref())
        .await?,
    )
  }

  pub(crate) async fn soa(
    &self,
    zone_id: Uuid,
    original: Option<&Name>,
  ) -> anyhow::Result<RecordSet> {
    let zone = zone::Entity::find_by_id(zone_id)
      .filter(Expr::col((zone::Entity, zone::Column::Verified)).eq(Expr::val(true)))
      .one(self.db.as_ref())
      .await?;

    let zone = match zone {
      Some(zone) => zone,
      None => return Err(anyhow!("zone by id not found")),
    };

    let serial = match self.records_serial(zone_id).await? {
      Some(serial) => serial.max(zone.updated),
      None => zone.updated,
    };
    let serial = (serial - EPOCH).whole_seconds() as u32;

    let mut name = Name::from_ascii(zone.name)?;
    name.set_fqdn(true);

    let mut set = RecordSet::new(&name, RecordType::SOA, 0);

    if let Some(original) = original {
      if &name != original {
        return Ok(set);
      }
    }

    set.insert(
      Record::from_rdata(
        name,
        zone.ttl as u32,
        RData::SOA(rdata::SOA::new(
          Name::from_ascii("ns.dns.dresden.zone.")?,
          Name::from_ascii("dns.dresden.zone")?,
          serial,
          zone.refresh,
          zone.retry,
          zone.expire,
          zone.minimum as u32,
        )),
      ),
      0,
    );

    Ok(set)
  }

  pub(crate) async fn lookup_any(&self, zone_id: Uuid) -> anyhow::Result<Vec<RecordSet>> {
    let mut records = Vec::with_capacity(7);

    let soa = self.soa(zone_id, None).await?;
    let origin = soa.name();
    // records.push(soa);

    records.append(
      &mut query_all_records_raw::<record_a::Entity, _>(
        &self.db,
        zone_id,
        origin,
        RecordType::A,
        |model| Ok(model.try_into()?),
      )
      .await?,
    );
    records.append(
      &mut query_all_records_raw::<record_aaaa::Entity, _>(
        &self.db,
        zone_id,
        origin,
        RecordType::AAAA,
        |model| Ok(model.try_into()?),
      )
      .await?,
    );
    records.append(
      &mut query_all_records_raw::<record_cname::Entity, _>(
        &self.db,
        zone_id,
        origin,
        RecordType::CNAME,
        |model| {
          let n = if model.target.ends_with('@') {
            let n = Name::from_ascii(&model.target[..model.target.len() - 1])?;
            n.append_name(origin)?
          } else {
            Name::from_ascii(model.target)?
          };
          Ok(RData::CNAME(rdata::CNAME(n)))
        },
      )
      .await?,
    );

    records.append(
      &mut query_all_records_raw::<record_mx::Entity, _>(
        &self.db,
        zone_id,
        origin,
        RecordType::MX,
        |model| Ok(model.try_into()?),
      )
      .await?,
    );
    records.append(
      &mut query_all_records_raw::<record_ns::Entity, _>(
        &self.db,
        zone_id,
        origin,
        RecordType::NS,
        |model| Ok(model.try_into()?),
      )
      .await?,
    );
    records.append(
      &mut query_all_records_raw::<record_txt::Entity, _>(
        &self.db,
        zone_id,
        origin,
        RecordType::TXT,
        |model| Ok(model.try_into()?),
      )
      .await?,
    );

    Ok(records)
  }
  pub(crate) async fn lookup(
    &self,
    zone_id: Uuid,
    origin: &Name,
    original: &LowerName,
    host: &str,
    record_type: RecordType,
  ) -> anyhow::Result<Option<RecordSet>> {
    let name = original.into();

    let set = match record_type {
      RecordType::SOA => self.soa(zone_id, Some(&name)).await?,
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
          record_type,
          host,
          |model| {
            let n = if model.target.ends_with('@') {
              let n = Name::from_ascii(&model.target[..model.target.len() - 1])?;
              n.append_name(origin)?
            } else {
              Name::from_ascii(model.target)?
            };
            Ok(RData::CNAME(rdata::CNAME(n)))
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
        host,
        |model| {
          let n = if model.target.ends_with('@') {
            let n = Name::from_ascii(&model.target[..model.target.len() - 1])?;
            n.append_name(origin)?
          } else {
            Name::from_ascii(model.target)?
          };
          Ok(RData::CNAME(rdata::CNAME(n)))
        },
      )
      .await?;
      if records.is_empty() {
        None
      } else {
        Some(records)
      }
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
  E: EntityTrait<Model = M>,
  M: TryInto<RData, Error = EntityError>,
  record::Entity: Related<E>,
{
  query_records_raw(db, zone_id, name, record_type, host, |model| {
    Ok(model.try_into()?)
  })
  .await
}

async fn query_records_raw<E, M>(
  db: &DatabaseConnection,
  zone_id: Uuid,
  name: &Name,
  record_type: RecordType,
  host: &str,
  to_record: impl FnOnce(E::Model) -> anyhow::Result<RData> + Copy,
) -> anyhow::Result<RecordSet>
where
  E: EntityTrait<Model = M>,
  M: TryInto<RData, Error = EntityError>,
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

    set.insert(
      Record::from_rdata(name.clone(), record.ttl as u32, to_record(model)?),
      0,
    );
  }

  Ok(set)
}

async fn query_all_records_raw<E, M>(
  db: &DatabaseConnection,
  zone_id: Uuid,
  origin: &Name,
  record_type: RecordType,
  to_record: impl FnOnce(E::Model) -> anyhow::Result<RData> + Copy,
) -> anyhow::Result<Vec<RecordSet>>
where
  E: EntityTrait<Model = M>,
  M: TryInto<RData, Error = EntityError>,
  record::Entity: Related<E>,
{
  fn call(zone_id: Uuid) -> Select<record::Entity> {
    record::Entity::find().inner_join(zone::Entity).filter(
      Expr::col((zone::Entity, zone::Column::Id))
        .eq(Expr::val(zone_id))
        .and(Expr::col((zone::Entity, zone::Column::Verified)).eq(Expr::val(true))),
    )
  }

  let records = call(zone_id)
    .inner_join(E::default())
    .select_also(E::default())
    .all(db)
    .await?;

  let mut set = HashMap::new();

  for (record, model) in records {
    // we are using an inner join, so this can never be none
    let model = model.unwrap();

    let name = if record.name == "@" {
      origin.clone()
    } else {
      Name::from_ascii(record.name)?.append_domain(origin)?
    };

    let record = Record::from_rdata(name.clone(), record.ttl as u32, to_record(model)?);

    match set.entry(name.clone()) {
      Entry::Vacant(vac) => {
        let mut set = RecordSet::new(&name, record_type, 0);
        set.insert(record, 0);
        vac.insert(set);
      }
      Entry::Occupied(mut occ) => {
        occ.get_mut().insert(record, 0);
      }
    };
  }

  Ok(set.into_values().collect())
}
