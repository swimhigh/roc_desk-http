//! HTTP domain logic. Host-specific filesystem and database adapters are
//! intentionally kept behind the `host-adapter` feature during migration.
pub mod client;
pub mod import;
pub mod model;
pub mod vars;

#[cfg(feature = "host-adapter")]
pub mod export;
#[cfg(feature = "host-adapter")]
pub mod service;

pub use roc_desk_core::error::AppError;
