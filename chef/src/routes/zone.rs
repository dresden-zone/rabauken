use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use email_address::EmailAddress;
use entity::zone;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::state::ChefState;

#[derive(Deserialize)]
pub(crate) struct CreateZoneRequest {
  name: String,
  admin: String,
  refresh: u32,
  retry: u32,
  expire: u32,
  minimum: u32,
}

#[derive(Serialize)]
pub(crate) struct IdResponse {
  id: Uuid,
}

impl CreateZoneRequest {
  fn valid_email(&self) -> bool {
    EmailAddress::from_str(&self.admin).is_ok()
  }
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
  Json(payload): Json<CreateZoneRequest>,
) -> Result<Json<Arc<IdResponse>>, StatusCode> {
  if !payload.valid_email() {
    return Err(StatusCode::BAD_REQUEST);
  }

  let current_time = OffsetDateTime::now_utc();
  let id = Uuid::new_v4();
  match state
    .zone_service
    .create(zone::ActiveModel {
      id: ActiveValue::Set(id),
      created: ActiveValue::Set(current_time),
      updated: ActiveValue::Set(current_time),
      admin: ActiveValue::Set(payload.admin),
      name: ActiveValue::Set(payload.name),
      refresh: ActiveValue::Set(payload.refresh as i32),
      retry: ActiveValue::Set(payload.retry as i32),
      expire: ActiveValue::Set(payload.expire as i32),
      minimum: ActiveValue::Set(payload.minimum as i32),
    })
    .await
  {
    Ok(_) => Ok(Json(Arc::new(IdResponse { id }))),
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}
