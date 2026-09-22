//! HTTP 测试工作台：集合/环境/请求的本地 YAML 存储 + 发送请求 + 请求历史/标签页。
//!
//! ## 和宿主 `roc_desk` 的 `commands/http_desk.rs` 的关键差异
//!
//! 宿主版本的每个 command 都先用 `workspace_id: Uuid` 去 `AppState.workspaces`
//! （"已打开工作区注册表"）里找 `WorkspaceHandle`（里面才有 `file_ops`/`root_path`），
//! 这是宿主专属的、还没抽成公共库能力的概念（跟工作区、多种 `FileOps` 实现——本地/
//! SSH/Windows Agent——绑定）。这个独立工具没有"打开的工作区"这回事，用户直接指定
//! 一个本地目录作为"HTTP 集合根目录"就够了，所以这里把 28 个命令的 `workspace_id`
//! 参数全部换成 `root: String`（本地目录路径），直接用
//! `roc_desk_common::fsops::LocalFileOps` 作为 `file_ops`，不再需要
//! `get_handle`/`WorkspaceHandle` 这套机制——这和 `roc_desk-editor` 处理
//! `editor_symbols_*` 命令（`workspace_id: Uuid` → `root: String`）用的是同一个思路，
//! 详见 `roc_desk-editor/lib/src/symbols.rs` 顶部注释。
//!
//! 请求历史/标签页不再依赖宿主的数据库连接，这个 crate 自己开一个 SQLite 文件
//! （[`db`] 模块，建立在 `roc_desk_core::db::{DbPool, create_pool, apply_migrations}`
//! 之上，参照 `roc_desk-sql` 的 `db::open` 先例）。

#[path = "http_desk/mod.rs"]
pub mod http_desk;
pub mod error {
    pub use roc_desk_core::error::AppError;
}
pub use roc_desk_core::error::AppError;
pub use http_desk::{client, import, model, vars};

#[cfg(feature = "business")]
pub mod db;

pub use roc_desk_common::fsops::{FileOps, LocalFileOps};

pub const TOOL_NAME: &str = "roc_desk-http";
pub const TOOL_DESCRIPTION: &str = "HTTP 测试工作台：集合、环境与请求";

/// Returns the user-visible metadata used by the standalone shell and host launcher.
pub fn tool_info() -> (&'static str, &'static str) {
    (TOOL_NAME, TOOL_DESCRIPTION)
}

/// Shared state backing the request-history/tabs commands: two SQLite-backed
/// repos over one connection pool. Constructed once at startup (see
/// [`HttpAppState::new`]) and registered with `tauri::Builder::manage`.
/// Collection/request/environment CRUD does NOT go through this state -- it
/// reads/writes YAML files directly via `LocalFileOps` and takes `root` as a
/// plain command argument, so it needs no shared state at all.
#[cfg(feature = "business")]
pub struct HttpAppState {
    pub history: std::sync::Arc<db::repo::http_request_history_repo::HttpRequestHistoryRepo>,
    pub tabs: std::sync::Arc<db::repo::http_tabs_repo::HttpTabsRepo>,
}

#[cfg(feature = "business")]
impl HttpAppState {
    /// `db_path` is this tool's own SQLite file (request history + tab
    /// metadata); it has nothing to do with any particular `root`.
    pub fn new(db_path: &std::path::Path) -> Result<Self, AppError> {
        let pool = db::open(db_path)?;
        Ok(Self {
            history: std::sync::Arc::new(
                db::repo::http_request_history_repo::HttpRequestHistoryRepo::new(pool.clone()),
            ),
            tabs: std::sync::Arc::new(db::repo::http_tabs_repo::HttpTabsRepo::new(pool)),
        })
    }
}

