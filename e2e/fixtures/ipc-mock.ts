/**
 * IPC Mock - E2E 测试 IPC Mock 处理器
 * Story 9.2: Task 4
 * Story 14.9: Task 8.1 (Tauri 插件 Mock 支持)
 *
 * 实现所有 Tauri IPC 命令的 Mock 版本
 * 用于 Playwright E2E 测试，独立于 Rust 后端运行
 */

import type { InvokeArgs } from "@tauri-apps/api/core";
import {
  MOCK_PROJECTS,
  MOCK_SNAPSHOT,
  MOCK_DART_CODE,
  MOCK_BUILTIN_RULES,
  MOCK_DEFAULT_PATHS,
  MOCK_DISCOVERED_FILES,
  MOCK_SEARCH_RESULTS,
  MOCK_MCP_SERVICES,
  MOCK_GATEWAY_STATUS_RUNNING,
  MOCK_MCP_TOOLS,
  getSessionsByProjectId,
  getSessionById,
  getProjectBySessionId,
} from "./mock-data";

/**
 * 模拟网络延迟 (10-50ms)
 * 更真实地模拟 IPC 调用
 */
function simulateDelay(): Promise<void> {
  const delay = Math.random() * 40 + 10; // 10-50ms
  return new Promise((resolve) => setTimeout(resolve, delay));
}

/**
 * 解析参数
 * 处理不同格式的参数传递
 */
function getArg<T>(args: InvokeArgs | undefined, key: string): T | undefined {
  if (!args) return undefined;
  // 处理对象格式的参数
  if (typeof args === "object" && args !== null) {
    return (args as Record<string, unknown>)[key] as T | undefined;
  }
  return undefined;
}

// ==========================================================================
// Story 14.9: Tauri 插件 Mock 支持 (Channel 回调管理)
// ==========================================================================

/**
 * Channel 回调注册表
 * 用于模拟 Tauri Channel 机制（插件通过 Channel 发送进度事件）
 */
const callbackMap: Record<number, { callback: (payload: unknown) => void; once: boolean }> = {};
let callbackIdCounter = 0;

/**
 * 调用已注册的 Channel 回调
 */
function callMockCallback(id: number, payload: unknown): void {
  const entry = callbackMap[id];
  if (entry) {
    entry.callback(payload);
    if (entry.once) delete callbackMap[id];
  }
}

/**
 * Mock 更新配置（可通过 window.__MOCK_UPDATE_CONFIG__ 从测试中控制）
 */
interface MockUpdateConfig {
  hasUpdate: boolean;
  version?: string;
  body?: string;
}

function getMockUpdateConfig(): MockUpdateConfig {
  const config = (window as unknown as Record<string, unknown>).__MOCK_UPDATE_CONFIG__ as MockUpdateConfig | undefined;
  return config ?? { hasUpdate: true, version: "0.8.0", body: "Bug fixes and improvements" };
}

/**
 * 初始化 window.__TAURI_INTERNALS__ Mock
 *
 * Tauri 插件（updater, process, app）直接调用 window.__TAURI_INTERNALS__.invoke()，
 * 不经过 ipc-adapter.ts。此函数在 Playwright 模式下拦截这些调用。
 */
export function setupTauriInternals(): void {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (window as any).__TAURI_INTERNALS__ = {
    invoke: async (cmd: string, args?: InvokeArgs) => {
      return mockInvoke(cmd, args);
    },
    transformCallback: (callback: (payload: unknown) => void, once: boolean = false) => {
      const id = callbackIdCounter++;
      callbackMap[id] = { callback, once };
      return id;
    },
    convertFileSrc: (filePath: string) => filePath,
  };
  console.log("[IPC Mock] window.__TAURI_INTERNALS__ 已设置");
}

/**
 * Mock Invoke 处理器
 *
 * 根据命令名路由到对应的 Mock 实现
 * 未实现的命令打印警告并返回 null
 *
 * @param cmd - Tauri 命令名
 * @param args - 命令参数
 * @returns Mock 结果
 */
