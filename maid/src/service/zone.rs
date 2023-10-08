use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::sync::Arc;

use sea_orm::prelude::Expr;
use sea_orm::sea_query::extension::postgres::PgExpr;
use sea_orm::{DatabaseConnection, EntityTrait, QueryFilter};
use trust_dns_server::proto::rr::{rdata, LowerName, Name, RData, Record, RecordType};

use entity::{record, record_a, record_aaaa, record_cname, record_mx, record_ns, record_txt, zone};

pub(crate) struct ZoneService {
  db: Arc<DatabaseConnection>,
}

impl ZoneService {
  pub(crate) fn new(db: Arc<DatabaseConnection>) -> Self {
    Self { db }
  }

  pub(crate) async fn lookup(
    &self,
    name: &LowerName,
    r#type: RecordType,
  ) -> anyhow::Result<Vec<Record>> {
    // println!(
    //   "Looking up on zone {} with name {} and type {}",
    //   zone_id, name, r#type
    // );

    let query = record::Entity::find().inner_join(zone::Entity).filter(
      Expr::col((record::Entity, record::Column::Name))
        .concat(Expr::val("."))
        .concat(Expr::col((zone::Entity, zone::Column::Name)))
        .concat(Expr::val("."))
        .eq(name.to_string()),
    );

    let records = match r#type {
      RecordType::A => query
        .find_also_related(record_a::Entity)
        .all(self.db.as_ref())
        .await?
        .into_iter()
        .map(|(record, a)| {
          Record::from_rdata(
            Name::from_ascii(record.name).unwrap(),
            record.ttl as u32,
            RData::A(rdata::A(Ipv4Addr::from_str(&a.unwrap().address).unwrap())),
          )
        })
        .collect(),
      RecordType::AAAA => query
        .find_also_related(record_aaaa::Entity)
        .all(self.db.as_ref())
        .await?
        .into_iter()
        .map(|(record, aaaa)| {
          Record::from_rdata(
            Name::from_ascii(record.name).unwrap(),
            record.ttl as u32,
            RData::AAAA(rdata::AAAA(
              Ipv6Addr::from_str(&aaaa.unwrap().address).unwrap(),
            )),
          )
        })
        .collect(),
      RecordType::MX => query
        .find_also_related(record_mx::Entity)
        .all(self.db.as_ref())
        .await?
        .into_iter()
        .map(|(record, mx)| {
          let a = mx.unwrap();
          Record::from_rdata(
            Name::from_ascii(record.name).unwrap(),
            record.ttl as u32,
            RData::MX(rdata::MX::new(
              a.preference as u16,
              Name::from_ascii(a.exchange).unwrap(),
            )),
          )
        })
        .collect(),
      RecordType::NS => query
        .find_also_related(record_ns::Entity)
        .all(self.db.as_ref())
        .await?
        .into_iter()
        .map(|(record, ns)| {
          Record::from_rdata(
            Name::from_ascii(record.name).unwrap(),
            record.ttl as u32,
            RData::NS(rdata::NS(Name::from_ascii(ns.unwrap().target).unwrap())),
          )
        })
        .collect(),
      RecordType::CNAME => query
        .find_also_related(record_cname::Entity)
        .all(self.db.as_ref())
        .await?
        .into_iter()
        .map(|(record, cname)| {
          Record::from_rdata(
            Name::from_ascii(record.name).unwrap(),
            record.ttl as u32,
            RData::CNAME(rdata::CNAME(
              Name::from_ascii(cname.unwrap().target).unwrap(),
            )),
          )
        })
        .collect(),
      RecordType::TXT => query
        .find_also_related(record_txt::Entity)
        .all(self.db.as_ref())
        .await?
        .into_iter()
        .map(|(record, txt)| {
          Record::from_rdata(
            Name::from_ascii(record.name).unwrap(),
            record.ttl as u32,
            RData::TXT(rdata::TXT::new(vec![txt.unwrap().content])),
          )
        })
        .collect(),
      _ => todo!(),
    };

    Ok(records)
  }
}
