use std::sync::Arc;

use axum::Router;
use axum::Server;
use clap::Parser;
use sea_orm::Database;
use tokio::select;
use tokio::signal::ctrl_c;
use tracing::info;

use migration::{Migrator, MigratorTrait};

use crate::args::ChefArgs;
use crate::state::State;

mod args;
mod service;
mod state;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  tracing_subscriber::fmt::init();

  let args = ChefArgs::parse();

  let db = Database::connect(args.database_url).await?;
  Migrator::up(&db, None).await?;

  // TODO: create services

  let state = State {};

  let router = Router::new().with_state(Arc::new(state));

  let server = Server::bind(&args.listen_addr).serve(router.into_make_service());

  info!("Listening on http://{}...", args.listen_addr);

  select! {
   result = server => {
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
