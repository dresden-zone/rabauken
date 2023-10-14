use crate::routes::zone::{create_zone, delete_zone, get_zone, list_zones, modify_zone};
use crate::service::model::{
  CreateAAAARecord, CreateARecord, CreateCnameRecord, CreateMxRecord, CreateNsRecord,
  CreateTxtRecord,
};
use crate::state::ChefState;
use axum::routing::{delete, get, post, put};
use axum::Router;
use tracing::error;

mod model;
mod record;
mod zone;

use crate::routes::record::{create_record, get_record, list_record};
use entity::prelude::{RecordA, RecordAaaa, RecordCname, RecordMx, RecordNs, RecordTxt};

pub fn record_error(error: anyhow::Error) {
  error!("Error Occurred: {:?}", &error);
}

pub(super) fn routes() -> Router<ChefState> {
  Router::new()
    .route("/v1/zone", get(list_zones))
    .route("/v1/zone", post(create_zone))
    .route("/v1/zone/:zone_id", put(modify_zone))
    .route("/v1/zone/:zone_id", delete(delete_zone))
    .route("/v1/zone/:zone_id", get(get_zone))
    .route(
      "/v1/zone/:zone_id/record/a",
      get(list_record::<RecordA, entity::record_a::Model>),
    )
    .route(
      "/v1/zone/:zone_id/record/aaaa",
      get(list_record::<RecordAaaa, entity::record_aaaa::Model>),
    )
    .route(
      "/v1/zone/:zone_id/record/cname",
      get(list_record::<RecordCname, entity::record_cname::Model>),
    )
    .route(
      "/v1/zone/:zone_id/record/mx",
      get(list_record::<RecordMx, entity::record_mx::Model>),
    )
    .route(
      "/v1/zone/:zone_id/record/ns",
      get(list_record::<RecordNs, entity::record_ns::Model>),
    )
    .route(
      "/v1/zone/:zone_id/record/txt",
      get(list_record::<RecordTxt, entity::record_txt::Model>),
    )
    .route(
      "/v1/zone/:zone_id/record/a/:record_id",
      get(get_record::<RecordA, entity::record_a::Model>),
    )
    .route(
      "/v1/zone/:zone_id/record/aaaa/:record_id",
      get(get_record::<RecordAaaa, entity::record_aaaa::Model>),
    )
    .route(
      "/v1/zone/:zone_id/record/cname/:record_id",
      get(get_record::<RecordCname, entity::record_cname::Model>),
    )
    .route(
      "/v1/zone/:zone_id/record/mx/:record_id",
      get(get_record::<RecordMx, entity::record_mx::Model>),
    )
    .route(
      "/v1/zone/:zone_id/record/ns/:record_id",
      get(get_record::<RecordNs, entity::record_ns::Model>),
    )
    .route(
      "/v1/zone/:zone_id/record/txt/:record_id",
      get(get_record::<RecordTxt, entity::record_txt::Model>),
    )
    .route(
      "/v1/zone/:zone_id/record/a/",
      post(create_record::<RecordA, _, _, CreateARecord>),
    )
    .route(
      "/v1/zone/:zone_id/record/aaaa/",
      post(create_record::<RecordAaaa, _, _, CreateAAAARecord>),
    )
    .route(
      "/v1/zone/:zone_id/record/cname/",
      post(create_record::<RecordCname, _, _, CreateCnameRecord>),
    )
    .route(
      "/v1/zone/:zone_id/record/mx/",
      post(create_record::<RecordMx, _, _, CreateMxRecord>),
    )
    .route(
      "/v1/zone/:zone_id/record/ns/",
      post(create_record::<RecordNs, _, _, CreateNsRecord>),
    )
    .route(
      "/v1/zone/:zone_id/record/txt/",
      post(create_record::<RecordTxt, _, _, CreateTxtRecord>),
    )
}
