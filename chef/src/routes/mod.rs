use axum::routing::get;
use axum::Router;
use tracing::error;

use entity::prelude::{RecordA, RecordAaaa, RecordCname, RecordMx, RecordNs, RecordTxt};
use entity::{record_a, record_aaaa, record_cname, record_mx, record_ns, record_txt};

use crate::ctx::Context;
use crate::routes::record::{
  create_record, delete_record, get_record, list_records, modify_record,
};
use crate::routes::zone::{create_zone, delete_zone, get_zone, list_zones};

mod model;
mod record;
mod zone;

pub fn record_error(error: anyhow::Error) {
  error!("Error Occurred: {:?}", &error);
}

pub(super) fn router() -> Router<Context> {
  Router::new()
    .route(
      "/api/dns/v1/zone",
      get(list_zones)
        .post(create_zone),
    )
    .route(
      "/api/dns/v1/zone/:zone_id",
      get(get_zone)
        .delete(delete_zone)
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/a",
      get(list_records::<RecordA>)
        .post(create_record::<RecordA, _, _, CreateARecord>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/aaaa",
      get(list_records::<RecordAaaa>)
        .post(create_record::<RecordAaaa, _, _, CreateAAAARecord>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/cname",
      get(list_records::<RecordCname>)
        .post(create_record::<RecordCname, _, _, CreateCnameRecord>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/mx",
      get(list_records::<RecordMx>)
        .post(create_record::<RecordMx, _, _, CreateMxRecord>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/ns",
      get(list_records::<RecordNs>)
        .post(create_record::<RecordNs, _, _, CreateNsRecord>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/txt",
      get(list_recordss::<RecordTxt>)
        .post(create_record::<RecordTxt, _, _, CreateTxtRecord>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/a/:record_id",
      get(get_record::<RecordA>)
        .delete(delete_record::<RecordA>).
        put(modify_record::<RecordA, record_a::Model, record_a::ActiveModel, CreateARecord>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/aaaa/:record_id",
      get(get_record::<RecordAaaa>)
        .delete(delete_record::<RecordAaaa>)
        .put(modify_record::<RecordAaaa, record_aaaa::Model, record_aaaa::ActiveModel, CreateAAAARecord>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/cname/:record_id",
      get(get_record::<RecordCname>)
        .delete(delete_record::<RecordCname>)
        .put(modify_record::<RecordCname, record_cname::Model, record_cname::ActiveModel, CreateCnameRecord>,
        ),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/mx/:record_id",
      get(get_record::<RecordMx>)
        .delete(delete_record::<RecordMx>)
        .put(modify_record::<RecordMx, record_mx::Model, record_mx::ActiveModel, CreateMxRecord>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/ns/:record_id",
      get(get_record::<RecordNs>)
        .delete(delete_record::<RecordNs>).
        put(modify_record::<RecordNs, record_ns::Model, record_ns::ActiveModel, CreateNsRecord>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/txt/:record_id",
      get(get_record::<RecordTxt>)
        .delete(delete_record::<RecordTxt>).
        put(modify_record::<RecordTxt, record_txt::Model, record_txt::ActiveModel, CreateTxtRecord>),
    )
}
