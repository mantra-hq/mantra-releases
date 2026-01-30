/**
 * feedback.ts - 统一交互反馈工具
 * Story 2-26: 国际化支持
 *
 * 提供一致的用户操作反馈机制，基于 Sonner Toast
 *
 * 设计规范：
 * - CRUD 操作：Toast ✅/❌
 * - 批量操作：Toast + 数量
 * - 复制操作：图标切换（由组件自行处理）
 */

import { toast } from "sonner";
import i18n from "@/i18n";

/**
 * 获取翻译文本的辅助函数
 */
const t = (key: string, options?: Record<string, unknown>) =>
  i18n.t(key, options);

/**
 * 统一反馈工具
 */
export const feedback = {
  // ============================================================
  // 通用反馈
  // ============================================================

  /**
   * 通用成功消息
   * Story 11.6: Hub 模块需要通用成功反馈
   * @param message - 成功消息
   */
  success: (message: string) => toast.success(message),

  // ============================================================
  // CRUD 操作
  // ============================================================

  /**
   * 保存成功
   * @param name - 可选，保存对象名称
   */
  saved: (name?: string) =>
    toast.success(name ? `「${name}」${t("feedback.saved")}` : t("feedback.saved")),

  /**
   * 删除成功
   * @param name - 可选，删除对象名称
   */
  deleted: (name?: string) =>
    toast.success(name ? `「${name}」${t("feedback.deleted")}` : t("feedback.deleted")),

  /**
   * 复制成功
   * Story 2.28: AC3
   * @param message - 可选，自定义消息
   */
  copied: (message?: string) =>
    toast.success(message || t("feedback.copiedToClipboard")),

  // ============================================================
  // 批量操作
  // ============================================================

  /**
   * 导入完成
   * @param count - 导入数量
   * @param type - 对象类型（如 "规则"、"文件"）
   */
  imported: (count: number, type: string) =>
    toast.success(t("feedback.importComplete"), {
      description: t("feedback.importedCount", { count, type }),
    }),

  /**
   * 导出完成
   * @param count - 导出数量
   * @param type - 对象类型
   */
  exported: (count: number, type: string) =>
    toast.success(t("feedback.exportComplete"), {
      description: t("feedback.exportedCount", { count, type }),
    }),

  /**
   * 重试结果反馈
   * @param success - 成功数量
   * @param failed - 失败数量
   */
  retryResult: (success: number, failed: number) => {
    if (failed === 0 && success > 0) {
      toast.success(t("feedback.retrySuccess"), {
        description: t("feedback.filesImported", { success }),
      });
    } else if (success > 0 && failed > 0) {
      toast.warning(t("feedback.partialRetrySuccess"), {
        description: t("feedback.partialRetryResult", { success, failed }),
      });
    } else if (success === 0 && failed > 0) {
      toast.error(t("feedback.retryFailed"), {
        description: t("feedback.stillFailed", { failed }),
      });
    }
    // success === 0 && failed === 0 时不显示（无操作）
  },

  // ============================================================
  // 错误反馈
  // ============================================================

  /**
   * 操作失败
   * @param action - 操作名称（如 "保存"、"导入"）
   * @param reason - 可选，失败原因
   */
  error: (action: string, reason?: string) =>
    toast.error(t("feedback.actionFailed", { action }), {
      description: reason || t("feedback.retryLater"),
    }),
};

export default feedback;