/// The `#[tauri::command]` functions must live in a submodule, not at the
/// crate root -- Tauri's command macro emits a `#[macro_export]` macro_rules
/// item *and* a self-referential `pub use` of the same name in the enclosing
/// scope, which collide at crate root (`E0255`). `standalone/src/main.rs`
/// and the host reference these as `roc_desk_http::cmd::http_list_collections`, etc.
#[cfg(feature = "business")]
pub mod cmd {
    use chrono::Utc;
    use tauri::State;
    use uuid::Uuid;

    use crate::db::repo::http_request_history_repo::NewHistoryEntry;
    use crate::error::AppError;
    use crate::http_desk::model::*;
    use crate::http_desk::vars::VarContext;
    use crate::http_desk::{client, export, import, service};
    use crate::{HttpAppState, LocalFileOps};

    async fn build_var_context_owned(
        root: &str,
        slug: &str,
        environment_id: Option<&str>,
    ) -> Result<(Vec<EnvVar>, Vec<EnvVar>, Vec<EnvVar>), AppError> {
        let environment = match environment_id {
            Some(id) if !id.is_empty() => service::list_environments(&LocalFileOps, root, slug)
                .await?
                .into_iter()
                .find(|e| e.id == id)
                .map(|e| e.variables)
                .unwrap_or_default(),
            _ => Vec::new(),
        };
        let collection = service::collection_variables(&LocalFileOps, root, slug).await?;
        let global = service::get_global_variables(&LocalFileOps, root).await?;
        Ok((environment, collection, global))
    }

    fn redact(text: &str, secrets: &[String]) -> String {
        let mut out = text.to_string();
        for s in secrets {
            if !s.is_empty() {
                out = out.replace(s.as_str(), "***");
            }
        }
        out
    }

    // ---------------------------------------------------------------------
    // 集合
    // ---------------------------------------------------------------------

    #[tauri::command]
    pub async fn http_list_collections(root: String) -> Result<Vec<HttpCollectionSummary>, AppError> {
        service::list_collections(&LocalFileOps, &root).await
    }

    #[tauri::command]
    pub async fn http_create_collection(root: String, name: String) -> Result<HttpCollectionSummary, AppError> {
        service::create_collection(&LocalFileOps, &root, &name).await
    }

    #[tauri::command]
    pub async fn http_rename_collection(root: String, slug: String, name: String) -> Result<(), AppError> {
        service::rename_collection(&LocalFileOps, &root, &slug, &name).await
    }

    #[tauri::command]
    pub async fn http_delete_collection(root: String, slug: String) -> Result<(), AppError> {
        service::delete_collection(&LocalFileOps, &root, &slug).await
    }

    // ---------------------------------------------------------------------
    // 请求
    // ---------------------------------------------------------------------

    #[tauri::command]
    pub async fn http_list_requests(root: String, slug: String) -> Result<Vec<RequestSummary>, AppError> {
        service::list_requests(&LocalFileOps, &root, &slug).await
    }

    #[tauri::command]
    pub async fn http_get_request(root: String, slug: String, id: String) -> Result<RequestDef, AppError> {
        service::get_request(&LocalFileOps, &root, &slug, &id).await
    }

    #[tauri::command]
    pub async fn http_create_request(
        root: String,
        slug: String,
        name: String,
        folder: Vec<String>,
    ) -> Result<RequestDef, AppError> {
        service::create_request(&LocalFileOps, &root, &slug, &name, folder).await
    }

    #[tauri::command]
    pub async fn http_save_request(root: String, slug: String, request: RequestDef) -> Result<(), AppError> {
        service::save_request(&LocalFileOps, &root, &slug, &request).await
    }

    #[tauri::command]
    pub async fn http_delete_request(
        state: State<'_, HttpAppState>,
        root: String,
        slug: String,
        id: String,
    ) -> Result<(), AppError> {
        service::delete_request(&LocalFileOps, &root, &slug, &id).await?;
        state.tabs.delete_by_request(&root, &id)?;
        Ok(())
    }

    // ---------------------------------------------------------------------
    // 环境 / 变量
    // ---------------------------------------------------------------------

