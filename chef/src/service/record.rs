use std::sync::Arc;

use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, Related};
use uuid::Uuid;

use entity::prelude::{Record, Zone};
use entity::{record, zone};

#[derive(Clone)]
pub(crate) struct RecordService {
  db: Arc<DatabaseConnection>,
}

impl RecordService {
  pub(crate) fn new(db: Arc<DatabaseConnection>) -> Self {
    Self { db }
  }

  pub(crate) async fn list<E: EntityTrait>(
    &self,
    user_id: Uuid,
    zone_id: Uuid,
  ) -> anyhow::Result<Vec<(record::Model, E::Model)>>
  where
    record::Entity: Related<E>,
  {
    let zones = Record::find()
      .inner_join(Zone)
      .inner_join(E::default())
      .filter(
        record::Column::ZoneId
          .eq(zone_id)
          .and(zone::Column::Owner.eq(user_id)),
      )
      .select_also(E::default())
      .all(self.db.as_ref())
      .await?
      .into_iter()
      .map(|(common, specific)| (common, specific.unwrap()))
      .collect();

    Ok(zones)
  }

  pub(crate) async fn by_id<E: EntityTrait>(
    &self,
    user_id: Uuid,
    record_id: Uuid,
  ) -> anyhow::Result<Option<(record::Model, E::Model)>>
  where
    record::Entity: Related<E>,
    <<E as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType: From<Uuid>,
  {
    let zones = Record::find_by_id(record_id)
      .inner_join(Zone)
      .inner_join(E::default())
      .filter(zone::Column::Owner.eq(user_id))
      .select_also(E::default())
      .one(self.db.as_ref())
      .await?
      .map(|(common, specific)| (common, specific.unwrap()));

    Ok(zones)
  }

  pub(crate) async fn create(&self, user_id: Uuid, name: String) -> anyhow::Result<zone::Model> {
    todo!()
  }

  pub(crate) async fn delete<E: EntityTrait>(
    &self,
    user_id: Uuid,
    record_id: Uuid,
  ) -> anyhow::Result<bool>
  where
    <<E as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType: From<Uuid>,
  {
    // validate access
    let access = Record::find_by_id(record_id)
      .filter(zone::Column::Owner.eq(user_id))
      .count(self.db.as_ref())
      .await?
      == 1;

    if !access {
      return Ok(false);
    }

    // delete
    let result = E::delete_by_id(record_id).exec(self.db.as_ref()).await?;

    if result.rows_affected == 0 {
      return Ok(false);
    }

    let result = Record::delete_by_id(record_id)
      .exec(self.db.as_ref())
      .await?;

    Ok(result.rows_affected == 1)
  }
}
