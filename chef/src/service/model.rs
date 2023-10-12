use entity::prelude::Zone;
use entity::zone;
use sea_orm::entity::EntityTrait;
use sea_orm::{ActiveModelTrait, ActiveValue, PrimaryKeyTrait};
use serde::Deserialize;
use time::OffsetDateTime;
use uuid::Uuid;

pub(crate) trait ToModel<A: EntityTrait, B: ActiveModelTrait<Entity = A>> {
  fn new_with_uuid(&self, id: <<A as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType) -> B;
}

#[derive(Deserialize)]
pub(crate) struct ZoneRequest {
  name: String,
  verified: bool,
}

impl ToModel<Zone, zone::ActiveModel> for ZoneRequest {
  fn new_with_uuid(&self, id: Uuid) -> zone::ActiveModel {
    let current_time = OffsetDateTime::now_utc();
    zone::ActiveModel {
      id: ActiveValue::Set(id),
      created: ActiveValue::Unchanged(current_time),
      updated: ActiveValue::Set(current_time),
      verified: ActiveValue::Set(self.verified),
      name: ActiveValue::Set(self.name.clone()),
    }
  }
}
