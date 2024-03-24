use std::sync::Arc;

use sea_orm::{
  ActiveModelBehavior, ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr,
  EntityTrait, PaginatorTrait, QueryFilter, Related, Select, TransactionTrait,
};
use time::OffsetDateTime;
use uuid::Uuid;

use entity::prelude::{Record, Zone};
use entity::{record, zone};
pub(crate) use model::*;

mod model;

fn map_entry<E: EntityTrait>(
  (common, specific): (record::Model, Option<E::Model>),
) -> (record::Model, <E as EntityTrait>::Model) {
  (common, specific.unwrap())
}

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
    // workaround to fix generic type errors
    fn call(user_id: Uuid, zone_id: Uuid) -> Select<record::Entity> {
      Record::find().inner_join(Zone).filter(
        record::Column::ZoneId
          .eq(zone_id)
          .and(zone::Column::Owner.eq(user_id)),
      )
    }

    let zones = call(user_id, zone_id)
      .inner_join(E::default())
      .select_also(E::default())
      .all(self.db.as_ref())
      .await?
      .into_iter()
      .map(map_entry::<E>)
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
    // workaround to fix generic type errors
    fn call(user_id: Uuid, record_id: Uuid) -> Select<record::Entity> {
      Record::find_by_id(record_id)
        .inner_join(Zone)
        .filter(zone::Column::Owner.eq(user_id))
    }

    let zones = call(user_id, record_id)
      .inner_join(E::default())
      .select_also(E::default())
      .one(self.db.as_ref())
      .await?
      .map(map_entry::<E>);

    Ok(zones)
  }

  pub(crate) async fn create<
    A: ActiveModelTrait + ActiveModelBehavior + Send,
    R: RecordRequestTrait<A> + Send + 'static,
  >(
    &self,
    user_id: Uuid,
    zone_id: Uuid,
    name: String,
    ttl: Option<u32>,
    req: R,
  ) -> anyhow::Result<(
    record::Model,
    <<A as ActiveModelTrait>::Entity as EntityTrait>::Model,
  )>
  where
    <<A as ActiveModelTrait>::Entity as EntityTrait>::Model: sea_orm::IntoActiveModel<A>,
  {
    let result = self
      .db
      .transaction(|tx| {
        Box::pin(async move {
          // validate access
          let access = Zone::find_by_id(zone_id)
            .filter(zone::Column::Owner.eq(user_id))
            .count(tx)
            .await?
            == 1;

          // TODO: error handling
          assert!(access);

          let record = record::ActiveModel {
            id: ActiveValue::NotSet,
            created: ActiveValue::NotSet,
            updated: ActiveValue::NotSet,
            name: ActiveValue::Set(name),
            zone_id: ActiveValue::Set(zone_id),
            ttl: ActiveValue::Set(ttl.map(|x| x as i32)),
          };

          let record = record.insert(tx).await?;

          let specific = req
            .into_active_model(ActiveValue::Set(record.id))
            .insert(tx)
            .await?;

          Ok::<
            (
              record::Model,
              <<A as ActiveModelTrait>::Entity as EntityTrait>::Model,
            ),
            DbErr,
          >((record, specific))
        })
      })
      .await?;

    Ok(result)
  }

  pub(crate) async fn modify<
    A: ActiveModelTrait + ActiveModelBehavior + Send,
    R: RecordRequestTrait<A> + Send + 'static,
  >(
    &self,
    user_id: Uuid,
    record_id: Uuid,
    name: String,
    ttl: Option<u32>,
    req: R,
  ) -> anyhow::Result<(
    record::Model,
    <<A as ActiveModelTrait>::Entity as EntityTrait>::Model,
  )>
  where
    <<A as ActiveModelTrait>::Entity as EntityTrait>::Model: sea_orm::IntoActiveModel<A>,
  {
    let result = self
      .db
      .transaction(|tx| {
        Box::pin(async move {
          // validate access
          let access = Record::find_by_id(record_id)
            .inner_join(zone::Entity)
            .filter(zone::Column::Owner.eq(user_id))
            .count(tx)
            .await?
            == 1;

          // TODO: error handling
          assert!(access);

          let now = OffsetDateTime::now_utc();

          let record = record::ActiveModel {
            id: ActiveValue::Unchanged(record_id),
            created: ActiveValue::NotSet,
            updated: ActiveValue::Set(now),
            name: ActiveValue::Set(name),
            zone_id: ActiveValue::NotSet,
            ttl: ActiveValue::Set(ttl.map(|x| x as i32)),
          };

          let record = record.update(tx).await?;

          let specific = req
            .into_active_model(ActiveValue::Unchanged(record.id))
            .update(tx)
            .await?;

          Ok::<
            (
              record::Model,
              <<A as ActiveModelTrait>::Entity as EntityTrait>::Model,
            ),
            DbErr,
          >((record, specific))
        })
      })
      .await?;

    Ok(result)
  }

  pub(crate) async fn delete<E: EntityTrait>(
    &self,
    user_id: Uuid,
    record_id: Uuid,
  ) -> anyhow::Result<bool>
  where
    <<E as EntityTrait>::PrimaryKey as sea_orm::PrimaryKeyTrait>::ValueType: From<Uuid>,
  {
    let result = self
      .db
      .transaction(|tx| {
        Box::pin(async move {
          // validate access
          let access = Record::find_by_id(record_id)
            .filter(zone::Column::Owner.eq(user_id))
            .count(tx)
            .await?
            == 1;

          if !access {
            return Ok::<bool, DbErr>(false);
          }

          // delete
          let result = E::delete_by_id(record_id).exec(tx).await?;

          if result.rows_affected == 0 {
            return Ok::<bool, DbErr>(false);
          }

          let result = Record::delete_by_id(record_id).exec(tx).await?;

          Ok::<bool, DbErr>(result.rows_affected == 1)
        })
      })
      .await?;

    Ok(result)
  }
}