export async function mockInvoke<T>(cmd: string, args?: InvokeArgs): Promise<T> {
  await simulateDelay();

  console.log(`[IPC Mock] ${cmd}`, args);

  switch (cmd) {
    // ==========================================================================
    // 项目相关 Mock (AC #4: 4.2)
    // ==========================================================================

    case "list_projects":
      return MOCK_PROJECTS as T;

    case "get_project": {
      const projectId = getArg<string>(args, "projectId");
      const project = MOCK_PROJECTS.find((p) => p.id === projectId);
      return (project ?? null) as T;
    }

    case "get_project_by_cwd": {
      const cwd = getArg<string>(args, "cwd");
      const project = MOCK_PROJECTS.find((p) => p.cwd === cwd);
      return (project ?? null) as T;
    }

    case "get_project_sessions": {
      const projectId = getArg<string>(args, "projectId");
      if (!projectId) return [] as T;
      const sessions = getSessionsByProjectId(projectId);
      return sessions as T;
    }

    case "get_logical_project_stats": {
      // 基于 MOCK_PROJECTS 生成逻辑项目统计
      return MOCK_PROJECTS.map((p) => ({
        physical_path: p.cwd,
        project_count: 1,
        project_ids: [p.id],
        total_sessions: p.session_count,
        last_activity: p.last_activity,
        display_name: p.name,
        path_type: "local" as const,
        path_exists: true,
        needs_association: false,
        has_git_repo: p.has_git_repo,
      })) as T;
    }

    // ==========================================================================
    // 会话相关 Mock (AC #4: 4.3)
    // ==========================================================================

    case "get_session": {
      const sessionId = getArg<string>(args, "sessionId");
      if (!sessionId) return null as T;
      const session = getSessionById(sessionId);
      return session as T;
    }

    case "get_project_by_session": {
      const sessionId = getArg<string>(args, "sessionId");
      if (!sessionId) return null as T;
      const project = getProjectBySessionId(sessionId);
      return project as T;
    }

    // ==========================================================================
    // Git 相关 Mock (AC #4: 4.4)
    // ==========================================================================

    case "detect_git_repo": {
      const dirPath = getArg<string>(args, "dirPath");
      // 检查是否是已知的 Mock 项目路径
      const project = MOCK_PROJECTS.find((p) => dirPath?.startsWith(p.cwd));
      if (project?.has_git_repo) {
        return project.git_repo_path as T;
      }
      return null as T;
    }

    case "get_representative_file": {
      const repoPath = getArg<string>(args, "repoPath");
      // 返回 Mock 代表性文件
      if (repoPath) {
        return {
          path: "README.md",
          content: `# Mock Project\n\nThis is a mock project for E2E testing.`,
          language: "markdown",
        } as T;
      }
      return null as T;
    }

    case "get_file_at_head": {
      const filePath = getArg<string>(args, "filePath");
      // 为 Dart 文件返回 Dart 代码
      if (filePath?.endsWith(".dart")) {
        return {
          content: MOCK_DART_CODE,
          commit_hash: "dart123abc",
          commit_message: "feat: add user model",
          commit_timestamp: 1704153600,
        } as T;
      }
      return MOCK_SNAPSHOT as T;
    }

    case "get_snapshot_at_time": {
      const filePath = getArg<string>(args, "filePath");
      // 为 Dart 文件返回 Dart 代码
      if (filePath?.endsWith(".dart")) {
        return {
          content: MOCK_DART_CODE,
          commit_hash: "dart123abc",
          commit_message: "feat: add user model",
          commit_timestamp: 1704153600,
        } as T;
      }
      return MOCK_SNAPSHOT as T;
    }

    case "get_snapshot_with_fallback": {
      const filePath = getArg<string>(args, "filePath");
      // 为 Dart 文件返回 Dart 代码
      if (filePath?.endsWith(".dart")) {
        return {
          content: MOCK_DART_CODE,
          commit_hash: "dart123abc",
          commit_message: "feat: add user model",
          commit_timestamp: 1704153600,
          source: "git",
        } as T;
      }
      return {
        ...MOCK_SNAPSHOT,
        source: "git",
      } as T;
    }

    case "list_files_at_commit": {
      // 返回 Mock 文件列表
      return [
        "README.md",
        "src/index.ts",
        "src/components/App.tsx",
        "package.json",
      ] as T;
    }

    case "list_tree_at_commit": {
      // 返回 Mock 文件树 (TreeNode[] 格式)
      return [
        {
          name: "src",
          path: "src",
          type: "directory",
          children: [
            { name: "index.ts", path: "src/index.ts", type: "file" },
            {
              name: "components",
              path: "src/components",
              type: "directory",
              children: [
                { name: "App.tsx", path: "src/components/App.tsx", type: "file" },
              ],
            },
          ],
        },
        { name: "README.md", path: "README.md", type: "file" },
        { name: "package.json", path: "package.json", type: "file" },
      ] as T;
    }

    // ==========================================================================
    // 搜索相关 Mock (AC #4: 4.5)
    // ==========================================================================

    case "search_sessions": {
      const query = getArg<string>(args, "query");
      if (!query || query.trim() === "") {
        return [] as T;
      }
      // 简单模拟：返回预设结果
      return MOCK_SEARCH_RESULTS as T;
    }

    // ==========================================================================
    // 导入相关 Mock (AC #4: 4.6)
    // ==========================================================================

    case "get_imported_session_ids": {
      // 返回部分已导入的会话 ID
      return ["mock-session-alpha-1", "mock-session-beta-1"] as T;
    }

    case "get_default_paths": {
      return MOCK_DEFAULT_PATHS as T;
    }

    case "scan_log_directory": {
      return MOCK_DISCOVERED_FILES as T;
    }

    case "scan_custom_directory": {
      return MOCK_DISCOVERED_FILES as T;
    }

    case "import_sessions": {
      return {
        imported_count: 1,
        skipped_count: 0,
        new_projects_count: 0,
        errors: [],
      } as T;
    }

    case "import_sessions_with_progress": {
      return {
        imported_count: 1,
        skipped_count: 0,
        new_projects_count: 0,
        errors: [],
      } as T;
    }

    case "cancel_import": {
      return undefined as T;
    }

    // ==========================================================================
    // 脱敏相关 Mock (AC #4: 4.7)
    // ==========================================================================

    case "sanitize_text": {
      const text = getArg<string>(args, "text");
      return {
        sanitized_text: text ?? "",
        has_matches: false,
        stats: { counts: {}, total: 0 },
      } as T;
    }

    case "sanitize_session": {
      const sessionId = getArg<string>(args, "sessionId");
      const session = sessionId ? getSessionById(sessionId) : null;
      const originalText = session ? JSON.stringify(session, null, 2) : "";
      return {
        sanitized_text: originalText,
        has_matches: false,
        stats: { counts: {}, total: 0 },
      } as T;
    }

    case "validate_regex": {
      const pattern = getArg<string>(args, "pattern");
      try {
        new RegExp(pattern ?? "");
        return { valid: true } as T;
      } catch {
        return { valid: false, error: "Invalid regex" } as T;
      }
    }

    case "get_builtin_rules": {
      return MOCK_BUILTIN_RULES as T;
    }

    // ==========================================================================
    // 项目管理相关 Mock
    // ==========================================================================

    case "sync_project": {
      return {
        new_sessions: [],
        updated_sessions: [],
        unchanged_count: 3,
      } as T;
    }

    case "remove_project": {
      return undefined as T;
    }

    case "rename_project": {
      return undefined as T;
    }

    case "update_project_cwd": {
      const projectId = getArg<string>(args, "projectId");
      const newCwd = getArg<string>(args, "newCwd");
      const project = MOCK_PROJECTS.find((p) => p.id === projectId);
      if (project) {
        return { ...project, cwd: newCwd } as T;
      }
      return null as T;
    }

    // ==========================================================================
    // MCP 服务相关 Mock (Story 11.11: Inspector)
    // ==========================================================================

    case "list_mcp_services":
      return MOCK_MCP_SERVICES as T;

    case "get_gateway_status":
      return MOCK_GATEWAY_STATUS_RUNNING as T;

    case "toggle_mcp_service": {
      const id = getArg<string>(args, "id");
      const enabled = getArg<boolean>(args, "enabled");
      const service = MOCK_MCP_SERVICES.find((s) => s.id === id);
      if (service) {
        return { ...service, enabled: enabled ?? !service.enabled } as T;
      }
      return null as T;
    }

    case "start_gateway":
      return MOCK_GATEWAY_STATUS_RUNNING as T;

    case "stop_gateway":
      return { ...MOCK_GATEWAY_STATUS_RUNNING, running: false, port: null } as T;

    case "regenerate_gateway_token":
      return "new-mock-token-67890" as T;

    case "discover_tools": {
      return {
        serviceId: getArg<string>(args, "serviceId") ?? "unknown",
        tools: MOCK_MCP_TOOLS,
        fromCache: false,
      } as T;
    }

    // ==========================================================================
    // Tauri 插件 Mock (Story 14.9: Task 8.1)
    // ==========================================================================

    // Plugin: Updater
    case "plugin:updater|check": {
      const config = getMockUpdateConfig();
      if (!config.hasUpdate) {
        return null as T;
      }
      return {
        rid: 1,
        version: config.version ?? "0.8.0",
        date: "2026-02-08",
        ...(config.body !== undefined ? { body: config.body } : {}),
      } as T;
    }

    case "plugin:updater|download_and_install": {
      // 获取 Channel 对象，通过回调模拟下载进度
      const onEvent = getArg<{ id: number }>(args, "onEvent");
      if (onEvent && typeof onEvent.id === "number") {
        // 模拟下载事件序列：Started → Progress → Finished
        callMockCallback(onEvent.id, { event: "Started", data: { contentLength: 1000 } });
        callMockCallback(onEvent.id, { event: "Progress", data: { chunkLength: 500 } });
        callMockCallback(onEvent.id, { event: "Progress", data: { chunkLength: 500 } });
        callMockCallback(onEvent.id, { event: "Finished", data: {} });
      }
      return undefined as T;
    }

    case "plugin:updater|close": {
      return undefined as T;
    }

    // Plugin: Process
    case "plugin:process|restart": {
      console.log("[IPC Mock] 🔄 模拟应用重启");
      return undefined as T;
    }

    // Plugin: App
    case "plugin:app|version": {
      return "0.7.0" as T;
    }

    // Plugin: Opener (openUrl)
    case "plugin:opener|open_url": {
      const url = getArg<string>(args, "url");
      console.log("[IPC Mock] 🔗 模拟打开 URL:", url);
      return undefined as T;
    }

    // ==========================================================================
    // 未匹配命令 (AC #4: 4.8)
    // ==========================================================================

    default:
      console.warn(`[IPC Mock] ⚠️ 未实现的命令: ${cmd}`, args);
      return null as T;
  }
}
