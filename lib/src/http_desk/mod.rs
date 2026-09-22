//! HTTP workspace domain modules.
pub mod client;
pub mod import;
pub mod model;
pub mod vars;
#[cfg(feature = "host-adapter")]
pub mod export;
#[cfg(feature = "host-adapter")]
pub mod service;
