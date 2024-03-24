use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use sea_orm::{ActiveModelBehavior, ActiveModelTrait, EntityTrait, PrimaryKeyTrait, Related};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use entity::prelude::Record;
use entity::record;
use session::Session;

use crate::ctx::Context;
use crate::service::RecordRequestTrait;

#[derive(Serialize)]
pub(crate) struct RecordResponse<E: EntityTrait>
where
  Record: Related<E>,
{
  #[serde(flatten)]
  common: record::Model,
  #[serde(flatten)]
  specific: E::Model,
}

#[derive(Deserialize)]
pub(crate) struct RecordRequest<S> {
  #[serde(flatten)]
  common: RecordCommonReq,
  #[serde(flatten)]
  specific: S,
}

#[derive(Deserialize)]
pub(crate) struct RecordCommonReq {
  name: String,
  ttl: Option<u32>,
}

pub(crate) async fn list_records<E: EntityTrait>(
  State(ctx): State<Context>,
  Path(zone_id): Path<Uuid>,
  session: Session<ROLE_DNS>,
) -> Result<Json<Vec<RecordResponse<E>>>, StatusCode>
where
  Record: Related<E>,
{
  let records = ctx
    .record_service
    .list::<E>(session.user_id, zone_id)
    .await
    .map_err(|err| {
      error!("Unable to list records: {}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?
    .into_iter()
    .map(|(common, specific)| RecordResponse { common, specific })
    .collect();

  Ok(Json(records))
}

pub(crate) async fn get_record<E: EntityTrait>(
  State(ctx): State<Context>,
  Path(record_id): Path<Uuid>,
  session: Session<ROLE_DNS>,
) -> Result<Json<RecordResponse<E>>, StatusCode>
where
  Record: Related<E>,
  <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Uuid>,
{
  let record = ctx
    .record_service
    .by_id::<E>(session.user_id, record_id)
    .await
    .map_err(|err| {
      error!("Unable to get record: {}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let (common, specific) = record.ok_or(StatusCode::NOT_FOUND)?;

  Ok(Json(RecordResponse { common, specific }))
}

pub(crate) async fn create_record<
  R: RecordRequestTrait<A> + Send + 'static,
  A: ActiveModelTrait + ActiveModelBehavior + Send,
>(
  State(ctx): State<Context>,
  Path(zone_id): Path<Uuid>,
  session: Session<ROLE_DNS>,
  Json(req): Json<RecordRequest<R>>,
) -> Result<Json<RecordResponse<<A as ActiveModelTrait>::Entity>>, StatusCode>
where
  Record: Related<A::Entity>,
  <<A::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Uuid>,
  <<A as ActiveModelTrait>::Entity as EntityTrait>::Model: sea_orm::IntoActiveModel<A>,
{
  let (common, specific) = ctx
    .record_service
    .create(
      session.user_id,
      zone_id,
      req.common.name,
      req.common.ttl,
      req.specific,
    )
    .await
    .map_err(|err| {
      error!("Unable to create record: {}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(RecordResponse { common, specific }))
}
pub(crate) async fn modify_record<
  R: RecordRequestTrait<A> + Send + 'static,
  A: ActiveModelTrait + ActiveModelBehavior + Send,
>(
  State(ctx): State<Context>,
  Path(record_id): Path<Uuid>,
  session: Session<ROLE_DNS>,
  Json(req): Json<RecordRequest<R>>,
) -> Result<Json<RecordResponse<<A as ActiveModelTrait>::Entity>>, StatusCode>
where
  Record: Related<A::Entity>,
  <<A::Entity as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Uuid>,
  <<A as ActiveModelTrait>::Entity as EntityTrait>::Model: sea_orm::IntoActiveModel<A>,
{
  let (common, specific) = ctx
    .record_service
    .modify(
      session.user_id,
      record_id,
      req.common.name,
      req.common.ttl,
      req.specific,
    )
    .await
    .map_err(|err| {
      error!("Unable to modify record: {}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(RecordResponse { common, specific }))
}

pub(crate) async fn delete_record<E: EntityTrait>(
  State(ctx): State<Context>,
  Path(record_id): Path<Uuid>,
  session: Session<ROLE_DNS>,
) -> Result<StatusCode, StatusCode>
where
  Record: Related<E>,
  <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Uuid>,
{
  let found = ctx
    .record_service
    .delete::<E>(session.user_id, record_id)
    .await
    .map_err(|err| {
      error!("Unable to delete record: {}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  if found {
    Ok(StatusCode::NO_CONTENT)
  } else {
    Err(StatusCode::NOT_FOUND)
  }
}
