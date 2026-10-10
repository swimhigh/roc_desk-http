import { useEffect, useState } from "react";
import { open as openDirDialog } from "@tauri-apps/plugin-dialog";
import { Send, FolderOpen } from "lucide-react";
import { useHttpDeskStore } from "./stores/httpDeskStore";
import { ThemeToggle } from "./components/shared/ThemeToggle";
import { ToastStack } from "./components/shared/Toast";
import { HttpWorkspace } from "./components/HttpDesk/HttpWorkspace";

const ROOT_STORAGE_KEY = "roc_desk_http.root";

/** 独立壳的顶层组件——宿主 `roc_desk` 里 `HttpDeskShell` 打开 HTTP 工作台靠的是
 * 宿主"工作区"模块（`useWorkspaceStore`/`WorkspacePicker`，选一个已经注册过的
 * 本地/远程目录），这个独立工具没有那一整套工作区注册表，直接让用户填一个本地
 * 目录路径当"HTTP 集合根目录"，逻辑对应宿主 `HttpDeskShell` 里 `!current` 分支
 * 之外的部分——一旦"打开"了，下面渲染的就是和宿主完全同一份 `HttpWorkspace`/
 * `CollectionExplorer`/`RequestEditor`/`ResponsePanel`/`EnvironmentDialog`。 */
export default function App() {
  const [root, setRoot] = useState(() => localStorage.getItem(ROOT_STORAGE_KEY) ?? "");
  const [rootInput, setRootInput] = useState(root);
  const init = useHttpDeskStore((s) => s.init);

  useEffect(() => {
    if (root) void init(root);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [root]);

  function openRoot(next: string) {
    const value = next.trim();
    if (!value) return;
    localStorage.setItem(ROOT_STORAGE_KEY, value);
    setRootInput(value);
    setRoot(value);
  }

  async function pickDirectory() {
    const picked = await openDirDialog({ directory: true, multiple: false });
    if (!picked || Array.isArray(picked)) return;
    openRoot(picked);
  }

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100vh" }}>
      <div className="tab-bar">
        <Send className="app-icon" />
        <span className="workspace-name-btn" style={{ cursor: "default" }}>
          HTTP 测试工作台
        </span>
        <input
          className="form-input"
          style={{ flex: 1, minWidth: 0, marginLeft: 8, marginRight: 8 }}
          placeholder="本地集合根目录，比如 C:\Users\me\http-collections"
          value={rootInput}
          onChange={(e) => setRootInput(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && openRoot(rootInput)}
        />
        <button className="btn ghost sm" onClick={() => void pickDirectory()} title="选择目录">
          <FolderOpen size={14} />
        </button>
        <button className="btn primary sm" onClick={() => openRoot(rootInput)}>
          打开
        </button>
        <div className="quick-tools" style={{ marginLeft: "auto" }}>
          <ThemeToggle />
        </div>
      </div>
      <div style={{ flex: 1, minHeight: 0, display: "flex" }}>
        {root ? (
          <HttpWorkspace workspaceId={root} />
        ) : (
          <div className="empty-state" style={{ margin: "auto" }}>
            先在上方填一个本地目录路径（或点右侧文件夹图标选择），作为这个 HTTP 集合的根目录
          </div>
        )}
      </div>
      <ToastStack />
    </div>
  );
}
