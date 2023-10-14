use crate::service::model::ToModel;
use sea_orm::entity::EntityTrait;
use sea_orm::{ActiveModelTrait, DatabaseConnection, IntoActiveModel, PrimaryKeyTrait};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct GenericDatabaseService {
  db: Arc<DatabaseConnection>,
}

impl GenericDatabaseService {
  pub(crate) fn from_db(db: Arc<DatabaseConnection>) -> GenericDatabaseService {
    GenericDatabaseService { db }
  }

  pub(crate) async fn all<A: EntityTrait>(&mut self) -> anyhow::Result<Vec<A::Model>> {
    Ok(A::find().all(&*self.db).await?)
  }
  pub(crate) async fn find<A: EntityTrait>(
    &mut self,
    id: <<A as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
  ) -> anyhow::Result<Option<A::Model>> {
    Ok(A::find_by_id(id).one(&*self.db).await?)
  }

  pub(crate) async fn create<A: EntityTrait, B: ActiveModelTrait<Entity = A>>(
    &mut self,
    id: <<A as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType,
    data: impl ToModel<A, B, <<A as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>,
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
    data: impl ToModel<A, B, <<A as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType>,
  ) -> anyhow::Result<A::Model>
  where
    <A as EntityTrait>::Model: IntoActiveModel<B>,
  {
    Ok(data.new_with_uuid(id).update(self.db.as_ref()).await?)
  }
}
