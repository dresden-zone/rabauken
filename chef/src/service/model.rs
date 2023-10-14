use entity::prelude::{
  Record, RecordA, RecordAaaa, RecordCname, RecordMx, RecordNs, RecordTxt, Zone,
};
use entity::{record, record_a, record_aaaa, record_cname, record_mx, record_ns, record_txt, zone};
use sea_orm::entity::EntityTrait;
use sea_orm::{ActiveModelTrait, ActiveValue};
use serde::Deserialize;
use std::net::{Ipv4Addr, Ipv6Addr};
use time::OffsetDateTime;
use uuid::Uuid;

pub(crate) trait ToModel<A: EntityTrait, B: ActiveModelTrait<Entity = A>, C> {
  fn new_with_uuid(self, id: C) -> B;
}

trait CreateRecord {
  fn get_name(&self) -> String;
  fn get_ttl(&self) -> i64;
}

#[derive(Deserialize)]
pub(crate) struct ZoneRequest {
  name: String,
}
fn create_active_model<A>(data: A, id: (Uuid, Uuid)) -> record::ActiveModel
where
  A: CreateRecord,
{
  let current_time = OffsetDateTime::now_utc();
  record::ActiveModel {
    id: ActiveValue::Set(id.0),
    created: ActiveValue::Set(current_time),
    updated: ActiveValue::Set(current_time),
    name: ActiveValue::Set(data.get_name()),
    ttl: ActiveValue::Set(data.get_ttl()),
    zone_id: ActiveValue::Set(id.1),
  }
}

#[derive(Deserialize, Clone)]
pub(crate) struct CreateARecord {
  pub name: String,
  pub ttl: i64,
  pub address: Ipv4Addr,
}

impl CreateRecord for CreateARecord {
  fn get_name(&self) -> String {
    self.name.clone()
  }
  fn get_ttl(&self) -> i64 {
    self.ttl.clone()
  }
}

impl ToModel<Record, record::ActiveModel, (Uuid, Uuid)> for CreateARecord {
  fn new_with_uuid(self, id: (Uuid, Uuid)) -> record::ActiveModel {
    create_active_model(self, id)
  }
}

#[derive(Deserialize, Clone)]
pub(crate) struct CreateAAAARecord {
  pub name: String,
  pub ttl: i64,
  pub address: Ipv6Addr,
}

impl CreateRecord for CreateAAAARecord {
  fn get_name(&self) -> String {
    self.name.clone()
  }
  fn get_ttl(&self) -> i64 {
    self.ttl.clone()
  }
}

impl ToModel<Record, record::ActiveModel, (Uuid, Uuid)> for CreateAAAARecord {
  fn new_with_uuid(self, id: (Uuid, Uuid)) -> record::ActiveModel {
    create_active_model(self, id)
  }
}

#[derive(Deserialize, Clone)]
pub(crate) struct CreateCnameRecord {
  pub name: String,
  pub ttl: i64,
  pub target: String,
}

impl CreateRecord for CreateCnameRecord {
  fn get_name(&self) -> String {
    self.name.clone()
  }
  fn get_ttl(&self) -> i64 {
    self.ttl.clone()
  }
}

impl ToModel<Record, record::ActiveModel, (Uuid, Uuid)> for CreateCnameRecord {
  fn new_with_uuid(self, id: (Uuid, Uuid)) -> record::ActiveModel {
    create_active_model(self, id)
  }
}

#[derive(Deserialize, Clone)]
pub(crate) struct CreateMxRecord {
  pub name: String,
  pub ttl: i64,
  pub preference: i16,
  pub exchange: String,
}

impl CreateRecord for CreateMxRecord {
  fn get_name(&self) -> String {
    self.name.clone()
  }
  fn get_ttl(&self) -> i64 {
    self.ttl.clone()
  }
}

impl ToModel<Record, record::ActiveModel, (Uuid, Uuid)> for CreateMxRecord {
  fn new_with_uuid(self, id: (Uuid, Uuid)) -> record::ActiveModel {
    create_active_model(self, id)
  }
}

