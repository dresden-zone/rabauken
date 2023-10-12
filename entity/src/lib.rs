pub use error::EntityError;
pub use models::*;

#[cfg(feature = "trust-dns-proto")]
mod error;
#[cfg(feature = "trust-dns-proto")]
mod try_from;

mod models;
