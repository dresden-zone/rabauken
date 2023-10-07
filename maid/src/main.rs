use std::str::FromStr;

use async_trait::async_trait;
use tokio::net::UdpSocket;
use trust_dns_server::authority::{
  AuthorityObject, Catalog, LookupError, LookupObject, LookupOptions, MessageRequest, UpdateResult,
  ZoneType,
};
use trust_dns_server::proto::rr::{LowerName, RecordType};
use trust_dns_server::server::RequestInfo;
use trust_dns_server::ServerFuture;

struct Auth {}

#[async_trait]
impl AuthorityObject for Auth {
  fn box_clone(&self) -> Box<dyn AuthorityObject> {
    todo!()
  }

  fn zone_type(&self) -> ZoneType {
    ZoneType::Primary
  }

  fn is_axfr_allowed(&self) -> bool {
    todo!()
  }

  async fn update(&self, update: &MessageRequest) -> UpdateResult<bool> {
    todo!()
  }

  fn origin(&self) -> &LowerName {
    todo!()
  }

  async fn lookup(
    &self,
    name: &LowerName,
    rtype: RecordType,
    lookup_options: LookupOptions,
  ) -> Result<Box<dyn LookupObject>, LookupError> {
    todo!()
  }

  async fn search(
    &self,
    request_info: RequestInfo<'_>,
    lookup_options: LookupOptions,
  ) -> Result<Box<dyn LookupObject>, LookupError> {
    todo!()
  }

  async fn get_nsec_records(
    &self,
    name: &LowerName,
    lookup_options: LookupOptions,
  ) -> Result<Box<dyn LookupObject>, LookupError> {
    todo!()
  }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let mut catalog = Catalog::new();
  catalog.upsert(LowerName::from_str("dresden.zone.")?, Box::new(Auth {}));

  let mut server = ServerFuture::new(catalog);
  server.register_socket(UdpSocket::bind("127.0.0.1:53").await?);

  server.block_until_done().await?;

  Ok(())
}
