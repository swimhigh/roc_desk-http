//! HTTP domain logic.
#[path = "http_desk/mod.rs"]
pub mod http_desk;
pub mod error { pub use roc_desk_core::error::AppError; }
pub use roc_desk_core::error::AppError;
pub use http_desk::{client, import, model, vars};
