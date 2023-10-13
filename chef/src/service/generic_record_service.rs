use entity::prelude::Record;
use entity::prelude::Zone;
use entity::record;
use entity::zone;
use sea_orm::entity::EntityTrait;
use sea_orm::{ActiveModelTrait, DatabaseConnection, PrimaryKeyTrait};
use sea_orm::Related;
use std::sync::Arc;
use uuid::Uuid;
use sea_orm::RelationTrait;

#[derive(Clone)]
pub(crate) struct GenericRecordService {
  db: Arc<DatabaseConnection>,
}

impl GenericRecordService {
  pub(crate) fn from_db(db: Arc<DatabaseConnection>) -> GenericRecordService {
    GenericRecordService { db }
  }

  pub(crate) async fn all<A: EntityTrait>(
    &mut self,
    id: Uuid,
  ) -> anyhow::Result<Vec<(zone::Model, record::Model, Option<<A as EntityTrait>::Model>)>>
  where
    entity::prelude::Record: Related<A>,
    entity::prelude::Zone: Related<A>,
  {
    Ok(
      Zone::find_by_id(id)
        .inner_join(entity::zone::Relation::Record.def())
        .inner_join(A::default())
        .select_also(A::default())
        .all(&*self.db)
        .await?,
    )
  }
 /*
  pub(crate) async fn find<A: EntityTrait>(
    &mut self,
    zone_id: uuid,
    record_id: <<A as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
  ) -> anyhow::Result<Option<A::Model>> {
    Ok(Record::find_by_id(zone_id)
        .inner_join(A::default())
        .filter()
        .select_also(A::default())
        .one(&*self.db).await?)
  }

  pub(crate) async fn create<A: EntityTrait, B: ActiveModelTrait<Entity = A>>(
    &mut self,
    id: <<A as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
    data: impl ToModel<A, B>,
  ) -> anyhow::Result<<<A as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType> {
    Ok(
      A::insert(data.new_with_uuid(id))
        .exec(&*self.db)
        .await?
        .last_insert_id,
    )
  }
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
