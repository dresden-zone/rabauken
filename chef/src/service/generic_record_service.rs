use crate::service::model::ToModel;
use entity::prelude::Record;
use entity::prelude::Zone;
use entity::zone;
use entity::{record, IntoRecord};
use sea_orm::entity::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::Related;
use sea_orm::{ActiveModelTrait, DatabaseConnection, PrimaryKeyTrait, Select};
use sea_query::Expr;
use std::sync::Arc;
use uuid::Uuid;
use entity::zone::Model;

#[derive(Clone)]
pub(crate) struct GenericRecordService {
  db: Arc<DatabaseConnection>,
}

impl GenericRecordService {
  pub(crate) fn from_db(db: Arc<DatabaseConnection>) -> GenericRecordService {
    GenericRecordService { db }
  }

  pub(crate) async fn all<E, M>(
    &mut self,
    zone_id: Uuid,
  ) -> anyhow::Result<Vec<(record::Model, Option<<E as EntityTrait>::Model>)>>
  where
    E: EntityTrait<Model = M>,
    record::Entity: Related<E>,
  {
    fn call(zone_id: Uuid) -> Select<record::Entity> {
      record::Entity::find().inner_join(zone::Entity).filter(
        Expr::col((zone::Entity, zone::Column::Id))
          .eq(Expr::val(zone_id))
          .and(Expr::col((zone::Entity, zone::Column::Verified)).eq(Expr::val(true))),
      )
    }

    Ok(
      call(zone_id)
        .inner_join(E::default())
        .select_also(E::default())
        .all(&*self.db)
        .await?,
    )
  }
  pub(crate) async fn get_record<E, M>(
    &mut self,
    zone_id: Uuid,
    record_id: Uuid,
  ) -> anyhow::Result<Option<(record::Model, Option<<E as EntityTrait>::Model>)>>
  where
    E: EntityTrait<Model = M>,
    record::Entity: Related<E>,
  {
    fn call(zone_id: Uuid, record_id: Uuid) -> Select<record::Entity> {
      record::Entity::find_by_id(record_id)
        .inner_join(zone::Entity)
        .filter(Expr::col((zone::Entity, zone::Column::Id)).eq(Expr::val(zone_id)))
    }

    Ok(
      call(zone_id, record_id)
        .inner_join(E::default())
        .select_also(E::default())
        .one(&*self.db)
        .await?,
    )
  }

  pub(crate) async fn create<E, M, A, D>(
    &mut self,
    zone_id: Uuid,
    data: D,
  ) -> anyhow::Result<<<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>
  where
    E: EntityTrait<Model = M>,
    A: ActiveModelTrait<Entity = E>,
    record::Entity: Related<E>,
    D: ToModel<E, A>
  {
    Ok(
      Record::insert(data.new_with_uuid(zone_id))
        .exec(&*self.db)
        .await?
        .last_insert_id,
    )
  }

  /*


  pub(crate) async fn delete<A: EntityTrait>(
    &mut self,
    id: <<A as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
  ) -> anyhow::Result<u64> {
    Ok(A::delete_by_id(id).exec(&*self.db).await?.rows_affected)
  }

  pub(crate) async fn update<
    A: EntityTrait,
    B: ActiveModelTrait<Entity = A> + sea_orm::ActiveModelBehavior + std::marker::Send,
  >(
    &mut self,
    id: <<A as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
    data: impl ToModel<A, B>,
  ) -> anyhow::Result<A::Model>
  where
    <A as EntityTrait>::Model: IntoActiveModel<B>,
  {
    Ok(data.new_with_uuid(id).update(self.db.as_ref()).await?)
  }*/
}
