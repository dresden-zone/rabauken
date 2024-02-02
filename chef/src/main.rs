use std::future::IntoFuture;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use sea_orm::{ConnectOptions, Database};
use tokio::net::TcpListener;
use tokio::select;
use tokio::signal::ctrl_c;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use migration::{Migrator, MigratorTrait};

use crate::args::ChefArgs;
use crate::routes::routes;
use crate::service::generic_database::GenericDatabaseService;
use crate::service::generic_record_service::GenericRecordService;
use crate::state::ChefState;

mod args;
mod routes;
mod service;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = ChefArgs::parse();

  let subscriber = FmtSubscriber::builder()
    .with_max_level(Level::INFO)
    .compact()
    .finish();

  tracing::subscriber::set_global_default(subscriber)?;

  info!(concat!(
    "Booting ",
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    "..."
  ));

  let mut db_options = ConnectOptions::new(args.database_url);
  db_options
    .max_connections(100)
    .min_connections(5)
    .connect_timeout(Duration::from_secs(8))
    .acquire_timeout(Duration::from_secs(8))
    .idle_timeout(Duration::from_secs(8))
    .max_lifetime(Duration::from_secs(8))
    .sqlx_logging(false);

  let db = Arc::new(Database::connect(db_options).await?);
  Migrator::up(&*db, None).await?;

  // TODO: create services

  let state = ChefState {
    database: GenericDatabaseService::from_db(db.clone()),
    record_service: GenericRecordService::from_db(db.clone()),
  };

  let router = routes().with_state(state);
  let listener = TcpListener::bind(&args.listen_addr).await?;

  info!("Listening on http://{}...", args.listen_addr);

  select! {
    result = axum::serve(listener, router).into_future() => {
      result?;
      info!("Socket closed, quitting...");
    },
    result = shutdown_signal() => {
      result?;
      info!("Termination signal received, quitting...");
    }
  }

  Ok(())
}

async fn shutdown_signal() -> anyhow::Result<()> {
  let ctrl_c = async { ctrl_c().await.expect("failed to install Ctrl+C handler") };

  #[cfg(unix)]
  {
    let terminate = async {
      tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("failed to install signal handler")
        .recv()
        .await;
    };

    select! {
      _ = ctrl_c => {},
      _ = terminate => {},
    }

    Ok(())
  }

  #[cfg(not(unix))]
  {
    ctrl_c.await;
    Ok(())
  }
}