#[derive(Deserialize, Clone)]
pub(crate) struct CreateNsRecord {
  pub name: String,
  pub ttl: i64,
  pub target: String,
}

impl CreateRecord for CreateNsRecord {
  fn get_name(&self) -> String {
    self.name.clone()
  }
  fn get_ttl(&self) -> i64 {
    self.ttl.clone()
  }
}

impl ToModel<Record, record::ActiveModel, (Uuid, Uuid)> for CreateNsRecord {
  fn new_with_uuid(self, id: (Uuid, Uuid)) -> record::ActiveModel {
    create_active_model(self, id)
  }
}

#[derive(Deserialize, Clone)]
pub(crate) struct CreateTxtRecord {
  pub name: String,
  pub ttl: i64,
  pub content: String,
}

impl CreateRecord for CreateTxtRecord {
  fn get_name(&self) -> String {
    self.name.clone()
  }
  fn get_ttl(&self) -> i64 {
    self.ttl.clone()
  }
}

impl ToModel<Record, record::ActiveModel, (Uuid, Uuid)> for CreateTxtRecord {
  fn new_with_uuid(self, id: (Uuid, Uuid)) -> record::ActiveModel {
    create_active_model(self, id)
  }
}

impl ToModel<Zone, zone::ActiveModel, Uuid> for ZoneRequest {
  fn new_with_uuid(self, id: Uuid) -> zone::ActiveModel {
    let current_time = OffsetDateTime::now_utc();
    zone::ActiveModel {
      id: ActiveValue::Set(id),
      created: ActiveValue::Set(current_time),
      updated: ActiveValue::Set(current_time),
      name: ActiveValue::Set(self.name),
      verified: ActiveValue::Set(false),
      ttl: ActiveValue::Set(500),
      refresh: ActiveValue::Set(21600),
      retry: ActiveValue::Set(1800),
      expire: ActiveValue::Set(2592000),
      minimum: ActiveValue::Set(1000),
    }
  }
}

impl ToModel<RecordA, record_a::ActiveModel, Uuid> for CreateARecord {
  fn new_with_uuid(self, id: Uuid) -> record_a::ActiveModel {
    record_a::ActiveModel {
      id: ActiveValue::Set(id),
      address: ActiveValue::Set(self.address.to_string()),
    }
  }
}

impl ToModel<RecordAaaa, record_aaaa::ActiveModel, Uuid> for CreateAAAARecord {
  fn new_with_uuid(self, id: Uuid) -> record_aaaa::ActiveModel {
    record_aaaa::ActiveModel {
      id: ActiveValue::Set(id),
      address: ActiveValue::Set(self.address.to_string()),
    }
  }
}

impl ToModel<RecordCname, record_cname::ActiveModel, Uuid> for CreateCnameRecord {
  fn new_with_uuid(self, id: Uuid) -> record_cname::ActiveModel {
    record_cname::ActiveModel {
      id: ActiveValue::Set(id),
      target: ActiveValue::Set(self.target),
    }
  }
}

impl ToModel<RecordMx, record_mx::ActiveModel, Uuid> for CreateMxRecord {
  fn new_with_uuid(self, id: Uuid) -> record_mx::ActiveModel {
    record_mx::ActiveModel {
      id: ActiveValue::Set(id),
      preference: ActiveValue::Set(self.preference),
      exchange: ActiveValue::Set(self.exchange),
    }
  }
}

impl ToModel<RecordNs, record_ns::ActiveModel, Uuid> for CreateNsRecord {
  fn new_with_uuid(self, id: Uuid) -> record_ns::ActiveModel {
    record_ns::ActiveModel {
      id: ActiveValue::Set(id),
      target: ActiveValue::Set(self.target),
    }
  }
}

impl ToModel<RecordTxt, record_txt::ActiveModel, Uuid> for CreateTxtRecord {
  fn new_with_uuid(self, id: Uuid) -> record_txt::ActiveModel {
    record_txt::ActiveModel {
      id: ActiveValue::Set(id),
      content: ActiveValue::Set(self.content),
    }
  }
}
