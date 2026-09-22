import { invoke } from "@tauri-apps/api/core";
import type {
  EnvironmentDef,
  EnvVar,
  HttpCollectionMeta,
  HttpCollectionSummary,
  HttpExecuteResult,
  HttpRequestHistoryDetail,
  HttpRequestHistoryEntry,
  RequestDef,
  RequestSummary,
} from "./types";

/** IPC 封装——所有 command 都用本地目录路径 `root` 标识"当前打开的 HTTP 集合根
 * 目录"，不再依赖宿主"已打开工作区注册表"的 `workspaceId`。 */
export const api = {
  listCollections(root: string): Promise<HttpCollectionSummary[]> {
    return invoke("http_list_collections", { root });
  },
  createCollection(root: string, name: string): Promise<HttpCollectionSummary> {
    return invoke("http_create_collection", { root, name });
  },
  renameCollection(root: string, slug: string, name: string): Promise<void> {
    return invoke("http_rename_collection", { root, slug, name });
  },
  deleteCollection(root: string, slug: string): Promise<void> {
    return invoke("http_delete_collection", { root, slug });
  },
  getCollectionMeta(root: string, slug: string): Promise<HttpCollectionMeta> {
    return invoke("http_get_collection_meta", { root, slug });
  },
  saveCollectionMeta(root: string, slug: string, meta: HttpCollectionMeta): Promise<void> {
    return invoke("http_save_collection_meta", { root, slug, meta });
  },

  listRequests(root: string, slug: string): Promise<RequestSummary[]> {
    return invoke("http_list_requests", { root, slug });
  },
  getRequest(root: string, slug: string, id: string): Promise<RequestDef> {
    return invoke("http_get_request", { root, slug, id });
  },
  createRequest(root: string, slug: string, name: string, folder: string[] = []): Promise<RequestDef> {
    return invoke("http_create_request", { root, slug, name, folder });
  },
  saveRequest(root: string, slug: string, request: RequestDef): Promise<void> {
    return invoke("http_save_request", { root, slug, request });
  },
  deleteRequest(root: string, slug: string, id: string): Promise<void> {
    return invoke("http_delete_request", { root, slug, id });
  },

  listEnvironments(root: string, slug: string): Promise<EnvironmentDef[]> {
    return invoke("http_list_environments", { root, slug });
  },
  saveEnvironment(root: string, slug: string, environment: EnvironmentDef): Promise<EnvironmentDef> {
    return invoke("http_save_environment", { root, slug, environment });
  },
  deleteEnvironment(root: string, slug: string, envId: string): Promise<void> {
    return invoke("http_delete_environment", { root, slug, envId });
  },
  getGlobalVariables(root: string): Promise<EnvVar[]> {
    return invoke("http_get_global_variables", { root });
  },
  saveGlobalVariables(root: string, variables: EnvVar[]): Promise<void> {
    return invoke("http_save_global_variables", { root, variables });
  },

  sendRequest(
    root: string,
    slug: string,
    request: RequestDef,
    environmentId: string | null,
  ): Promise<HttpExecuteResult> {
    return invoke("http_send_request", { root, slug, request, environmentId });
  },
  importCurl(command: string): Promise<RequestDef> {
    return invoke("http_import_curl", { command });
  },
  importPostman(root: string, content: string): Promise<HttpCollectionSummary> {
    return invoke("http_import_postman", { root, content });
  },
  importOpenApi(root: string, content: string): Promise<HttpCollectionSummary> {
    return invoke("http_import_openapi", { root, content });
  },
  exportPostman(root: string, slug: string): Promise<string> {
    return invoke("http_export_postman", { root, slug });
  },

  listHistory(root: string, limit = 200): Promise<HttpRequestHistoryEntry[]> {
    return invoke("http_list_history", { root, limit });
  },
  getHistoryDetail(id: string): Promise<HttpRequestHistoryDetail | null> {
    return invoke("http_get_history_detail", { id });
  },
  deleteHistory(id: string): Promise<void> {
    return invoke("http_delete_history", { id });
  },
  clearHistory(root: string): Promise<void> {
    return invoke("http_clear_history", { root });
  },
};