    #[tauri::command]
    pub async fn http_list_environments(root: String, slug: String) -> Result<Vec<EnvironmentDef>, AppError> {
        service::list_environments(&LocalFileOps, &root, &slug).await
    }

    #[tauri::command]
    pub async fn http_save_environment(
        root: String,
        slug: String,
        environment: EnvironmentDef,
    ) -> Result<EnvironmentDef, AppError> {
        service::save_environment(&LocalFileOps, &root, &slug, environment).await
    }

    #[tauri::command]
    pub async fn http_delete_environment(root: String, slug: String, env_id: String) -> Result<(), AppError> {
        service::delete_environment(&LocalFileOps, &root, &slug, &env_id).await
    }

    #[tauri::command]
    pub async fn http_get_global_variables(root: String) -> Result<Vec<EnvVar>, AppError> {
        service::get_global_variables(&LocalFileOps, &root).await
    }

    #[tauri::command]
    pub async fn http_save_global_variables(root: String, variables: Vec<EnvVar>) -> Result<(), AppError> {
        service::save_global_variables(&LocalFileOps, &root, &variables).await
    }

    #[tauri::command]
    pub async fn http_get_collection_meta(root: String, slug: String) -> Result<HttpCollectionMeta, AppError> {
        service::get_collection_meta(&LocalFileOps, &root, &slug).await
    }

    #[tauri::command]
    pub async fn http_save_collection_meta(
        root: String,
        slug: String,
        meta: HttpCollectionMeta,
    ) -> Result<(), AppError> {
        service::save_collection_meta(&LocalFileOps, &root, &slug, &meta).await
    }

    // ---------------------------------------------------------------------
    // 发送请求 + 历史
    // ---------------------------------------------------------------------

    #[tauri::command]
    pub async fn http_send_request(
        state: State<'_, HttpAppState>,
        root: String,
        slug: String,
        request: RequestDef,
        environment_id: Option<String>,
    ) -> Result<HttpExecuteResult, AppError> {
        let (environment, collection, global) =
            build_var_context_owned(&root, &slug, environment_id.as_deref()).await?;
        let secrets: Vec<String> = environment
            .iter()
            .chain(collection.iter())
            .chain(global.iter())
            .filter(|v| v.secret && !v.value.is_empty())
            .map(|v| v.value.clone())
            .collect();
        let ctx = VarContext {
            environment: &environment,
            collection: &collection,
            global: &global,
        };

        let outcome = client::execute(&request, &ctx).await;
        let created_at = Utc::now().to_rfc3339();
        let history_id = Uuid::new_v4().to_string();
        let request_snapshot = serde_json::to_string(&request).unwrap_or_default();

        match &outcome {
            Ok(result) => {
                let redacted = HttpExecuteResult {
                    resolved_url: redact(&result.resolved_url, &secrets),
                    body: redact(&result.body, &secrets),
                    headers: result
                        .headers
                        .iter()
                        .map(|(k, v)| (k.clone(), redact(v, &secrets)))
                        .collect(),
                    ..result.clone()
                };
                let response_snapshot = serde_json::to_string(&redacted).unwrap_or_default();
                let _ = state.history.create(&NewHistoryEntry {
                    id: history_id,
                    root: root.clone(),
                    collection_slug: slug,
                    request_id: if request.id.is_empty() { None } else { Some(request.id.clone()) },
                    environment_id,
                    method: request.method.clone(),
                    url: request.url.clone(),
                    status_code: Some(result.status as i64),
                    duration_ms: Some(result.duration_ms as i64),
                    response_size_bytes: Some(result.size_bytes as i64),
                    error_message: None,
                    request_snapshot: request_snapshot.as_str(),
                    response_snapshot: Some(response_snapshot.as_str()),
                    created_at,
                });
            }
            Err(e) => {
                let _ = state.history.create(&NewHistoryEntry {
                    id: history_id,
                    root: root.clone(),
                    collection_slug: slug,
                    request_id: if request.id.is_empty() { None } else { Some(request.id.clone()) },
                    environment_id,
                    method: request.method.clone(),
                    url: request.url.clone(),
                    status_code: None,
                    duration_ms: None,
                    response_size_bytes: None,
                    error_message: Some(redact(&e.to_string(), &secrets)),
                    request_snapshot: request_snapshot.as_str(),
                    response_snapshot: None,
                    created_at,
                });
            }
        }

        outcome
    }

