use trust_dns_proto::rr::{rdata, RData};

use crate::error::EntityError;
use crate::{record_a, record_aaaa, record_cname, record_mx, record_ns, record_txt};

impl TryFrom<record_a::Model> for RData {
  type Error = EntityError;

  fn try_from(value: record_a::Model) -> Result<Self, Self::Error> {
    Ok(Self::A(rdata::A(value.address.parse()?)))
  }
}

impl TryFrom<record_aaaa::Model> for RData {
  type Error = EntityError;

  fn try_from(value: record_aaaa::Model) -> Result<Self, Self::Error> {
    Ok(Self::AAAA(rdata::AAAA(value.address.parse()?)))
  }
}

impl TryFrom<record_cname::Model> for RData {
  type Error = EntityError;

  fn try_from(value: record_cname::Model) -> Result<Self, Self::Error> {
    Ok(Self::CNAME(rdata::CNAME(value.target.parse()?)))
  }
}

impl TryFrom<record_mx::Model> for RData {
  type Error = EntityError;

  fn try_from(value: record_mx::Model) -> Result<Self, Self::Error> {
    Ok(Self::MX(rdata::MX::new(
      value.preference as u16,
      value.exchange.parse()?,
    )))
  }
}

impl TryFrom<record_ns::Model> for RData {
  type Error = EntityError;

  fn try_from(value: record_ns::Model) -> Result<Self, Self::Error> {
    Ok(Self::NS(rdata::NS(value.target.parse()?)))
  }
}

impl TryFrom<record_txt::Model> for RData {
  type Error = EntityError;

  fn try_from(value: record_txt::Model) -> Result<Self, Self::Error> {
    Ok(Self::TXT(rdata::TXT::new(vec![value.content])))
  }
}
