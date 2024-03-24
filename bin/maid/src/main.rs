use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use hickory_server::authority::Catalog;
use hickory_server::proto::rr::LowerName;
use hickory_server::ServerFuture;
use sea_orm::prelude::Uuid;
use sea_orm::{ConnectOptions, Database};
use tokio::net::{TcpListener, UdpSocket};
use tokio::select;
use tokio::signal::ctrl_c;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use migration::{Migrator, MigratorTrait};

use crate::args::MaidArgs;
use crate::authority::ZoneAuthority;
use crate::service::ZoneService;

mod args;
mod authority;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let args = MaidArgs::parse();

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
  Migrator::up(db.as_ref(), None).await?;

  let zone_service = Arc::new(ZoneService::new(db));

  let mut catalog = Catalog::new();
  let name = LowerName::from_str("dresden.zone.")?;
  catalog.upsert(
    name.clone(),
    Box::new(Arc::new(ZoneAuthority::new(
      zone_service,
      Uuid::from_str("2067530c-c9b0-4105-8f59-692593a1095d")?,
      name,
    ))),
    // Box::new(Arc::new(FileAuthority::try_from_config(Name::from(name) ,ZoneType::Primary, false, None, &FileConfig {zone_file_path: "dresden.zone.db".to_string()}).unwrap())),
  );

  let mut server = ServerFuture::new(catalog);
  server.register_socket(UdpSocket::bind(args.listen_addr).await?);
  server.register_listener(
    TcpListener::bind(args.listen_addr).await?,
    Duration::from_secs(30),
  );

  info!("Listening on udp://{}...", args.listen_addr);
  info!("Listening on tcp://{}...", args.listen_addr);

  select! {
   result = server.block_until_done() => {
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
