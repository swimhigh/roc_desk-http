import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import path from "node:path";

// 这个前端只服务 standalone/ 目录下的独立 Tauri 外壳，构建产物必须落在
// standalone/dist（tauri.conf.json 的 build.frontendDist 指向那里），
// 不是 src-web 自己旁边的 dist——之前 explorer/editor 忘记这步导致 standalone exe
// 一直是占位页面，这次一次做对。
export default defineConfig({
  plugins: [react()],
  root: __dirname,
  build: {
    outDir: path.resolve(__dirname, "../standalone/dist"),
    emptyOutDir: true,
  },
  clearScreen: false,
});
