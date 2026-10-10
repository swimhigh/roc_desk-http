// IPC 类型绑定——手写对照 `lib/src/http_desk/model.rs` 和 `roc_desk_core::error::AppError`
// （来自 roc_desk-common）维护，不是自动生成。改了 Rust 侧记得同步这里。

export interface KeyValueItem {
  key: string;
  value: string;
  enabled: boolean;
}

export type ApiKeyLocation = "header" | "query";

export type AuthConfig =
  | { type: "none" }
  | { type: "bearer"; token: string }
  | { type: "basic"; username: string; password: string }
  | { type: "api_key"; key: string; value: string; add_to: ApiKeyLocation };

export type RequestBody =
  | { type: "none" }
  | { type: "json"; content: string }
  | { type: "raw"; content: string; content_type: string }
  | { type: "form_url_encoded"; items: KeyValueItem[] }
  | { type: "form_data"; items: KeyValueItem[] };

export interface RequestDef {
  id: string;
  name: string;
  method: string;
  url: string;
  params: KeyValueItem[];
  headers: KeyValueItem[];
  auth: AuthConfig;
  body: RequestBody;
}

export interface RequestSummary {
  id: string;
  name: string;
  method: string;
  folder: string[];
}

export interface EnvVar {
  key: string;
  value: string;
  secret: boolean;
  enabled: boolean;
}

export interface EnvironmentDef {
  id: string;
  name: string;
  variables: EnvVar[];
}

export interface HttpCollectionMeta {
  name: string;
  description?: string | null;
  auth: AuthConfig;
  variables: EnvVar[];
}

export interface HttpCollectionSummary {
  slug: string;
  name: string;
  description?: string | null;
  request_count: number;
}

export interface HttpExecuteResult {
  status: number;
  status_text: string;
  headers: [string, string][];
  body: string;
  body_is_text: boolean;
  body_base64: string | null;
  duration_ms: number;
  size_bytes: number;
  resolved_url: string;
  truncated: boolean;
}

export interface HttpRequestHistoryEntry {
  id: string;
  root: string;
  collection_slug: string;
  request_id: string | null;
  environment_id: string | null;
  method: string;
  url: string;
  status_code: number | null;
  duration_ms: number | null;
  response_size_bytes: number | null;
  error_message: string | null;
  created_at: string;
}

export interface HttpRequestHistoryDetail {
  id: string;
  request_snapshot: string;
  response_snapshot: string | null;
}

export interface HttpWorkspaceTab {
  id: string;
  root: string;
  collection_slug: string;
  request_id: string;
  title: string;
  sort_order: number;
  updated_at: string;
}

export function newRequest(id: string, name: string): RequestDef {
  return {
    id,
    name,
    method: "GET",
    url: "",
    params: [],
    headers: [],
    auth: { type: "none" },
    body: { type: "none" },
  };
}

/** 对应 `roc_desk_core::error::AppError`（`roc_desk-common` 仓库）——
 * `#[serde(tag = "kind", content = "message")]`，Tauri `invoke()` 的 rejection
 * 是这个裸对象，不是 JS `Error` 实例。 */
export type AppErrorKind =
  | "Connection"
  | "Auth"
  | "HostKeyRejected"
  | "PermissionDenied"
  | "NotFound"
  | "Database"
  | "Conflict"
  | "Internal";

export interface AppError {
  kind: AppErrorKind;
  message: string;
}

export function isAppError(e: unknown): e is AppError {
  return typeof e === "object" && e !== null && "kind" in e && "message" in e;
}
