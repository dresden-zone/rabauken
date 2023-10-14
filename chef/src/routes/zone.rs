use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use crate::routes::record_error;
use entity::prelude::Zone;
use entity::zone;

use crate::service::model::ZoneRequest;
use crate::state::ChefState;

#[derive(Serialize)]
pub(crate) struct IdResponse {
  id: Uuid,
}

pub(crate) async fn list_zones(
  State(mut state): State<ChefState>,
) -> Result<Json<Arc<Vec<zone::Model>>>, StatusCode> {
  match state.database.all::<Zone>().await {
    Ok(value) => Ok(Json(Arc::new(value))),
    Err(e) => {
      record_error(e);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

pub(crate) async fn create_zone(
  State(mut state): State<ChefState>,
  Json(payload): Json<ZoneRequest>,
) -> Result<Json<Arc<IdResponse>>, StatusCode> {
  let id = Uuid::new_v4();
  match state
    .database
    .create::<Zone, zone::ActiveModel>(id, payload)
    .await
  {
    Ok(_) => Ok(Json(Arc::new(IdResponse { id }))),
    Err(e) => {
      record_error(e);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}

pub(crate) async fn modify_zone(
  State(mut state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
  Json(payload): Json<ZoneRequest>,
) -> StatusCode {
  match state
    .database
    .update::<Zone, zone::ActiveModel>(zone_id, payload)
    .await
  {
    Ok(_) => StatusCode::OK,
    Err(e) => {
      record_error(e);
      StatusCode::INTERNAL_SERVER_ERROR
    }
  }
}

pub(crate) async fn delete_zone(
  State(mut state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
) -> StatusCode {
  match state.database.delete::<Zone>(zone_id).await {
    Ok(_) => StatusCode::OK,
    Err(e) => {
      record_error(e);
      StatusCode::INTERNAL_SERVER_ERROR
    }
  }
}

pub(crate) async fn get_zone(
  State(mut state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
) -> Result<Json<Arc<zone::Model>>, StatusCode> {
  match state.database.find::<Zone>(zone_id).await {
    Ok(value) => value.ok_or(StatusCode::NOT_FOUND).map(Arc::new).map(Json),
    Err(e) => {
      record_error(e);
      Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
  }
}
