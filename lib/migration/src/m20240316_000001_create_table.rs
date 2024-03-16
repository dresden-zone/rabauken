use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let db = manager.get_connection();

    // TODO: CAA, SRV

    db.execute_unprepared(
      r#"
      create table "user" (
        id           uuid         not null primary key default gen_random_uuid(),
        created      timestamptz  not null             default now(),
        updated      timestamptz  not null             default now(),
        name         varchar(255) not null unique,
        email        varchar(255) not null unique,
        display_name varchar(255) not null
      );

      create table "password" (
        id      uuid         not null primary key references "user" (id),
        created timestamptz  not null default now(),
        updated timestamptz  not null default now(),
        hash    varchar(255) not null
      );
    "#,
    )
    .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .get_connection()
      .execute_unprepared(
        r#"
        DROP TABLE "user";
        DROP TABLE "password";
      "#,
      )
      .await?;

    Ok(())
  }
}
