use email_address::EmailAddress;
use entity::prelude::Zone;
use entity::zone;
use sea_orm::{
  prelude::Uuid, ActiveModelTrait, ActiveValue, ActiveValue::Set, DatabaseConnection, DeleteResult,
  EntityTrait, ModelTrait, UpdateResult,
};
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use time::OffsetDateTime;

#[derive(Deserialize)]
pub(crate) struct ZoneRequest {
  name: String,
  admin: String,
  refresh: u32,
  retry: u32,
  expire: u32,
  minimum: u32,
}

impl ZoneRequest {
  pub(crate) fn valid_email(&self) -> bool {
    EmailAddress::from_str(&self.admin).is_ok()
  }
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
      admin: ActiveValue::Set(zone.admin),
      name: ActiveValue::Set(zone.name),
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
      admin: ActiveValue::Set(zone.admin),
      name: ActiveValue::Set(zone.name),
      refresh: ActiveValue::Set(zone.refresh as i32),
      retry: ActiveValue::Set(zone.retry as i32),
      expire: ActiveValue::Set(zone.expire as i32),
      minimum: ActiveValue::Set(zone.minimum as i32),
    };
    Ok(database_zone.update(&*self.db).await?)
  }
}
