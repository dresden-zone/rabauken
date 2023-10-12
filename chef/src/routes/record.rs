use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;

use sea_orm::{EntityTrait, Related};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::service::model::ZoneRequest;
use crate::state::ChefState;

use entity::prelude::{Record, RecordA};
use entity::record;

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

pub(crate) struct ApiRecord<A: EntityTrait> {
  pub record_type: RecordType,
  pub record: record::Model,
  pub value: <A as EntityTrait>::Model,
}

pub(crate) trait MergeObject<A: EntityTrait> {
  fn merge(
    value: (
      <Record as EntityTrait>::Model,
      Option<<A as EntityTrait>::Model>,
    ),
  ) -> ApiRecord<A>;
}

impl MergeObject<RecordA> for ApiRecord<RecordA> {
  fn merge(
    value: (
      entity::record::Model,
      std::option::Option<entity::record_a::Model>,
    ),
  ) -> ApiRecord<RecordA> {
    ApiRecord::<RecordA> {
      record_type: RecordType::A,
      record: value.0,
      value: value.1.unwrap(),
    }
  }
}

// RecordA
pub(crate) async fn list_record<A: MergeObject<A> + Related<Record> + EntityTrait>(
  State(mut state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
) -> Result<Json<Arc<Vec<ApiRecord<RecordA>>>>, StatusCode>
where
  entity::prelude::Record: Related<A>,
  Vec<ApiRecord<entity::prelude::RecordA>>: FromIterator<ApiRecord<A>>,
{
  match state.record_service.all::<A>(zone_id).await {
    Ok(values) => {
      let collection: Vec<ApiRecord<RecordA>> = values.into_iter().map(A::merge).collect();
      Ok(Json(Arc::new(collection)))
    }
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}

pub(crate) async fn create_record(
  State(_state): State<ChefState>,
  Json(_payload): Json<ZoneRequest>,
  Path(_record_type): Path<RecordType>,
) -> Result<Json<Arc<IdResponse>>, StatusCode> {
  Err(StatusCode::NOT_IMPLEMENTED)
}

pub(crate) async fn modify_record(
  State(_state): State<ChefState>,
  Path(_zone_id): Path<Uuid>,
  Path(_record_type): Path<RecordType>,
  Json(_payload): Json<ZoneRequest>,
) -> StatusCode {
  StatusCode::NOT_IMPLEMENTED
}

pub(crate) async fn delete_record(
  State(_state): State<ChefState>,
  Path(_zone_id): Path<Uuid>,
  Path(_record_type): Path<RecordType>,
) -> StatusCode {
  StatusCode::NOT_IMPLEMENTED
}

pub(crate) async fn get_record<A: MergeObject<A> + Related<Record> + EntityTrait>(
  State(_state): State<ChefState>,
  Path(_zone_id): Path<Uuid>,
) -> Result<Json<Arc<ApiRecord<A>>>, StatusCode> {
  Err(StatusCode::NOT_IMPLEMENTED)
}
