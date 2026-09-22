#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

fn main() {
    // The main window is declared once in tauri.conf.json.
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("resolve app data dir");
            std::fs::create_dir_all(&app_data_dir).expect("create app data dir");
            let db_path = app_data_dir.join("roc_desk_http.db");
            let state = roc_desk_http::HttpAppState::new(&db_path).expect("initialize HTTP tool state");
            app.manage(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            roc_desk_http::cmd::http_list_collections,
            roc_desk_http::cmd::http_create_collection,
            roc_desk_http::cmd::http_rename_collection,
            roc_desk_http::cmd::http_delete_collection,
            roc_desk_http::cmd::http_list_requests,
            roc_desk_http::cmd::http_get_request,
            roc_desk_http::cmd::http_create_request,
            roc_desk_http::cmd::http_save_request,
            roc_desk_http::cmd::http_delete_request,
            roc_desk_http::cmd::http_list_environments,
            roc_desk_http::cmd::http_save_environment,
            roc_desk_http::cmd::http_delete_environment,
            roc_desk_http::cmd::http_get_global_variables,
            roc_desk_http::cmd::http_save_global_variables,
            roc_desk_http::cmd::http_get_collection_meta,
            roc_desk_http::cmd::http_save_collection_meta,
            roc_desk_http::cmd::http_send_request,
            roc_desk_http::cmd::http_import_curl,
            roc_desk_http::cmd::http_import_postman,
            roc_desk_http::cmd::http_import_openapi,
            roc_desk_http::cmd::http_export_postman,
            roc_desk_http::cmd::http_list_history,
            roc_desk_http::cmd::http_get_history_detail,
            roc_desk_http::cmd::http_delete_history,
            roc_desk_http::cmd::http_clear_history,
            roc_desk_http::cmd::http_list_tabs,
            roc_desk_http::cmd::http_open_tab,
            roc_desk_http::cmd::http_close_tab,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run standalone tool");
}
