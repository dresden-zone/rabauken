use crate::service::generic_database::GenericDatabaseService;

#[derive(Clone)]
pub(crate) struct ChefState {
  pub database: GenericDatabaseService,
}
