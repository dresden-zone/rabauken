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
      create table zone(
          id       uuid primary key,
          created  timestamptz  not null,
          updated  timestamptz  not null,
          name     varchar(255) not null,
          verified boolean      not null,
          ttl      bigint       not null,
          refresh  integer      not null,
          retry    integer      not null,
          expire   integer      not null,
          minimum  bigint       not null
      );

      create table record(
          id      uuid primary key,
          created timestamptz  not null,
          updated timestamptz  not null,
          name    varchar(255) not null,
          zone_id uuid         not null references zone (id),
          ttl     bigint       not null
      );

      create table record_a(
          id      uuid primary key references record(id),
          address varchar(15)  not null
      );

      create table record_aaaa(
          id      uuid primary key references record(id),
          address varchar(41)  not null
      );

      create table record_cname(
          id      uuid primary key references record(id) unique,
          target  varchar(255) not null
      );

      create table record_mx(
          id         uuid primary key references record(id),
          preference smallint     not null,
          exchange   varchar(255) not null
      );

      create table record_ns(
          id       uuid primary key references record(id),
          target   varchar(255) not null
      );

      create table record_txt(
          id       uuid primary key references record(id),
          content  text         not null
      );

      create table "user"(
        id           uuid         not null primary key default gen_random_uuid(),
        created      timestamptz  not null             default now(),
        updated      timestamptz  not null             default now(),
        name         varchar(255) not null unique,
        email        varchar(255) not null unique,
        display_name varchar(255) not null
      );

      create table "password"(
        id      uuid         not null primary key references "user" (id),
        created timestamptz  not null default now(),
        updated timestamptz  not null default now(),
        hash    varchar(255) not null
      );

      create table invite(
        id      uuid          not null primary key,
        expiry  timestamptz   not null,
        created timestamptz   not null,
        email   varchar(255)  not null
      );

      -- TODO:
      -- create index zone_name_index on zone(name);
      -- create index record_name_index on record(name);
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
        DROP TABLE zone;
        DROP TABLE record_a;
        DROP TABLE record_aaaa;
        DROP TABLE record_cname;
        DROP TABLE record_mx;
        DROP TABLE record_ns;
        DROP TABLE record_txt;
        DROP TABLE "user";
        DROP TABLE "password";
      "#,
      )
      .await?;

    Ok(())
  }
}
