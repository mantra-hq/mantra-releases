import { clsx, type ClassValue } from "clsx"
import { twMerge } from "tailwind-merge"

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs))
}

/**
 * 格式化会话名称
 * 优先使用有效的 title，否则使用 ID 的简短形式
 *
 * @param id - 会话 ID
 * @param title - 会话标题（可选）
 * @returns 格式化后的会话名称
 */
export function formatSessionName(id: string, title?: string): string {
  // 优先使用 title，但如果 title 和 id 相同则忽略
  if (title && title !== id) {
    return title;
  }
  // 如果 ID 包含下划线或连字符，取最后一部分
  const parts = id.split(/[-_]/);
  if (parts.length > 1) {
    return parts[parts.length - 1].slice(0, 8);
  }
  // 否则取前 8 个字符
  return id.slice(0, 8);
}
