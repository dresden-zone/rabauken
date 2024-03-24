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
      create table "user"(
        id             uuid         not null primary key default gen_random_uuid(),
        created        timestamptz  not null             default now(),
        updated        timestamptz  not null             default now(),
        name           varchar(255) not null unique,
        email          varchar(255) not null unique,
        email_verified bool         not null,
        display_name   varchar(255) not null,
        roles          int2         not null
      );

      create table "password"(
        id      uuid         not null primary key references "user" (id),
        created timestamptz  not null default now(),
        updated timestamptz  not null default now(),
        hash    varchar(255) not null
      );

      create table invite(
        id      uuid         not null primary key default gen_random_uuid(),
        created timestamptz  not null             default now(),
        expire  timestamptz  not null,
        email   varchar(255) not null,
        roles   int2         not null
      );

      create table zone(
        id       uuid         not null primary key default gen_random_uuid(),
        created  timestamptz  not null             default now(),
        updated  timestamptz  not null             default now(),
        name     varchar(255) not null,
        owner    uuid         not null references "user" (id),
        verified boolean      not null             default false,
        serial   int8         not null             default 0,
        unique (name, owner)
      );

      create table record(
        id      uuid         not null primary key default gen_random_uuid(),
        created timestamptz  not null             default now(),
        updated timestamptz  not null             default now(),
        name    varchar(255) not null,
        zone_id uuid         not null references zone (id),
        ttl     int4
      );

      create table record_a(
          id   uuid         not null primary key references record(id),
          addr varchar(15)  not null
      );

      create table record_aaaa(
          id   uuid         not null primary key references record(id),
          addr varchar(41)  not null
      );

      create table record_cname(
          id      uuid         not null primary key references record(id),
          target  varchar(255) not null
      );

      create table record_mx(
          id         uuid         not null primary key references record(id),
          preference int4         not null,
          exchange   varchar(255) not null
      );

      create table record_ns(
          id       uuid         not null primary key references record(id),
          target   varchar(255) not null
      );

      create table record_txt(
          id       uuid not null primary key references record(id),
          content  text not null
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
        DROP TABLE "invite";
      "#,
      )
      .await?;

    Ok(())
  }
}