    #[tauri::command]
    pub fn http_import_curl(command: String) -> Result<RequestDef, AppError> {
        import::parse_curl(&command)
    }

    /// Postman/OpenAPI 导入整份文件内容后直接落成一个新集合（不是像 `http_import_curl`
    /// 那样只解析出单个请求交给前端再手动建），因为源文件本来就是"一整份集合"的粒度，
    /// 拆成"前端逐个调用 create_request"会引入没必要的中间态（一半导入成功、用户中途
    /// 关掉对话框）。前端负责用文件选择器读出文件内容传进来，这里只管解析+落盘。
    #[tauri::command]
    pub async fn http_import_postman(root: String, content: String) -> Result<HttpCollectionSummary, AppError> {
        let imported = import::parse_postman_collection(&content)?;
        service::import_collection(&LocalFileOps, &root, imported).await
    }

    #[tauri::command]
    pub async fn http_import_openapi(root: String, content: String) -> Result<HttpCollectionSummary, AppError> {
        let imported = import::parse_openapi(&content)?;
        service::import_collection(&LocalFileOps, &root, imported).await
    }

    #[tauri::command]
    pub async fn http_export_postman(root: String, slug: String) -> Result<String, AppError> {
        export::to_postman_collection(&LocalFileOps, &root, &slug).await
    }

    #[tauri::command]
    pub async fn http_list_history(
        state: State<'_, HttpAppState>,
        root: String,
        limit: Option<i64>,
    ) -> Result<Vec<HttpRequestHistoryEntry>, AppError> {
        state.history.list_for_root(&root, limit.unwrap_or(200))
    }

    #[tauri::command]
    pub async fn http_get_history_detail(
        state: State<'_, HttpAppState>,
        id: String,
    ) -> Result<Option<HttpRequestHistoryDetail>, AppError> {
        state.history.get_detail(&id)
    }

    #[tauri::command]
    pub async fn http_delete_history(state: State<'_, HttpAppState>, id: String) -> Result<(), AppError> {
        state.history.delete(&id)
    }

    #[tauri::command]
    pub async fn http_clear_history(state: State<'_, HttpAppState>, root: String) -> Result<(), AppError> {
        state.history.clear_for_root(&root)
    }

    // ---------------------------------------------------------------------
    // 标签页
    // ---------------------------------------------------------------------

    #[tauri::command]
    pub async fn http_list_tabs(
        state: State<'_, HttpAppState>,
        root: String,
    ) -> Result<Vec<HttpWorkspaceTab>, AppError> {
        state.tabs.list_for_root(&root)
    }

    #[tauri::command]
    pub async fn http_open_tab(
        state: State<'_, HttpAppState>,
        root: String,
        slug: String,
        request_id: String,
        title: String,
    ) -> Result<HttpWorkspaceTab, AppError> {
        let existing = state.tabs.list_for_root(&root)?;
        if let Some(tab) = existing.iter().find(|t| t.request_id == request_id) {
            return Ok(tab.clone());
        }
        let tab = HttpWorkspaceTab {
            id: Uuid::new_v4().to_string(),
            root,
            collection_slug: slug,
            request_id,
            title,
            sort_order: existing.len() as i64,
            updated_at: Utc::now().to_rfc3339(),
        };
        state.tabs.create(&tab)?;
        Ok(tab)
    }

    #[tauri::command]
    pub async fn http_close_tab(state: State<'_, HttpAppState>, id: String) -> Result<(), AppError> {
        state.tabs.delete(&id)
    }
}
