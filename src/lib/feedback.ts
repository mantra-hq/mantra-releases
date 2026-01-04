/**
 * feedback.ts - 统一交互反馈工具
 *
 * 提供一致的用户操作反馈机制，基于 Sonner Toast
 *
 * 设计规范：
 * - CRUD 操作：Toast ✅/❌
 * - 批量操作：Toast + 数量
 * - 复制操作：图标切换（由组件自行处理）
 */

import { toast } from "sonner";

/**
 * 统一反馈工具
 */
export const feedback = {
  // ============================================================
  // CRUD 操作
  // ============================================================

  /**
   * 保存成功
   * @param name - 可选，保存对象名称
   */
  saved: (name?: string) =>
    toast.success(name ? `「${name}」已保存` : "保存成功"),

  /**
   * 删除成功
   * @param name - 可选，删除对象名称
   */
  deleted: (name?: string) =>
    toast.success(name ? `「${name}」已删除` : "删除成功"),

  // ============================================================
  // 批量操作
  // ============================================================

  /**
   * 导入完成
   * @param count - 导入数量
   * @param type - 对象类型（如 "规则"、"文件"）
   */
  imported: (count: number, type: string) =>
    toast.success("导入完成", {
      description: `已导入 ${count} 条${type}`,
    }),

  /**
   * 导出完成
   * @param count - 导出数量
   * @param type - 对象类型
   */
  exported: (count: number, type: string) =>
    toast.success("导出完成", {
      description: `已导出 ${count} 条${type}`,
    }),

  /**
   * 重试结果反馈
   * @param success - 成功数量
   * @param failed - 失败数量
   */
  retryResult: (success: number, failed: number) => {
    if (failed === 0 && success > 0) {
      toast.success("重试成功", {
        description: `${success} 个文件已导入`,
      });
    } else if (success > 0 && failed > 0) {
      toast.warning("部分重试成功", {
        description: `成功 ${success} 个，仍失败 ${failed} 个`,
      });
    } else if (success === 0 && failed > 0) {
      toast.error("重试失败", {
        description: `${failed} 个文件仍然失败`,
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
    toast.error(`${action}失败`, {
      description: reason || "请稍后重试",
    }),
};

export default feedback;
