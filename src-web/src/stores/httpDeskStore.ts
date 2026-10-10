import { create } from "zustand";
import { httpDeskService } from "../services/httpDeskService";
import { formatError } from "../utils/error";
import type {
  EnvironmentDef,
  EnvVar,
  HttpCollectionSummary,
  HttpExecuteResult,
  HttpRequestHistoryEntry,
  HttpWorkspaceTab,
  RequestDef,
  RequestSummary,
} from "../types/bindings";

function emptyRequest(): RequestDef {
  return {
    id: "",
    name: "",
    method: "GET",
    url: "",
    params: [],
    headers: [],
    auth: { type: "none" },
    body: { type: "none" },
  };
}

interface HttpDeskState {
  /** 本地 HTTP 集合根目录的绝对路径——对应宿主 `roc_desk` 里的 `workspaceId`，
   * 这个独立工具没有"已打开工作区注册表"，直接用目录路径本身当标识。 */
  root: string | null;
  collections: HttpCollectionSummary[];
  activeSlug: string | null;
  requests: RequestSummary[];
  environments: EnvironmentDef[];
  activeEnvironmentId: string | null;
  globalVariables: EnvVar[];

  tabs: HttpWorkspaceTab[];
  activeTabId: string | null;
  /** 每个打开中的请求的可编辑草稿——切标签页不丢未保存的改动；key 是 request id。 */
  drafts: Record<string, RequestDef>;
  dirty: Record<string, boolean>;

  response: HttpExecuteResult | null;
  sendError: string | null;
  sending: boolean;

  history: HttpRequestHistoryEntry[];

  loading: boolean;
  error: string | null;

  init: (root: string) => Promise<void>;
  loadCollections: () => Promise<void>;
  createCollection: (name: string) => Promise<void>;
  renameCollection: (slug: string, name: string) => Promise<void>;
  deleteCollection: (slug: string) => Promise<void>;
  selectCollection: (slug: string) => Promise<void>;
  loadRequests: () => Promise<void>;
  loadEnvironments: () => Promise<void>;
  saveEnvironment: (env: EnvironmentDef) => Promise<void>;
  deleteEnvironment: (id: string) => Promise<void>;
  loadGlobalVariables: () => Promise<void>;
  saveGlobalVariables: (vars: EnvVar[]) => Promise<void>;
  setActiveEnvironment: (id: string | null) => void;

  createRequest: (name: string, folder?: string[]) => Promise<void>;
  deleteRequest: (id: string) => Promise<void>;
  openRequestTab: (id: string, name: string) => Promise<void>;
  closeTab: (tabId: string) => Promise<void>;
  setActiveTab: (tabId: string) => void;
  updateDraft: (requestId: string, patch: Partial<RequestDef>) => void;
  saveDraft: (requestId: string) => Promise<void>;
  sendDraft: (requestId: string) => Promise<void>;

  loadHistory: () => Promise<void>;
  clearHistory: () => Promise<void>;
  importCurl: (command: string) => Promise<void>;
  /** Postman/OpenAPI 导入整份文件内容为一个新集合——和 `importCurl`（只解析出单个
   * 请求塞进当前集合）不一样，落地成功后直接切到新建出来的集合。 */
  importPostmanCollection: (content: string) => Promise<void>;
  importOpenApiSpec: (content: string) => Promise<void>;
  /** 导出当前选中集合为 Postman Collection v2.1 JSON 字符串，写盘交给调用方
   * （文件选择对话框那一层，见 CollectionExplorer.tsx）。 */
  exportCurrentCollection: () => Promise<string>;
}

