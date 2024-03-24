use axum::routing::get;
use axum::Router;

use entity::prelude::{RecordA, RecordAaaa, RecordCname, RecordMx, RecordNs, RecordTxt};

use crate::ctx::Context;
use crate::routes::record::{
  create_record, delete_record, get_record, list_records, modify_record,
};
use crate::routes::zone::{create_zone, delete_zone, get_zone, list_zones};
use crate::service::{
  RecordARequest, RecordAaaaRequest, RecordCnameRequest, RecordMxRequest, RecordNsRequest,
  RecordTxtRequest,
};

mod record;
mod zone;

pub(super) fn router() -> Router<Context> {
  Router::new()
    .route("/api/dns/v1/zone", get(list_zones).post(create_zone))
    .route(
      "/api/dns/v1/zone/:zone_id",
      get(get_zone).delete(delete_zone),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/a",
      get(list_records::<RecordA>).post(create_record::<RecordARequest, _>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/aaaa",
      get(list_records::<RecordAaaa>).post(create_record::<RecordAaaaRequest, _>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/cname",
      get(list_records::<RecordCname>).post(create_record::<RecordCnameRequest, _>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/mx",
      get(list_records::<RecordMx>).post(create_record::<RecordMxRequest, _>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/ns",
      get(list_records::<RecordNs>).post(create_record::<RecordNsRequest, _>),
    )
    .route(
      "/api/dns/v1/zone/:zone_id/record/txt",
      get(list_records::<RecordTxt>).post(create_record::<RecordTxtRequest, _>),
    )
    .route(
      "/api/dns/v1/record/a/:record_id",
      get(get_record::<RecordA>)
        .delete(delete_record::<RecordA>)
        .put(modify_record::<RecordARequest, _>),
    )
    .route(
      "/api/dns/v1/record/aaaa/:record_id",
      get(get_record::<RecordAaaa>)
        .delete(delete_record::<RecordAaaa>)
        .put(modify_record::<RecordAaaaRequest, _>),
    )
    .route(
      "/api/dns/v1/record/cname/:record_id",
      get(get_record::<RecordCname>)
        .delete(delete_record::<RecordCname>)
        .put(modify_record::<RecordCnameRequest, _>),
    )
    .route(
      "/api/dns/v1/record/mx/:record_id",
      get(get_record::<RecordMx>)
        .delete(delete_record::<RecordMx>)
        .put(modify_record::<RecordMxRequest, _>),
    )
    .route(
      "/api/dns/v1/record/ns/:record_id",
      get(get_record::<RecordNs>)
        .delete(delete_record::<RecordNs>)
        .put(modify_record::<RecordNsRequest, _>),
    )
    .route(
      "/api/dns/v1/record/txt/:record_id",
      get(get_record::<RecordTxt>)
        .delete(delete_record::<RecordTxt>)
        .put(modify_record::<RecordTxtRequest, _>),
    )
}
