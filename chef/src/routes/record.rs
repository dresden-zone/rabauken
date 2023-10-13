use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{debug_handler, Json};

use sea_orm::{EntityTrait, Related};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::service::model::ZoneRequest;
use crate::state::ChefState;
use crate::routes::record_error;

use entity::prelude::{Record, RecordA, RecordAaaa, RecordCname, RecordMx, RecordNs, RecordTxt, Zone};
use entity::{IntoRecord, record};

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

#[derive(Serialize)]
pub(crate) struct ApiRecord<M> {
  pub record_type: RecordType,
  pub record: record::Model,
  pub value: M,
}

pub(crate) trait MergeObject<E: EntityTrait<Model = M>, M> {
  fn merge(
    value: (
      record::Model,
      Option<M>,
    ),
  ) -> ApiRecord<M>;
}

impl MergeObject<RecordA, entity::record_a::Model> for ApiRecord<entity::record_a::Model> {
  fn merge(
    value: (
      record::Model,
      Option<entity::record_a::Model>,
    ),
  ) -> ApiRecord<entity::record_a::Model> {
    ApiRecord::<entity::record_a::Model> {
      record_type: RecordType::A,
      record: value.0,
      value: value.1.unwrap(),
    }
  }
}

impl MergeObject<RecordAaaa, entity::record_aaaa::Model> for ApiRecord<entity::record_aaaa::Model> {
  fn merge(
    value: (
      record::Model,
      Option<entity::record_aaaa::Model>,
    ),
  ) -> ApiRecord<entity::record_aaaa::Model> {
    ApiRecord::<entity::record_aaaa::Model> {
      record_type: RecordType::AAAA,
      record: value.0,
      value: value.1.unwrap(),
    }
  }
}

impl MergeObject<RecordCname, entity::record_cname::Model> for ApiRecord<entity::record_cname::Model> {
  fn merge(
    value: (
      record::Model,
      Option<entity::record_cname::Model>,
    ),
  ) -> ApiRecord<entity::record_cname::Model> {
    ApiRecord::<entity::record_cname::Model> {
      record_type: RecordType::CNAME,
      record: value.0,
      value: value.1.unwrap(),
    }
  }
}

impl MergeObject<RecordMx, entity::record_mx::Model> for ApiRecord<entity::record_mx::Model> {
  fn merge(
    value: (
      record::Model,
      Option<entity::record_mx::Model>,
    ),
  ) -> ApiRecord<entity::record_mx::Model> {
    ApiRecord::<entity::record_mx::Model> {
      record_type: RecordType::MX,
      record: value.0,
      value: value.1.unwrap(),
    }
  }
}


impl MergeObject<RecordNs, entity::record_ns::Model> for ApiRecord<entity::record_ns::Model> {
  fn merge(
    value: (
      record::Model,
      Option<entity::record_ns::Model>,
    ),
  ) -> ApiRecord<entity::record_ns::Model> {
    ApiRecord::<entity::record_ns::Model> {
      record_type: RecordType::NS,
      record: value.0,
      value: value.1.unwrap(),
    }
  }
}

impl MergeObject<RecordTxt, entity::record_txt::Model> for ApiRecord<entity::record_txt::Model> {
  fn merge(
    value: (
      record::Model,
      Option<entity::record_txt::Model>,
    ),
  ) -> ApiRecord<entity::record_txt::Model> {
    ApiRecord::<entity::record_txt::Model> {
      record_type: RecordType::NS,
      record: value.0,
      value: value.1.unwrap(),
    }
  }
}

// RecordA
pub(crate) async fn list_record<E, M>(
  State(mut state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
) -> Result<Json<Arc<Vec<ApiRecord<M>>>>, StatusCode>
where
    E: EntityTrait<Model = M>,
    record::Entity: Related<E>,
    ApiRecord<M> : MergeObject<E, M>
{
  match state.record_service.all::<E, _>(zone_id).await {
    Ok(values) => Ok(Json(Arc::new(values.into_iter().map(ApiRecord::<M>::merge).collect()))),
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

/*
pub(crate) async fn get_record<A: MergeObject<A> + Related<Record> + EntityTrait>(
  State(state): State<ChefState>,
  Path(zone_id): Path<Uuid>,
  Path(record_id): Path<Uuid>,
) -> Result<Json<Arc<ApiRecord<A>>>, StatusCode> {
  /*match state.record_service.find::<A>(zone_id, record_id).await {
    Ok(values) => {
      let collection: Vec<ApiRecord<RecordA>> = values.into_iter().map(A::merge).collect();
      Ok(Json(Arc::new(collection)))
    }
    Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
  }*.

   */
  Err(StatusCode::INTERNAL_SERVER_ERROR)
}
*/