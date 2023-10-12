use std::net::AddrParseError;
use thiserror::Error;

use trust_dns_proto::error::ProtoError;

#[derive(Error, Debug)]
pub enum EntityError {
  #[error("Unable to parse addr")]
  AddrParseError(
    #[from]
    #[source]
    AddrParseError,
  ),
  #[error("Unable to parse name")]
  ProtoError(
    #[from]
    #[source]
    ProtoError,
  ),
}
