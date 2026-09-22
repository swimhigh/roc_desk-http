-- HTTP 测试工作台自己的 SQLite 存储：不存集合/请求/环境的内容（那些落在 `root`
-- 指向的本地目录下的 `.rock_desk/http/` 子目录，YAML 是唯一真相，见
-- `http_desk::service`），这里只存两张 UI/审计状态表。这个独立工具没有宿主那种
-- "已打开工作区注册表"，所以不再有 `workspace_id` 外键——`root` 就是一个普通的
-- 本地目录路径字符串，不受外键约束。
CREATE TABLE http_tabs (
  id TEXT PRIMARY KEY,
  root TEXT NOT NULL,
  collection_slug TEXT NOT NULL,
  request_id TEXT NOT NULL,
  title TEXT NOT NULL,
  sort_order INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL
);
CREATE INDEX idx_http_tabs_root ON http_tabs(root);

CREATE TABLE http_request_history (
  id TEXT PRIMARY KEY,
  root TEXT NOT NULL,
  collection_slug TEXT NOT NULL,
  request_id TEXT,
  environment_id TEXT,
  method TEXT NOT NULL,
  url TEXT NOT NULL,
  status_code INTEGER,
  duration_ms INTEGER,
  response_size_bytes INTEGER,
  error_message TEXT,
  request_snapshot TEXT NOT NULL,
  response_snapshot TEXT,
  created_at TEXT NOT NULL
);
CREATE INDEX idx_http_request_history_root ON http_request_history(root, created_at DESC);
