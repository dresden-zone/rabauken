use std::net::{Ipv4Addr, Ipv6Addr};

use sea_orm::{ActiveModelTrait, ActiveValue};
use serde::Deserialize;
use uuid::Uuid;

use entity::{record_a, record_aaaa, record_cname, record_mx, record_ns, record_txt};

pub(crate) trait RecordRequestTrait<A: ActiveModelTrait> {
  fn into_active_model(self, id: ActiveValue<Uuid>) -> A;
}

#[derive(Deserialize)]
pub(crate) struct RecordARequest {
  addr: Ipv4Addr,
}

#[derive(Deserialize)]
pub(crate) struct RecordAaaaRequest {
  addr: Ipv6Addr,
}

#[derive(Deserialize)]
pub(crate) struct RecordCnameRequest {
  target: String,
}

#[derive(Deserialize)]
pub(crate) struct RecordMxRequest {
  preference: u16,
  exchange: String,
}

#[derive(Deserialize)]
pub(crate) struct RecordNsRequest {
  target: String,
}

#[derive(Deserialize)]
pub(crate) struct RecordTxtRequest {
  content: String,
}

impl RecordRequestTrait<record_a::ActiveModel> for RecordARequest {
  fn into_active_model(self, id: ActiveValue<Uuid>) -> record_a::ActiveModel {
    record_a::ActiveModel {
      id,
      addr: ActiveValue::Set(self.addr.to_string()),
    }
  }
}

impl RecordRequestTrait<record_aaaa::ActiveModel> for RecordAaaaRequest {
  fn into_active_model(self, id: ActiveValue<Uuid>) -> record_aaaa::ActiveModel {
    record_aaaa::ActiveModel {
      id,
      addr: ActiveValue::Set(self.addr.to_string()),
    }
  }
}

impl RecordRequestTrait<record_cname::ActiveModel> for RecordCnameRequest {
  fn into_active_model(self, id: ActiveValue<Uuid>) -> record_cname::ActiveModel {
    record_cname::ActiveModel {
      id,
      target: ActiveValue::Set(self.target),
    }
  }
}

impl RecordRequestTrait<record_mx::ActiveModel> for RecordMxRequest {
  fn into_active_model(self, id: ActiveValue<Uuid>) -> record_mx::ActiveModel {
    record_mx::ActiveModel {
      id,
      preference: ActiveValue::Set(self.preference as i32),
      exchange: ActiveValue::Set(self.exchange),
    }
  }
}

impl RecordRequestTrait<record_ns::ActiveModel> for RecordNsRequest {
  fn into_active_model(self, id: ActiveValue<Uuid>) -> record_ns::ActiveModel {
    record_ns::ActiveModel {
      id,
      target: ActiveValue::Set(self.target),
    }
  }
}

impl RecordRequestTrait<record_txt::ActiveModel> for RecordTxtRequest {
  fn into_active_model(self, id: ActiveValue<Uuid>) -> record_txt::ActiveModel {
    record_txt::ActiveModel {
      id,
      content: ActiveValue::Set(self.content),
    }
  }
}
