use entity::zone;
use sea_orm::{
    EntityTrait,
    ActiveModelTrait,
    ModelTrait,
    DatabaseConnection,
    DeleteResult,
    prelude::Uuid,
    sea_query::{Expr, Value}
};
use tracing_subscriber::fmt::time;
use entity::prelude::Zone;

pub(crate) struct ZoneService<'a>{
    db: &'a DatabaseConnection
}

impl<'a> ZoneService<'a> {
    fn from(db: &'a DatabaseConnection) -> ZoneService {
        ZoneService {
            db
        }
    }

    async fn create(&mut self, zone: zone::ActiveModel) -> anyhow::Result<zone::Model> {
        Ok(zone.insert(self.db).await?)
    }

    async fn delete(&mut self, id: Uuid) -> anyhow::Result<DeleteResult> {
        let zone = Zone::find_by_id(id).one(self.db).await?.unwrap();
        Ok(zone.delete(self.db).await?)
    }

    /*
    async fn update(&mut self, id: Uuid, new_zone: zone::ActiveModel) {
        Zone::update_many()
            .col_expr(entity::zone::Column::Updated, Expr::value(Value::TimeDateTimeWithTimeZone(time::time())))
            .filter(entity::zone::Column::Id.eq(id))
            .exec(self.db)
            .await?;
    }
    */

}