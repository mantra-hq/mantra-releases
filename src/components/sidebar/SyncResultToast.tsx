/**
 * SyncResultToast - 同步结果 Toast 通知
 * Story 2.19: Task 3
 *
 * 显示同步结果的 Toast 通知，包含新会话数和更新会话数
 */

import { toast } from "sonner";
import type { SyncResult } from "@/lib/project-ipc";

// Re-export for convenience
export type { SyncResult } from "@/lib/project-ipc";

/**
 * 显示同步结果 Toast
 * @param projectName 项目名称
 * @param result 同步结果（成功时）
 * @param error 错误（失败时）
 */
export function showSyncResult(
  projectName: string,
  result: SyncResult | null,
  error?: Error
): void {
  // 错误状态
  if (error) {
    toast.error("同步失败", {
      description: error.message || "请稍后重试",
    });
    return;
  }

  // 空结果
  if (!result) {
    toast.error("同步失败", {
      description: "未返回同步结果",
    });
    return;
  }

  const { new_sessions, updated_sessions } = result;
  const hasNewSessions = new_sessions.length > 0;
  const hasUpdates = updated_sessions.length > 0;

  // AC8: 无更新时显示「已是最新」
  if (!hasNewSessions && !hasUpdates) {
    toast.success("已是最新", {
      description: `「${projectName}」没有新内容`,
    });
    return;
  }

  // AC7: 显示同步结果，包含新会话数 + 更新会话数
  const parts: string[] = [];

  if (hasNewSessions) {
    parts.push(`发现 ${new_sessions.length} 个新会话`);
  }

  if (hasUpdates) {
    parts.push(`${updated_sessions.length} 个会话有新消息`);
  }

  toast.success("同步完成", {
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
