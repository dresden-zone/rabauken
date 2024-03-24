use sea_orm::{
  ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};
use std::sync::Arc;
use uuid::Uuid;

use entity::prelude::Zone;
use entity::zone;

#[derive(Clone)]
pub(crate) struct ZoneService {
  db: Arc<DatabaseConnection>,
}

impl ZoneService {
  pub(crate) fn new(db: Arc<DatabaseConnection>) -> Self {
    Self { db }
  }

  pub(crate) async fn list(&self, user_id: Uuid) -> anyhow::Result<Vec<zone::Model>> {
    let zones = Zone::find()
      .filter(zone::Column::Owner.eq(user_id))
      .all(self.db.as_ref())
      .await?;

    Ok(zones)
  }

  pub(crate) async fn by_id(
    &self,
    user_id: Uuid,
    zone_id: Uuid,
  ) -> anyhow::Result<Option<zone::Model>> {
    let user = Zone::find_by_id(zone_id)
      .filter(zone::Column::Owner.eq(user_id))
      .one(self.db.as_ref())
      .await?;

    Ok(user)
  }

  pub(crate) async fn create(&self, user_id: Uuid, name: String) -> anyhow::Result<zone::Model> {
    let zone = zone::ActiveModel {
      id: ActiveValue::NotSet,
      created: ActiveValue::NotSet,
      updated: ActiveValue::NotSet,
      name: ActiveValue::Set(name),
      owner: ActiveValue::Set(user_id),
      verified: ActiveValue::NotSet,
      serial: ActiveValue::NotSet,
    };

    let zone = zone.insert(self.db.as_ref()).await?;

    Ok(zone)
  }

  pub(crate) async fn delete(&self, user_id: Uuid, zone_id: Uuid) -> anyhow::Result<bool> {
    let result = Zone::delete_by_id(zone_id)
      .filter(zone::Column::Owner.eq(user_id))
      .exec(self.db.as_ref())
      .await?;

    Ok(result.rows_affected == 1)
  }
}
