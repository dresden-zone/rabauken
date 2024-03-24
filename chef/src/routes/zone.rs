use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use tracing::error;
use uuid::Uuid;

use entity::zone;
use session::Session;

use crate::ctx::Context;

#[derive(Deserialize)]
pub(crate) struct CreateZoneRequest {
  name: String,
}

pub(crate) async fn list_zones(
  State(ctx): State<Context>,
  session: Session,
) -> Result<Json<Vec<zone::Model>>, StatusCode> {
  let zones = ctx
    .zone_service
    .list(session.user_id)
    .await
    .map_err(|err| {
      error!("Unable to list zones: {}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(zones))
}

pub(crate) async fn get_zone(
  State(ctx): State<Context>,
  Path(zone_id): Path<Uuid>,
  session: Session,
) -> Result<Json<zone::Model>, StatusCode> {
  let zone = ctx
    .zone_service
    .by_id(session.user_id, zone_id)
    .await
    .map_err(|err| {
      error!("Unable to get zone: {}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  let zone = zone.ok_or(StatusCode::NOT_FOUND)?;

  Ok(Json(zone))
}

pub(crate) async fn create_zone(
  State(ctx): State<Context>,
  session: Session,
  Json(req): Json<CreateZoneRequest>,
) -> Result<Json<zone::Model>, StatusCode> {
  let zone = ctx
    .zone_service
    .create(session.user_id, req.name)
    .await
    .map_err(|err| {
      error!("Unable to create zone: {}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  Ok(Json(zone))
}

pub(crate) async fn delete_zone(
  State(ctx): State<Context>,
  Path(zone_id): Path<Uuid>,
  session: Session,
) -> Result<StatusCode, StatusCode> {
  let found = ctx
    .zone_service
    .delete(session.user_id, zone_id)
    .await
    .map_err(|err| {
      error!("Unable to delete zone: {}", err);
      StatusCode::INTERNAL_SERVER_ERROR
    })?;

  if found {
    Ok(StatusCode::NO_CONTENT)
  } else {
    Err(StatusCode::NOT_FOUND)
  }
}
