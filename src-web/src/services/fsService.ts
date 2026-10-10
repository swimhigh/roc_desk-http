import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";

/** 这个独立工具没有"工作区"概念，导入/导出 Postman/OpenAPI 文件走系统文件
 * 对话框选好的绝对路径，直接用 `@tauri-apps/plugin-fs` 读写——不像宿主 `roc_desk`
 * 那样有 `local_read_file`/`local_write_file` 这类自定义 Tauri command（那两个
 * command 是宿主通用文件浏览器的一部分，这个工具没有搬那一整套能力，只需要
 * "读一个文件的文本内容"/"写一段文本到某个路径"这两个最小操作）。 */
export const localFileService = {
  async readFile(path: string): Promise<{ text: string }> {
    const text = await readTextFile(path);
    return { text };
  },

  async writeFile(path: string, content: string, _expectedMtime: number | null): Promise<void> {
    await writeTextFile(path, content);
  },
};
