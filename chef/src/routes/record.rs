use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use sea_orm::{EntityTrait, PrimaryKeyTrait, Related};
use serde::{Serialize};
use tracing::error;
use uuid::Uuid;

use entity::record;
use session::Session;

use crate::ctx::Context;

#[derive(Serialize)]
pub(crate) struct Record<E: EntityTrait>
  where
    record::Entity: Related<E>,
{
  #[serde(flatten)]
  common: record::Model,
  #[serde(flatten)]
  specific: E::Model,
}

pub(crate) async fn list_records<E: EntityTrait>(
  State(ctx): State<Context>,
  Path(zone_id): Path<Uuid>,
  session: Session,
) -> Result<Json<Vec<Record<E>>>, StatusCode>
  where
    entity::prelude::Record: Related<E>,
{
  let records = ctx
    .record_service
    .list::<E>(session.user_id, zone_id)
    .await
    .map_err(|err| {
      error!("Unable to list zones: {}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?
    .into_iter()
    .map(|(common, specific)| Record { common, specific })
    .collect();

  Ok(Json(records))
}

pub(crate) async fn get_record<E: EntityTrait>(
  State(ctx): State<Context>,
  Path(record_id): Path<Uuid>,
  session: Session,
) -> Result<Json<Record<E>>, StatusCode>
  where
    entity::prelude::Record: Related<E>,
    <<E as EntityTrait>::PrimaryKey as PrimaryKeyTrait>::ValueType: From<Uuid>,
{
  let record = ctx
    .record_service
    .by_id::<E>(session.user_id, record_id)
    .await
    .map_err(|err| {
      error!("Unable to get zone: {}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let (common, specific) = record.ok_or(StatusCode::NOT_FOUND)?;

  Ok(Json(Record { common, specific }))
}
