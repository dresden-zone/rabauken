use std::str::FromStr;
use std::sync::Arc;

use sea_orm::{
  prelude::Uuid, ActiveModelTrait, ActiveValue, DatabaseConnection, DeleteResult, EntityTrait,
  ModelTrait,
};
use serde::Deserialize;
use time::OffsetDateTime;

use entity::prelude::Zone;
use entity::zone;

#[derive(Deserialize)]
pub(crate) struct ZoneRequest {
  name: String,
  verified: bool,
  refresh: u32,
  retry: u32,
  expire: u32,
  minimum: u32,
}

#[derive(Clone)]
pub(crate) struct ZoneService {
  db: Arc<DatabaseConnection>,
}

impl ZoneService {
  pub(crate) fn from_db(db: Arc<DatabaseConnection>) -> ZoneService {
    ZoneService { db }
  }

  pub(crate) async fn all(&mut self) -> anyhow::Result<Vec<zone::Model>> {
    Ok(Zone::find().all(&*self.db).await?)
  }

  pub(crate) async fn find(&mut self, id: Uuid) -> anyhow::Result<Option<zone::Model>> {
    Ok(Zone::find_by_id(id).one(&*self.db).await?)
  }

  pub(crate) async fn create(
    &mut self,
    id: Uuid,
    zone: ZoneRequest,
  ) -> anyhow::Result<zone::Model> {
    let current_time = time::OffsetDateTime::now_utc();
    let database_zone = zone::ActiveModel {
      id: ActiveValue::Set(id),
      created: ActiveValue::Set(current_time),
      updated: ActiveValue::Set(current_time),
      name: ActiveValue::Set(zone.name),
      verified: ActiveValue::Set(zone.verified),
      refresh: ActiveValue::Set(zone.refresh as i32),
      retry: ActiveValue::Set(zone.retry as i32),
      expire: ActiveValue::Set(zone.expire as i32),
      minimum: ActiveValue::Set(zone.minimum as i32),
    };
    Ok(database_zone.insert(&*self.db).await?)
  }

  pub(crate) async fn delete(&mut self, id: Uuid) -> anyhow::Result<DeleteResult> {
    let zone = Zone::find_by_id(id).one(&*self.db).await?.unwrap();
    Ok(zone.delete(&*self.db).await?)
  }

  pub(crate) async fn update(
    &mut self,
    id: Uuid,
    mut zone: ZoneRequest,
  ) -> anyhow::Result<zone::Model> {
    let current_time = OffsetDateTime::now_utc();
    let database_zone = zone::ActiveModel {
      id: ActiveValue::Set(id),
      created: ActiveValue::NotSet,
      updated: ActiveValue::Set(current_time),
      verified: ActiveValue::Set(zone.verified),
      name: ActiveValue::Set(zone.name),
      refresh: ActiveValue::Set(zone.refresh as i32),
      retry: ActiveValue::Set(zone.retry as i32),
      expire: ActiveValue::Set(zone.expire as i32),
      minimum: ActiveValue::Set(zone.minimum as i32),
    };
    Ok(database_zone.update(&*self.db).await?)
  }
}
