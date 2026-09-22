use rusqlite::{params, OptionalExtension};

use roc_desk_core::db::DbPool;
use roc_desk_core::error::AppError;

use crate::http_desk::model::HttpWorkspaceTab;

pub struct HttpTabsRepo {
    pool: DbPool,
}

impl HttpTabsRepo {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn create(&self, tab: &HttpWorkspaceTab) -> Result<(), AppError> {
        let conn = self.pool.get()?;
        conn.execute(
            "INSERT INTO http_tabs
                (id, root, collection_slug, request_id, title, sort_order, updated_at)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                tab.id,
                tab.root,
                tab.collection_slug,
                tab.request_id,
                tab.title,
                tab.sort_order,
                tab.updated_at,
            ],
        )?;
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let conn = self.pool.get()?;
        conn.execute("DELETE FROM http_tabs WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn delete_by_request(&self, root: &str, request_id: &str) -> Result<(), AppError> {
        let conn = self.pool.get()?;
        conn.execute(
            "DELETE FROM http_tabs WHERE root = ?1 AND request_id = ?2",
            params![root, request_id],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get(&self, id: &str) -> Result<Option<HttpWorkspaceTab>, AppError> {
        let conn = self.pool.get()?;
        conn.query_row(
            "SELECT id, root, collection_slug, request_id, title, sort_order, updated_at
             FROM http_tabs WHERE id = ?1",
            params![id],
            Self::map_row,
        )
        .optional()
        .map_err(AppError::from)
    }

    pub fn list_for_root(&self, root: &str) -> Result<Vec<HttpWorkspaceTab>, AppError> {
        let conn = self.pool.get()?;
        let mut stmt = conn.prepare(
            "SELECT id, root, collection_slug, request_id, title, sort_order, updated_at
             FROM http_tabs WHERE root = ?1 ORDER BY sort_order",
        )?;
        let rows = stmt
            .query_map(params![root], Self::map_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    fn map_row(row: &rusqlite::Row) -> rusqlite::Result<HttpWorkspaceTab> {
        Ok(HttpWorkspaceTab {
            id: row.get(0)?,
            root: row.get(1)?,
            collection_slug: row.get(2)?,
            request_id: row.get(3)?,
            title: row.get(4)?,
            sort_order: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }
}
