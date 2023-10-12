use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::service::model::ZoneRequest;
use crate::state::ChefState;

use entity::prelude::Record;

#[derive(Serialize)]
pub(crate) struct IdResponse {
  id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) enum RecordType {
  #[serde(rename = "a")]
  A,
  #[serde(rename = "aaaa")]
  AAAA,
  #[serde(rename = "cname")]
  CNAME,
  #[serde(rename = "mx")]
  MX,
  #[serde(rename = "ns")]
  NS,
  #[serde(rename = "txt")]
  TXT,
}

pub(crate) struct ApiRecord<A> {
  pub record_type: RecordType,
  pub record: Record,
  pub value: A,
}

pub(crate) async fn list_record<A, B>(
  State(mut state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
) -> Result<Json<Arc<Vec<ApiRecord<A>>>>, StatusCode> {
  Err(StatusCode::NOT_IMPLEMENTED)
}

pub(crate) async fn create_record(
  State(mut state): State<ChefState>,
  Json(payload): Json<ZoneRequest>,
  Path(record_type): Path<RecordType>,
) -> Result<Json<Arc<IdResponse>>, StatusCode> {
  Err(StatusCode::NOT_IMPLEMENTED)
}

pub(crate) async fn modify_record(
  State(mut state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
  Path(record_type): Path<RecordType>,
  Json(payload): Json<ZoneRequest>,
) -> StatusCode {
  StatusCode::NOT_IMPLEMENTED
}

pub(crate) async fn delete_record(
  State(mut state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
  Path(record_type): Path<RecordType>,
) -> StatusCode {
  StatusCode::NOT_IMPLEMENTED
}

pub(crate) async fn get_record<A, B>(
  State(mut state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
) -> Result<Json<Arc<ApiRecord<A>>>, StatusCode> {
  Err(StatusCode::NOT_IMPLEMENTED)
}
