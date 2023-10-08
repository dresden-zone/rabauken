use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use entity::zone;
use sea_orm::ActiveValue;
use serde::Serialize;
use std::sync::Arc;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::service::zone::ZoneRequest;
use crate::state::ChefState;

#[derive(Serialize)]
pub(crate) struct IdResponse {
  id: Uuid,
}

pub(crate) async fn list_zones(
  State(mut state): State<ChefState>,
) -> Result<Json<Arc<Vec<zone::Model>>>, StatusCode> {
  match state.zone_service.all().await {
    Ok(value) => Ok(Json(Arc::new(value))),
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}

pub(crate) async fn create_zone(
  State(mut state): State<ChefState>,
  Json(payload): Json<ZoneRequest>,
) -> Result<Json<Arc<IdResponse>>, StatusCode> {
  if !payload.valid_email() {
    return Err(StatusCode::BAD_REQUEST);
  }

  let current_time = OffsetDateTime::now_utc();
  let id = Uuid::new_v4();
  match state.zone_service.create(id, payload).await {
    Ok(_) => Ok(Json(Arc::new(IdResponse { id }))),
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}

pub(crate) async fn modify_zone(
  State(mut state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
  Json(payload): Json<ZoneRequest>,
) -> StatusCode {
  if !payload.valid_email() {
    return StatusCode::BAD_REQUEST;
  }

  match state.zone_service.update(zone_id, payload).await {
    Ok(_) => StatusCode::OK,
    Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
  }
}

pub(crate) async fn delete_zone(
  State(mut state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
) -> StatusCode {
  match state.zone_service.delete(zone_id).await {
    Ok(_) => StatusCode::OK,
    Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
  }
}

pub(crate) async fn get_zone(
  State(mut state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
) -> Result<Json<Arc<zone::Model>>, StatusCode> {
  match state.zone_service.find(zone_id).await {
    Ok(value) => value.ok_or(StatusCode::NOT_FOUND).map(Arc::new).map(Json),
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}
