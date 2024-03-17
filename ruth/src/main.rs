use bb8_redis::bb8::Pool;
use bb8_redis::RedisConnectionManager;
use std::future::IntoFuture;
use std::sync::Arc;

use clap::Parser;
use sea_orm::Database;
use tokio::net::TcpListener;
use tokio::select;
use tower_http::trace::TraceLayer;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use url::Url;

use migration::{Migrator, MigratorTrait};
use session::SessionStore;

use crate::args::Args;
use crate::ctx::Context;
use crate::routes::router;
use crate::service::UserService;
use crate::utils::shutdown_signal;

mod args;
mod ctx;
mod routes;
mod service;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = Args::parse();

  let subscriber = FmtSubscriber::builder()
    .with_max_level(Level::DEBUG)
    .compact()
    .finish();

  tracing::subscriber::set_global_default(subscriber)?;

  info!(concat!(
    "booting ",
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    "..."
  ));

  let mut db_url = Url::parse("postgres://a:b@c/d").unwrap();
  db_url.set_username(&args.db_user).unwrap();
  db_url
    .set_password(Some(&match args.db_pass {
      None => tokio::fs::read_to_string(args.db_pass_path.unwrap()).await?,
      Some(pass) => pass,
    }))
    .unwrap();
  db_url.set_ip_host(args.db_addr).unwrap();
  db_url.set_port(Some(args.db_port)).unwrap();
  db_url.set_path(&args.db_name);

  let db = Database::connect(db_url.as_str()).await?;

  info!("connected to db");

  Migrator::up(&db, None).await?;

  let listener = TcpListener::bind(args.listen_addr).await?;
  info!("listening at http://{}...", args.listen_addr);

  let redis_pool = {
    let manager = RedisConnectionManager::new((args.redis_addr.to_string(), args.redis_port))?;
    Pool::builder().build(manager).await?
  };

  let user_service = Arc::new(UserService::new(db));
  let session_store = SessionStore::new(redis_pool);

  let router = router()
    .with_state(Context {
      user_service,
      session_store,
    })
    .layer(TraceLayer::new_for_http())
    .into_make_service();

  let axum = axum::serve(listener, router)
    .with_graceful_shutdown(shutdown_signal())
    .into_future();

  select! {
    result = axum => { result? }
  }

  Ok(())
}
