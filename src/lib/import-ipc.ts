/**
 * import-ipc - Tauri IPC 导入功能封装
 * Story 2.9: Task 6
 *
 * 提供导入功能的 Tauri IPC 调用封装：
 * - scanLogDirectory - 扫描默认路径
 * - parseLogFiles - 解析日志文件
 * - selectLogDirectory - 目录选择对话框
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
 * 后端 import_sessions 返回的结果类型
 */
interface BackendImportResult {
  imported_count: number;
  skipped_count: number;
  new_projects_count: number;
  errors: string[];
}

/**
 * 解析并导入日志文件
 *
 * @param paths - 要导入的文件路径列表
 * @param onProgress - 进度回调 (当前未使用，后续实现)
 * @returns 导入结果列表
 */
export async function parseLogFiles(
  paths: string[],
  _onProgress: (progress: ImportProgressData) => void
): Promise<ImportResult[]> {
  // 调用 import_sessions 而不是 parse_log_files
  // import_sessions 会解析文件并将会话保存到数据库
  const backendResult = await invoke<BackendImportResult>("import_sessions", { paths });

  // 将后端结果转换为前端期望的格式
  // 由于后端返回的是汇总统计，我们为每个路径生成对应结果
  const results: ImportResult[] = [];
  const errorPaths = new Set<string>();

  // 解析错误信息获取失败的文件路径
  for (const errorMsg of backendResult.errors) {
    // 错误格式: "路径: 错误信息"
    const colonIdx = errorMsg.indexOf(": ");
    if (colonIdx > 0) {
      errorPaths.add(errorMsg.substring(0, colonIdx));
    }
  }

  // 为每个输入路径生成结果
  for (const path of paths) {
    const hasError = errorPaths.has(path);
    const errorMsg = backendResult.errors.find(e => e.startsWith(path + ": "));

    results.push({
      success: !hasError,
      filePath: path,
      projectId: hasError ? undefined : "imported",
      sessionId: hasError ? undefined : "imported",
      error: hasError ? errorMsg?.substring(path.length + 2) : undefined,
    });
  }

  return results;
}

/**
 * 打开目录选择对话框并扫描选中目录的日志文件
 *
 * @returns 发现的文件列表
 */
export async function selectLogFiles(): Promise<DiscoveredFile[]> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "选择日志目录",
  });

  if (!selected || typeof selected !== "string") return [];

  // 调用后端扫描选中的目录
  return invoke<DiscoveredFile[]>("scan_custom_directory", { path: selected });
}

