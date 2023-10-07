use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use std::sync::Arc;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use sea_orm::prelude::Uuid;
use tracing::info;
use trust_dns_server::proto::rr::{LowerName, Name, rdata, RData, Record, RecordType};

use entity::{record, record_a, record_aaaa, record_mx, record_txt};

pub(crate) struct ZoneService {
  db: Arc<DatabaseConnection>,
}

impl ZoneService {
  pub(crate) fn new(db: Arc<DatabaseConnection>) -> Self {
    Self { db }
  }

  pub(crate) async fn lookup(
    &self,
    zone_id: Uuid,
    name: &LowerName,
    r#type: RecordType,
  ) -> anyhow::Result<Vec<Record>> {
    info!(
      "Looking up on zone {} with name {} and type {}",
      zone_id, name, r#type
    );

    let records = match r#type {
      RecordType::A => record_a::Entity::find()
        .find_also_related(record::Entity)
        .filter(
          record::Column::ZoneId
            .eq(zone_id)
            .and(record::Column::Name.eq(name.to_string())),
        )
        .all(self.db.as_ref())
        .await?
        .into_iter()
        .map(|(a, record)| {
          let model = record.unwrap();
          Record::from_rdata(
            Name::from_ascii(model.name).unwrap(),
            model.ttl as u32,
            RData::A(rdata::A(Ipv4Addr::from_str(&a.address).unwrap())),
          )
        })
        .collect(),
      RecordType::AAAA => record_aaaa::Entity::find()
        .find_also_related(record::Entity)
        .filter(
          record::Column::ZoneId
            .eq(zone_id)
            .and(record::Column::Name.eq(name.to_string())),
        )
        .all(self.db.as_ref())
        .await?
        .into_iter()
        .map(|(a, record)| {
          let model1 = record.unwrap();
          Record::from_rdata(
            Name::from_ascii(model1.name).unwrap(),
            model1.ttl as u32,
            RData::AAAA(rdata::AAAA(Ipv6Addr::from_str(&a.address).unwrap())),
          )
        })
        .collect(),
      RecordType::MX => record_mx::Entity::find()
        .find_also_related(record::Entity)
        .filter(
          record::Column::ZoneId
            .eq(zone_id)
            .and(record::Column::Name.eq(name.to_string())),
        )
        .all(self.db.as_ref())
        .await?
        .into_iter()
        .map(|(a, record)| {
          let model2 = record.unwrap();
          Record::from_rdata(
            Name::from_ascii(model2.name).unwrap(),
            model2.ttl as u32,
            RData::MX(rdata::MX::new(
              a.priority as u16,
              Name::from_ascii(a.target).unwrap(),
            )),
          )
        })
        .collect(),
      // RecordType::NS => record_ns::Entity::find()
      //   .filter(record::Column::ZoneId.eq(zone_id).and(record::Column::Name.eq(name.to_string()))),
      RecordType::TXT => record_txt::Entity::find()
        .find_also_related(record::Entity)
        .filter(
          record::Column::ZoneId
            .eq(zone_id)
            .and(record::Column::Name.eq(name.to_string())),
        )
        .all(self.db.as_ref())
        .await?
        .into_iter()
        .map(|(a, record)| {
          let model3 = record.unwrap();
          Record::from_rdata(
            Name::from_ascii(model3.name).unwrap(),
            model3.ttl as u32,
            RData::TXT(rdata::TXT::new(vec![a.content])),
          )
        })
        .collect(),
      _ => todo!(),
    };

    Ok(records)
  }
}
