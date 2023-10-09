use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::sync::Arc;

use anyhow::anyhow;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use sea_orm::prelude::{Expr, Uuid};
use trust_dns_server::proto::rr::{LowerName, Name, rdata, RData, Record, RecordSet, RecordType};
use trust_dns_server::proto::rr::RecordType::SOA;

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
      Ok(Some(Record::from_rdata(name, 31, RData::SOA(rdata::SOA::new(
        Name::from_ascii("ns.dns.dresden.zone.")?,
        Name::from_ascii("dns.dresden.zone")?,
        1,
        zone.refresh,
        zone.retry,
        zone.expire,
        zone.minimum as u32,
      )))))
    }
  }

  pub(crate) async fn lookup_any(&self, zone_id: Uuid) -> Vec<RecordSet> { todo!() }

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
      Expr::col((zone::Entity, zone::Column::Id)).eq(Expr::val(zone_id))
        .and(Expr::col((record::Entity, record::Column::Name)).eq(host))
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
            set.insert(Record::from_rdata(
              name.clone(),
              record.ttl as u32,
              RData::A(rdata::A(Ipv4Addr::from_str(&a.unwrap().address).unwrap())),
            ), 2);
          }),
        RecordType::AAAA => query
          .inner_join(record_aaaa::Entity)
          .select_also(record_aaaa::Entity)
          .all(self.db.as_ref())
          .await?
          .into_iter()
          .for_each(|(record, aaaa)| {
            set.insert(Record::from_rdata(
              name.clone(),
              record.ttl as u32,
              RData::AAAA(rdata::AAAA(
                Ipv6Addr::from_str(&aaaa.unwrap().address).unwrap(),
              )),
            ), 2);
          }),
        RecordType::MX => query
          .inner_join(record_mx::Entity)
          .select_also(record_mx::Entity)
          .all(self.db.as_ref())
          .await?
          .into_iter()
          .for_each(|(record, mx)| {
            let a = mx.unwrap();
            set.insert(Record::from_rdata(
              name.clone(),
              record.ttl as u32,
              RData::MX(rdata::MX::new(
                a.preference as u16,
                Name::from_ascii(a.exchange).unwrap(),
              )),
            ), 2);
          }),
        RecordType::NS => query
          .inner_join(record_ns::Entity)
          .select_also(record_ns::Entity)
          .all(self.db.as_ref())
          .await?
          .into_iter()
          .for_each(|(record, ns)| {
            set.insert(Record::from_rdata(
              name.clone(),
              record.ttl as u32,
              RData::NS(rdata::NS(Name::from_ascii(ns.unwrap().target).unwrap())),
            ), 2);
          }),
        RecordType::CNAME => query
          .inner_join(record_cname::Entity)
          .select_also(record_cname::Entity)
          .all(self.db.as_ref())
          .await?
          .into_iter()
          .for_each(|(record, cname)| {
            set.insert(Record::from_rdata(
              name.clone(),
              record.ttl as u32,
              RData::CNAME(rdata::CNAME(
                Name::from_ascii(cname.unwrap().target).unwrap(),
              )),
            ), 2);
          }),
        RecordType::TXT => query
          .inner_join(record_txt::Entity)
          .select_also(record_txt::Entity)
          .all(self.db.as_ref())
          .await?
          .into_iter()
          .for_each(|(record, txt)| {
            set.insert(Record::from_rdata(
              name.clone(),
              record.ttl as u32,
              RData::TXT(rdata::TXT::new(vec![txt.unwrap().content])),
            ), 2);
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
            let n = Name::from_ascii(&target[..target.len()-1]).unwrap();
            n.append_name(origin).unwrap()
          } else {
            Name::from_ascii(target).unwrap()
          };

          set.insert(Record::from_rdata(
            name.clone(),
            record.ttl as u32,
            RData::CNAME(rdata::CNAME(
              n,
            )),
          ), 2);
        });

      Some(set)
    } else { Some(set) })
  }
}
