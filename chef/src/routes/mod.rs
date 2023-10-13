use std::sync::Arc;
use axum::extract::{Path, State};
use axum::routing::{delete, get, post, put};
use axum::{debug_handler, Json, Router};
use axum::http::StatusCode;
use tracing::error;
use uuid::Uuid;

use crate::routes::zone::{create_zone, delete_zone, get_zone, list_zones, modify_zone};
use crate::state::ChefState;

mod record;
mod zone;

use crate::routes::record::{ApiRecord, list_record};
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
        .route("/v1/zone/:zone_id/record/a", get(list_record::<RecordA, entity::record_a::Model>))
        .route("/v1/zone/:zone_id/record/aaaa", get(list_record::<RecordAaaa, entity::record_aaaa::Model>))
        .route("/v1/zone/:zone_id/record/cname", get(list_record::<RecordCname, entity::record_cname::Model>))
        .route("/v1/zone/:zone_id/record/mx", get(list_record::<RecordMx, entity::record_mx::Model>))
        .route("/v1/zone/:zone_id/record/ns", get(list_record::<RecordNs, entity::record_ns::Model>))
        .route("/v1/zone/:zone_id/record/txt", get(list_record::<RecordTxt, entity::record_txt::Model>))
}

