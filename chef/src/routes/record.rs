use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{debug_handler, Json};

use sea_orm::{EntityTrait, Related};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::routes::record_error;
use crate::service::model::ZoneRequest;
use crate::state::ChefState;
use crate::service::merge::{ApiRecord, MergeObject, RecordType};

use entity::prelude::{
  Record, RecordA, RecordAaaa, RecordCname, RecordMx, RecordNs, RecordTxt, Zone,
};
use entity::{record, IntoRecord};

#[derive(Serialize)]
pub(crate) struct IdResponse {
  id: Uuid,
}

// RecordA
pub(crate) async fn list_record<E, M>(
  State(mut state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
) -> Result<Json<Arc<Vec<ApiRecord<M>>>>, StatusCode>
where
  E: EntityTrait<Model = M>,
  record::Entity: Related<E>,
  ApiRecord<M>: MergeObject<E, M>,
{
  match state.record_service.all::<E, _>(zone_id).await {
    Ok(values) => Ok(Json(Arc::new(
      values.into_iter().map(ApiRecord::<M>::merge).collect(),
    ))),
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

pub(crate) async fn get_record<E, M>(
  State(mut state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
  Path(record_id): Path<Uuid>,
) -> Result<Json<Arc<ApiRecord<M>>>, StatusCode>
where
  E: EntityTrait<Model = M>,
  record::Entity: Related<E>,
  ApiRecord<M>: MergeObject<E, M>,
{
  match state
    .record_service
    .get_record::<E, _>(zone_id, record_id)
    .await
  {
    Ok(Some(values)) => Ok(Json(Arc::new(ApiRecord::<M>::merge(values)))),
    Ok(None) => Err(StatusCode::NOT_FOUND),
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }
}
