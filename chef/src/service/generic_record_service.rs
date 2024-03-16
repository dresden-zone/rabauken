use std::sync::Arc;

use sea_orm::entity::EntityTrait;
use sea_orm::ColumnTrait;
use sea_orm::{ActiveModelBehavior, Related};
use sea_orm::{ActiveModelTrait, DatabaseConnection, Select};
use sea_orm::{PrimaryKeyTrait, QueryFilter};
use uuid::Uuid;

use entity::prelude::Record;
use entity::record;
use entity::zone;

use crate::service::model::ToModel;

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
        zone::Column::Id
          .eq(zone_id)
          .and(zone::Column::Verified.eq(true)),
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
        .filter(zone::Column::Id.eq(zone_id))
    }

    Ok(
      call(zone_id, record_id)
        .inner_join(E::default())
        .select_also(E::default())
        .one(&*self.db)
        .await?,
    )
  }

  pub(crate) async fn create<Entity, Model, ActiveModel, RequestData>(
    &mut self,
    zone_id: Uuid,
    data: RequestData,
  ) -> anyhow::Result<(record::Model, Option<Model>)>
  where
    Entity: EntityTrait<Model = Model>,
    ActiveModel: ActiveModelTrait<Entity = Entity> + ActiveModelBehavior + Send,
    record::Entity: Related<Entity>,
    RequestData: ToModel<Entity, ActiveModel, Uuid>
      + ToModel<Record, entity::record::ActiveModel, (Uuid, Uuid)>
      + Clone,
    Model: sea_orm::IntoActiveModel<ActiveModel>,
  {
    let record_uuid = Uuid::new_v4();
    let record_result =
      <RequestData as ToModel<Record, record::ActiveModel, (Uuid, Uuid)>>::new_with_uuid(
        data.clone(),
        (record_uuid, zone_id),
      )
      .insert(&*self.db)
      .await?;

    let record_result_special =
      <RequestData as ToModel<Entity, ActiveModel, Uuid>>::new_with_uuid(data, record_uuid)
        .insert(&*self.db)
        .await?;

    Ok((record_result, Some(record_result_special)))
  }

  pub(crate) async fn delete<Entity>(&mut self, record_id: Uuid) -> anyhow::Result<bool>
  where
    Entity: EntityTrait,
    <<Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: From<uuid::Uuid>,
  {
    Ok(
      Entity::delete_by_id(record_id)
        .exec(&*self.db)
        .await?
        .rows_affected
        > 0
        && Record::delete_by_id(record_id)
          .exec(&*self.db)
          .await?
          .rows_affected
          > 0,
    )
  }

  pub(crate) async fn update<Entity, Model, ActiveModel, RequestData>(
    &mut self,
    zone_id: Uuid,
    record_id: Uuid,
    data: RequestData,
  ) -> anyhow::Result<(record::Model, Option<<Entity as EntityTrait>::Model>)>
  where
    Entity: EntityTrait<Model = Model>,
    ActiveModel: ActiveModelTrait<Entity = Entity> + sea_orm::ActiveModelBehavior + Send,
    record::Entity: Related<Entity>,
    RequestData: ToModel<Entity, ActiveModel, Uuid>
      + ToModel<Record, entity::record::ActiveModel, (Uuid, Uuid)>
      + Clone,
    Model: sea_orm::IntoActiveModel<ActiveModel>,
  {
    let record_model =
      <RequestData as ToModel<Record, record::ActiveModel, (Uuid, Uuid)>>::new_with_uuid(
        data.clone(),
        (record_id, zone_id),
      )
      .update(&*self.db)
      .await?;

    let record_model_special =
      <RequestData as ToModel<Entity, ActiveModel, Uuid>>::new_with_uuid(data, record_id)
        .update(&*self.db)
        .await?;

    Ok((record_model, Some(record_model_special)))
  }
}
