use crate::async_trait::async_trait;
use sea_orm::Statement;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    let db = manager.get_connection();

    // TODO: CAA, SRV

    db.execute_unprepared(
      "
      create table zone
      (
          id      uuid primary key,
          created timestamptz  not null,
          updated timestamptz  not null,
          admin   varchar(255) not null,
          name    varchar(255) not null,
          refresh integer      not null,
          retry   integer      not null,
          expire  integer      not null,
          minimum integer      not null
      );

      create table record
      (
          id      uuid primary key,
          created timestamptz  not null,
          updated timestamptz  not null,
          name    varchar(255) not null,
          zone_id uuid         not null references zone (id),
          ttl     integer      not null
      );

      create table record_a
      (
          id      uuid primary key references record(id),
          address varchar(15)  not null
      );

      create table record_aaaa
      (
          id      uuid primary key references record(id),
          address varchar(41)  not null
      );

      create table record_cname
      (
          id      uuid primary key references record(id),
          target  varchar(255) not null
      );

      create table record_mx
      (
          id       uuid primary key references record(id),
          target   varchar(255) not null,
          priority smallint     not null
      );

      create table record_txt
      (
          id       uuid primary key references record(id),
          content  text         not null
      );
    ",
    )
    .await?;

    Ok(())
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
      .get_connection()
      .execute_unprepared(
        "
        DROP TABLE zone;
        DROP TABLE record_a;
        DROP TABLE record_aaaa;
        DROP TABLE record_cname;
        DROP TABLE record_mx;
        DROP TABLE record_txt;
      ",
      )
      .await?;

    Ok(())
  }
}
