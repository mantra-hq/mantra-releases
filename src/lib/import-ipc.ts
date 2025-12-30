/**
 * import-ipc - Tauri IPC 导入功能封装
 * Story 2.9: Task 6
 *
 * 提供导入功能的 Tauri IPC 调用封装：
 * - scanLogDirectory - 扫描默认路径
 * - parseLogFiles - 解析日志文件
 * - selectLogFiles - 文件选择对话框
 */

import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type { DiscoveredFile, ImportSource, ImportResult, ImportProgressData } from "@/components/import";

/**
 * 扫描指定来源的默认日志目录
 *
 * @param source - 导入来源 (claude/gemini/cursor)
 * @returns 发现的文件列表
 */
export async function scanLogDirectory(source: ImportSource): Promise<DiscoveredFile[]> {
  return invoke<DiscoveredFile[]>("scan_log_directory", { source });
}

/**
 * 解析日志文件
 *
 * @param paths - 要解析的文件路径列表
 * @param onProgress - 进度回调
 * @returns 解析结果列表
 */
export async function parseLogFiles(
  paths: string[],
  _onProgress: (progress: ImportProgressData) => void
): Promise<ImportResult[]> {
  // TODO: 使用 Tauri 事件监听进度
  // const unlisten = await listen<ImportProgressData>('import-progress', (event) => {
  //   onProgress(event.payload);
  // });

  try {
    return await invoke<ImportResult[]>("parse_log_files", { paths });
  } finally {
    // unlisten();
  }
}

/**
 * 打开文件选择对话框
 *
 * @returns 选择的文件路径列表
 */
export async function selectLogFiles(): Promise<string[]> {
  const selected = await open({
    multiple: true,
    filters: [
      {
        name: "JSON Logs",
        extensions: ["json"],
      },
    ],
  });

  if (!selected) return [];
  return Array.isArray(selected) ? selected : [selected];
}
