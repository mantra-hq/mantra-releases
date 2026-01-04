/**
 * SyncResultToast - 同步结果 Toast 通知
 * Story 2.19: Task 3
 * Story 2.26: 国际化支持
 *
 * 显示同步结果的 Toast 通知，包含新会话数和更新会话数
 */

import { toast } from "sonner";
import i18n from "@/i18n";
import type { SyncResult } from "@/lib/project-ipc";

// Re-export for convenience
export type { SyncResult } from "@/lib/project-ipc";

/**
 * 显示同步结果 Toast
 * @param projectName 项目名称
 * @param result 同步结果（成功时）
 * @param error 错误（失败时）
 * @param isForceSync 是否为强制重新解析
 */
export function showSyncResult(
  projectName: string,
  result: SyncResult | null,
  error?: Error,
  isForceSync?: boolean
): void {
  const t = i18n.t.bind(i18n);

  // 错误状态
  if (error) {
    toast.error(isForceSync ? t("sync.reParseFailed") : t("sync.syncFailed"), {
      description: error.message || t("sync.retryLater"),
    });
    return;
  }

  // 空结果
  if (!result) {
    toast.error(isForceSync ? t("sync.reParseFailed") : t("sync.syncFailed"), {
      description: t("sync.noResult"),
    });
    return;
  }

  const { new_sessions, updated_sessions } = result;
  const hasNewSessions = new_sessions.length > 0;
  const hasUpdates = updated_sessions.length > 0;

  // 强制重新解析模式
  if (isForceSync) {
    if (!hasNewSessions && !hasUpdates) {
      toast.success(t("sync.reParseComplete"), {
        description: t("sync.allLatest", { name: projectName }),
      });
      return;
    }

    const parts: string[] = [];
    if (hasNewSessions) {
      parts.push(t("sync.foundNewSessions", { count: new_sessions.length }));
    }
    if (hasUpdates) {
      parts.push(t("sync.reparseSessionsCount", { count: updated_sessions.length }));
    }

    toast.success(t("sync.reParseComplete"), {
      description: (
        <div className="flex flex-col gap-1">
          <div className="font-medium">🔃 {projectName}</div>
          {parts.map((part, index) => (
            <div key={index}>• {part}</div>
          ))}
        </div>
      ),
    });
    return;
  }

  // AC8: 无更新时显示「已是最新」
  if (!hasNewSessions && !hasUpdates) {
    toast.success(t("sync.upToDate"), {
      description: t("sync.noNewContent", { name: projectName }),
    });
    return;
  }

  // AC7: 显示同步结果，包含新会话数 + 更新会话数
  const parts: string[] = [];

  if (hasNewSessions) {
    parts.push(t("sync.foundNewSessions", { count: new_sessions.length }));
  }

  if (hasUpdates) {
    parts.push(t("sync.sessionsUpdated", { count: updated_sessions.length }));
  }

  toast.success(t("sync.syncComplete"), {
    description: (
      <div className="flex flex-col gap-1">
        <div className="font-medium">📁 {projectName}</div>
        {parts.map((part, index) => (
          <div key={index}>• {part}</div>
        ))}
      </div>
    ),
  });
}
