/**
 * Compress Exporter - 压缩会话导出工具
 * Story 10.7: Task 1
 *
 * 提供压缩后会话内容的导出功能
 * 支持 JSONL、Markdown 格式导出和剪贴板复制
 */

import type { PreviewMessage } from "@/hooks/useCompressState";
import type { TokenStats } from "@/components/compress/TokenStatistics";
import { getMessageDisplayContent } from "@/lib/message-utils";
import { formatTokenCount } from "@/lib/token-counter";

/**
 * 导出为 JSONL 格式
 * AC3: 生成符合 Claude Code 格式的 JSONL 文件
 *
 * @param messages - 预览消息列表
 * @returns JSONL 格式字符串，每行一条消息
 */
export function exportToJsonl(messages: PreviewMessage[]): string {
  return messages
    // 过滤掉删除的消息
    .filter((m) => m.operation !== "delete")
    .map((m) => {
      const content = getMessageDisplayContent(m.message.content);
      return JSON.stringify({
        role: m.message.role,
        content,
      });
    })
    .join("\n");
}

/**
 * 导出为 Markdown 格式
 * AC4/AC5: 生成人类可读的 Markdown 格式
 *
 * @param messages - 预览消息列表
 * @param stats - Token 统计数据
 * @param sessionName - 会话名称
 * @returns Markdown 格式字符串
 */
export function exportToMarkdown(
  messages: PreviewMessage[],
  stats: TokenStats,
  sessionName?: string
): string {
  const lines: string[] = [];

  // 标题
  lines.push(`# ${sessionName || "Compressed Session"}`);
  lines.push("");

  // 元信息
  const savedPercentage = Math.round(stats.savedPercentage);
  lines.push(
    `> Original: ${formatTokenCount(stats.originalTotal)} tokens → Compressed: ${formatTokenCount(stats.compressedTotal)} tokens (saved ${savedPercentage}%)`
  );
  lines.push(`> Exported at: ${new Date().toISOString()}`);
  lines.push("");
  lines.push("---");
  lines.push("");

  // 消息内容
  messages
    .filter((m) => m.operation !== "delete")
    .forEach((m) => {
      const roleLabel = m.message.role === "user" ? "User" : "Assistant";
      lines.push(`## ${roleLabel}`);
      lines.push("");
      lines.push(getMessageDisplayContent(m.message.content));
      lines.push("");
      lines.push("---");
      lines.push("");
    });

  return lines.join("\n");
}

/**
 * 获取导出内容 (用于剪贴板复制)
 * AC6: 将压缩后内容复制到剪贴板 (Markdown 格式)
 *
 * @param messages - 预览消息列表
 * @returns Markdown 格式的纯文本
 */
export function getExportContent(messages: PreviewMessage[]): string {
  const lines: string[] = [];

  messages
    .filter((m) => m.operation !== "delete")
    .forEach((m) => {
      const roleLabel = m.message.role === "user" ? "User" : "Assistant";
      lines.push(`## ${roleLabel}`);
      lines.push("");
      lines.push(getMessageDisplayContent(m.message.content));
      lines.push("");
      lines.push("---");
      lines.push("");
    });

  return lines.join("\n").trim();
}

/**
 * 格式化导出文件名
 *
 * @param sessionName - 会话名称
 * @param format - 文件格式 (jsonl | md)
 * @returns 格式化后的文件名
 */
export function formatExportFilename(
  sessionName: string | undefined,
  format: "jsonl" | "md"
): string {
  // 清理会话名称，移除不安全字符
  const safeName = (sessionName || "session")
    .replace(/[<>:"/\\|?*]/g, "-")
    .replace(/\s+/g, "-")
    .slice(0, 50);

  // 生成时间戳
  const now = new Date();
  const timestamp = now
    .toISOString()
    .replace(/[:.]/g, "-")
    .slice(0, 19);

  return `${safeName}-compressed-${timestamp}.${format}`;
}
