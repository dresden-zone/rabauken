use axum::routing::{delete, get, post, put};
use axum::Router;

pub mod zone;

use crate::state::ChefState;

use crate::routes::zone::{delete_zone, get_zone, list_zones, modify_zone};
use zone::create_zone;

pub(crate) fn route() -> Router<ChefState> {
  Router::new()
    .route("/v1/zone", get(list_zones))
    .route("/v1/zone", post(create_zone))
    .route("/v1/zone/:zone_id", put(modify_zone))
    .route("/v1/zone/:zone_id", delete(delete_zone))
    .route("/v1/zone/:zone_id", get(get_zone))
}
