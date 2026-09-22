//! HTTP tool's own SQLite-backed storage: request history and workspace tab
//! metadata. Built on `roc_desk_core::db` (the pool + generic migration
//! runner shared by all tool crates); the schema here is specific to this
//! tool so it lives in this crate, not in `roc_desk_core`.

pub mod repo;

pub use roc_desk_core::db::DbPool;

const MIGRATIONS: &[(&str, &str)] = &[(
    "0001_http_desk",
    include_str!("../../migrations/0001_http_desk.sql"),
)];

/// Opens (creating if needed) the HTTP tool's SQLite database at `db_path`
/// and applies any migrations that haven't run yet.
pub fn open(db_path: &std::path::Path) -> Result<DbPool, roc_desk_core::error::AppError> {
    let pool = roc_desk_core::db::pool::create_pool(db_path)?;
    let conn = pool.get()?;
    roc_desk_core::db::migrate::apply_migrations(&conn, MIGRATIONS)?;
    Ok(pool)
}
