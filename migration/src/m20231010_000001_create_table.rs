use crate::async_trait::async_trait;
use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let db = manager.get_connection();

    // Use `execute_unprepared` if the SQL statement doesn't have value bindings
    db.execute_unprepared(
      "create table zone (
                id      uuid primary key,
                created timestamptz  not null default current_timestamp,
                admin   varchar(255) not null,
                name    varchar(255) not null,
                refresh integer      not null,
                retry   integer      not null,
                expire  integer      not null,
                minimum integer      not null
            );",
    )
    .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .get_connection()
      .execute_unprepared("DROP TABLE `zone`")
      .await?;

    Ok(())
  }
}
