use crate::service::zone::ZoneService;

#[derive(Clone)]
pub(crate) struct ChefState {
  pub zone_service: ZoneService,
}
