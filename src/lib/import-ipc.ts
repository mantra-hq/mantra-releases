/**
 * import-ipc - Tauri IPC 导入功能封装
 * Story 2.9: Task 6
 * Story 2.23: Import Progress Events
 *
 * 提供导入功能的 Tauri IPC 调用封装：
 * - scanLogDirectory - 扫描默认路径
 * - parseLogFiles - 解析日志文件
 * - selectLogDirectory - 目录选择对话框
 * - importSessionsWithProgress - 带进度事件的导入
 * - cancelImport - 取消导入
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
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

// ============================================================================
// Story 2.23: Import with Progress Events
// ============================================================================

/** 进度事件载荷 */
export interface ImportProgressEvent {
  current: number;
  total: number;
  currentFile: string;
  successCount: number;
  failureCount: number;
}

/** 文件处理完成事件载荷 */
export interface ImportFileDoneEvent {
  filePath: string;
  success: boolean;
  error?: string;
  projectId?: string;
  sessionId?: string;
}

/** 导入取消事件载荷 */
export interface ImportCancelledEvent {
  processedCount: number;
  successCount: number;
  failureCount: number;
}

/** 进度回调集合 */
export interface ImportProgressCallbacks {
  /** 每个文件处理前调用 */
  onProgress?: (event: ImportProgressEvent) => void;
  /** 每个文件处理完成后调用 */
  onFileDone?: (event: ImportFileDoneEvent) => void;
  /** 导入取消时调用 */
  onCancelled?: (event: ImportCancelledEvent) => void;
}

/**
 * 带进度事件的导入函数
 *
 * @param paths - 要导入的文件路径列表
 * @param callbacks - 进度回调函数集合
 * @returns 导入结果列表
 */
export async function importSessionsWithProgress(
  paths: string[],
  callbacks: ImportProgressCallbacks
): Promise<ImportResult[]> {
  const unlisteners: UnlistenFn[] = [];
  const results: ImportResult[] = [];
  const fileResults = new Map<string, ImportFileDoneEvent>();

  try {
    // 设置事件监听
    if (callbacks.onProgress) {
      const unlisten = await listen<ImportProgressEvent>("import-progress", (event) => {
        callbacks.onProgress?.(event.payload);
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onFileDone) {
      const unlisten = await listen<ImportFileDoneEvent>("import-file-done", (event) => {
        fileResults.set(event.payload.filePath, event.payload);
        callbacks.onFileDone?.(event.payload);
      });
      unlisteners.push(unlisten);
    }

    if (callbacks.onCancelled) {
      const unlisten = await listen<ImportCancelledEvent>("import-cancelled", (event) => {
        callbacks.onCancelled?.(event.payload);
      });
      unlisteners.push(unlisten);
    }

    // 调用后端命令
    await invoke<BackendImportResult>("import_sessions_with_progress", { paths });

    // 从文件结果构建返回值
    for (const path of paths) {
      const fileResult = fileResults.get(path);
      if (fileResult) {
        results.push({
          success: fileResult.success,
          filePath: fileResult.filePath,
          projectId: fileResult.projectId,
          sessionId: fileResult.sessionId,
          error: fileResult.error,
        });
      } else {
        // 文件可能在取消前未处理
        results.push({
          success: false,
          filePath: path,
          error: "导入已取消",
        });
      }
    }

    return results;
  } finally {
    // 清理事件监听
    for (const unlisten of unlisteners) {
      unlisten();
    }
  }
}

/**
 * 取消当前导入操作
 */
export async function cancelImport(): Promise<void> {
  await invoke("cancel_import");
}

