use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use migration::{Migrator, MigratorTrait};
use sea_orm::prelude::Uuid;
use sea_orm::Database;
use tokio::net::{TcpListener, UdpSocket};
use tokio::select;
use tokio::signal::ctrl_c;
use tracing::info;
use trust_dns_server::authority::Catalog;
use trust_dns_server::proto::rr::LowerName;
use trust_dns_server::ServerFuture;

use crate::args::MaidArgs;
use crate::authority::CatalogAuthority;
use crate::service::ZoneService;

mod args;
mod authority;
mod service;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt::init();

  let args = MaidArgs::parse();

  let db = Arc::new(Database::connect(args.database_url).await?);
  Migrator::up(db.as_ref(), None).await?;

  let zone_service = Arc::new(ZoneService::new(db));

  let mut catalog = Catalog::new();
  catalog.upsert(
    LowerName::from_str("dresden.zone.")?,
    Box::new(CatalogAuthority::new(
      zone_service,
      Uuid::from_str("123e4567-e89b-12d3-a456-426614174000")?,
      LowerName::from_str("dresden.zone.").unwrap(),
    )),
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
