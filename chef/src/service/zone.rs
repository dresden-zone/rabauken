use entity::prelude::Zone;
use entity::zone;
use sea_orm::{
  prelude::Uuid, ActiveModelTrait, ActiveValue::Set, DatabaseConnection, DeleteResult, EntityTrait,
  ModelTrait, UpdateResult,
};
use time::OffsetDateTime;

pub(crate) struct ZoneService<'a> {
  db: &'a DatabaseConnection,
}

impl<'a> ZoneService<'a> {
  fn from(db: &'a DatabaseConnection) -> ZoneService {
    ZoneService { db }
  }

  async fn all(&mut self) -> anyhow::Result<Vec<zone::Model>> {
    Ok(Zone::find().all(self.db).await?)
  }

  async fn find(&mut self, id: Uuid) -> anyhow::Result<Option<zone::Model>> {
    Ok(Zone::find_by_id(id).one(self.db).await?)
  }

  async fn create(&mut self, zone: zone::ActiveModel) -> anyhow::Result<zone::Model> {
    Ok(zone.insert(self.db).await?)
  }

  async fn delete(&mut self, id: Uuid) -> anyhow::Result<DeleteResult> {
    let zone = Zone::find_by_id(id).one(self.db).await?.unwrap();
    Ok(zone.delete(self.db).await?)
  }

  async fn update(
    &mut self,
    id: Uuid,
    mut new_zone: zone::ActiveModel,
  ) -> anyhow::Result<zone::Model> {
    let current_time = OffsetDateTime::now_utc();
    new_zone.updated = Set(current_time);
    Ok(new_zone.update(self.db).await?)
  }
}
