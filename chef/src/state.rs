use crate::service::generic_database::GenericDatabaseService;
use crate::service::generic_record_service::GenericRecordService;

#[derive(Clone)]
pub(crate) struct ChefState {
  pub database: GenericDatabaseService,
  pub record_service: GenericRecordService,
}
