use axum::routing::{get, post};
use axum::Router;

pub mod zone;

use crate::state::ChefState;

use zone::create_zone;
use crate::routes::zone::list_zones;

pub(crate) fn route() -> Router<ChefState> {
  Router::new()
      .route("/v1/zone", get(list_zones))
      .route("/v1/zone", post(create_zone))
}
