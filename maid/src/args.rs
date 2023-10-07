use std::net::SocketAddr;

use clap::Parser;
use url::Url;

#[derive(Parser)]
#[command(author, version, about, long_about)]
pub(super) struct MaidArgs {
  #[arg(
    long,
    short,
    env = "MAID_LISTEN_ADDR",
    default_value = "127.0.0.1:8080"
  )]
  pub(super) listen_addr: SocketAddr,
  #[arg(long, short, env = "MAID_DATABASE_URL")]
  pub(super) database_url: Url,
}
