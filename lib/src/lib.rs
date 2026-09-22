//! HTTP domain logic. Host-specific filesystem and database adapters remain in
//! the compatibility source until common interfaces are complete.
#[path = "http_desk/mod.rs"]
pub mod http_desk;
pub use roc_desk_core::error::AppError;
pub use http_desk::{client, import, model, vars};
