use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::routes::zone::list_zones; //, create_zone};//delete_zone, get_zone, , modify_zone};
use crate::state::ChefState;

mod zone;

pub(super) fn routes() -> Router<ChefState> {
  Router::new().route("/v1/zone", get(list_zones))
  //.route("/v1/zone", post(create_zone))
  /* .route("/v1/zone/:zone_id", put(modify_zone))
  .route("/v1/zone/:zone_id", delete(delete_zone))
  .route("/v1/zone/:zone_id", get(get_zone))*/
}
