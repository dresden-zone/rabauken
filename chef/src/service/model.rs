use entity::prelude::Zone;
use entity::zone;
use sea_orm::entity::EntityTrait;
use sea_orm::{ActiveModelTrait, ActiveValue, PrimaryKeyTrait};
use serde::Deserialize;
use time::OffsetDateTime;
use uuid::Uuid;

pub(crate) trait ToModel<A: EntityTrait, B: ActiveModelTrait<Entity = A>> {
  fn new_with_uuid(self, id: <<A as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType) -> B;
}

#[derive(Deserialize)]
pub(crate) struct ZoneRequest {
  name: String,
}

impl ToModel<Zone, zone::ActiveModel> for ZoneRequest {
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