export const useHttpDeskStore = create<HttpDeskState>((set, get) => ({
  root: null,
  collections: [],
  activeSlug: null,
  requests: [],
  environments: [],
  activeEnvironmentId: null,
  globalVariables: [],

  tabs: [],
  activeTabId: null,
  drafts: {},
  dirty: {},

  response: null,
  sendError: null,
  sending: false,

  history: [],

  loading: false,
  error: null,

  init: async (root) => {
    set({
      root,
      collections: [],
      activeSlug: null,
      requests: [],
      environments: [],
      activeEnvironmentId: null,
      tabs: [],
      activeTabId: null,
      drafts: {},
      dirty: {},
      response: null,
      sendError: null,
      history: [],
      error: null,
    });
    await get().loadCollections();
    const tabs = await httpDeskService.listTabs(root);
    set({ tabs });
    if (tabs.length > 0) {
      set({ activeSlug: tabs[0].collection_slug });
      await get().loadRequests();
      await get().loadEnvironments();
      get().setActiveTab(tabs[0].id);
    }
  },

  loadCollections: async () => {
    const { root } = get();
    if (!root) return;
    set({ loading: true, error: null });
    try {
      const collections = await httpDeskService.listCollections(root);
      set({ collections, loading: false });
      if (!get().activeSlug && collections.length > 0) {
        await get().selectCollection(collections[0].slug);
      }
    } catch (e) {
      set({ loading: false, error: formatError(e) });
    }
  },

  createCollection: async (name) => {
    const { root } = get();
    if (!root) return;
    const collection = await httpDeskService.createCollection(root, name);
    await get().loadCollections();
    await get().selectCollection(collection.slug);
  },

  renameCollection: async (slug, name) => {
    const { root } = get();
    if (!root) return;
    await httpDeskService.renameCollection(root, slug, name);
    await get().loadCollections();
  },

  deleteCollection: async (slug) => {
    const { root } = get();
    if (!root) return;
    await httpDeskService.deleteCollection(root, slug);
    if (get().activeSlug === slug) {
      set({ activeSlug: null, requests: [], environments: [], activeEnvironmentId: null });
    }
    await get().loadCollections();
  },

  selectCollection: async (slug) => {
    set({ activeSlug: slug });
    await Promise.all([get().loadRequests(), get().loadEnvironments()]);
  },

  loadRequests: async () => {
    const { root, activeSlug } = get();
    if (!root || !activeSlug) return;
    try {
      const requests = await httpDeskService.listRequests(root, activeSlug);
      set({ requests });
    } catch (e) {
      set({ error: formatError(e) });
    }
  },

  loadEnvironments: async () => {
    const { root, activeSlug } = get();
    if (!root || !activeSlug) return;
    try {
      const environments = await httpDeskService.listEnvironments(root, activeSlug);
      set((s) => ({
        environments,
        activeEnvironmentId:
          s.activeEnvironmentId && environments.some((e) => e.id === s.activeEnvironmentId)
            ? s.activeEnvironmentId
            : (environments[0]?.id ?? null),
      }));
      await get().loadGlobalVariables();
    } catch (e) {
      set({ error: formatError(e) });
    }
  },

  saveEnvironment: async (env) => {
    const { root, activeSlug } = get();
    if (!root || !activeSlug) return;
    await httpDeskService.saveEnvironment(root, activeSlug, env);
    await get().loadEnvironments();
  },

  deleteEnvironment: async (id) => {
    const { root, activeSlug } = get();
    if (!root || !activeSlug) return;
    await httpDeskService.deleteEnvironment(root, activeSlug, id);
    await get().loadEnvironments();
  },

  loadGlobalVariables: async () => {
    const { root } = get();
    if (!root) return;
    const globalVariables = await httpDeskService.getGlobalVariables(root);
    set({ globalVariables });
  },

  saveGlobalVariables: async (vars) => {
    const { root } = get();
    if (!root) return;
    await httpDeskService.saveGlobalVariables(root, vars);
    await get().loadGlobalVariables();
  },

  setActiveEnvironment: (id) => set({ activeEnvironmentId: id }),

  createRequest: async (name, folder = []) => {
    const { root, activeSlug } = get();
    if (!root || !activeSlug) return;
    const req = await httpDeskService.createRequest(root, activeSlug, name, folder);
    await get().loadRequests();
    await get().openRequestTab(req.id, req.name);
  },

  deleteRequest: async (id) => {
    const { root, activeSlug, tabs } = get();
    if (!root || !activeSlug) return;
    await httpDeskService.deleteRequest(root, activeSlug, id);
    const tab = tabs.find((t) => t.request_id === id);
    if (tab) await get().closeTab(tab.id);
    await get().loadRequests();
  },

  openRequestTab: async (id, name) => {
    const { root, activeSlug } = get();
    if (!root || !activeSlug) return;
    const tab = await httpDeskService.openTab(root, activeSlug, id, name);
    const draft = await httpDeskService.getRequest(root, activeSlug, id);
    set((s) => ({
      tabs: s.tabs.some((t) => t.id === tab.id) ? s.tabs : [...s.tabs, tab],
      drafts: { ...s.drafts, [id]: draft },
      activeTabId: tab.id,
      response: null,
      sendError: null,
    }));
  },

  closeTab: async (tabId) => {
    await httpDeskService.closeTab(tabId);
    set((s) => {
      const closed = s.tabs.find((t) => t.id === tabId);
      const tabs = s.tabs.filter((t) => t.id !== tabId);
      const drafts = { ...s.drafts };
      if (closed) delete drafts[closed.request_id];
      const activeTabId = s.activeTabId === tabId ? (tabs[0]?.id ?? null) : s.activeTabId;
      return { tabs, drafts, activeTabId };
    });
  },

  setActiveTab: (tabId) => set({ activeTabId: tabId, response: null, sendError: null }),

  updateDraft: (requestId, patch) =>
    set((s) => ({
      drafts: { ...s.drafts, [requestId]: { ...(s.drafts[requestId] ?? emptyRequest()), ...patch } },
      dirty: { ...s.dirty, [requestId]: true },
    })),

  saveDraft: async (requestId) => {
    const { root, activeSlug, drafts } = get();
    const draft = drafts[requestId];
    if (!root || !activeSlug || !draft) return;
    await httpDeskService.saveRequest(root, activeSlug, draft);
    set((s) => ({ dirty: { ...s.dirty, [requestId]: false } }));
    await get().loadRequests();
  },

  sendDraft: async (requestId) => {
    const { root, activeSlug, drafts, activeEnvironmentId } = get();
    const draft = drafts[requestId];
    if (!root || !activeSlug || !draft) return;
    set({ sending: true, sendError: null, response: null });
    try {
      const result = await httpDeskService.sendRequest(root, activeSlug, draft, activeEnvironmentId);
      set({ response: result, sending: false });
      void get().loadHistory();
    } catch (e) {
      set({ sendError: formatError(e), sending: false });
      void get().loadHistory();
    }
  },

  loadHistory: async () => {
    const { root } = get();
    if (!root) return;
    try {
      const history = await httpDeskService.listHistory(root);
      set({ history });
    } catch (e) {
      set({ error: formatError(e) });
    }
  },

  clearHistory: async () => {
    const { root } = get();
    if (!root) return;
    await httpDeskService.clearHistory(root);
    await get().loadHistory();
  },

  importCurl: async (command) => {
    const { activeSlug, root } = get();
    if (!activeSlug || !root) return;
    const parsed = await httpDeskService.importCurl(command);
    const name = (() => {
      try {
        const u = new URL(parsed.url);
        return `${parsed.method} ${u.pathname}`;
      } catch {
        return parsed.method + " " + parsed.url;
      }
    })();
    const created = await httpDeskService.createRequest(root, activeSlug, name, []);
    const merged: RequestDef = { ...parsed, id: created.id, name };
    await httpDeskService.saveRequest(root, activeSlug, merged);
    await get().loadRequests();
    await get().openRequestTab(created.id, name);
  },

  importPostmanCollection: async (content) => {
    const { root } = get();
    if (!root) return;
    const collection = await httpDeskService.importPostman(root, content);
    await get().loadCollections();
    await get().selectCollection(collection.slug);
  },

  importOpenApiSpec: async (content) => {
    const { root } = get();
    if (!root) return;
    const collection = await httpDeskService.importOpenApi(root, content);
    await get().loadCollections();
    await get().selectCollection(collection.slug);
  },

  exportCurrentCollection: async () => {
    const { root, activeSlug } = get();
    if (!root || !activeSlug) throw new Error("请先选择一个集合");
    return httpDeskService.exportPostman(root, activeSlug);
  },
}));
