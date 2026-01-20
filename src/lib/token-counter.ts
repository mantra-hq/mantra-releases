/**
 * Token Counter - Token 估算工具
 * Story 10.2: Task 5
 *
 * 提供简单的 Token 估算算法，用于在前端显示消息的 Token 数量
 * 注意: 这是一个近似估算，不保证与实际 tokenizer 完全一致
 */

/**
 * 估算文本的 Token 数量
 *
 * 估算规则:
 * - 中文字符: ~1.5 tokens/字
 * - 英文单词: ~1.3 tokens/word
 * - 代码/符号: ~1 token/4 chars
 *
 * @param text - 要估算的文本
 * @returns 估算的 Token 数量
 */
export function estimateTokenCount(text: string): number {
  if (!text) return 0;

  // 匹配中文字符
  const chineseChars = (text.match(/[\u4e00-\u9fa5]/g) || []).length;

  // 匹配英文单词
  const englishWords = (text.match(/[a-zA-Z]+/g) || []).length;

  // 计算其他字符 (数字、符号、空白等)
  const englishCharsTotal = (text.match(/[a-zA-Z]/g) || []).length;
  const otherChars = text.length - chineseChars - englishCharsTotal;

  // 估算 Token 数量
  const estimate =
    chineseChars * 1.5 + // 中文字符
    englishWords * 1.3 + // 英文单词
    otherChars / 4; // 其他字符

  return Math.ceil(estimate);
}

/**
 * 格式化 Token 数量显示
 *
 * @param count - Token 数量
 * @returns 格式化后的字符串 (如 "1.2k")
 */
export function formatTokenCount(count: number): string {
  if (count < 1000) {
    return count.toString();
  }
  if (count < 10000) {
    return `${(count / 1000).toFixed(1)}k`;
  }
  return `${Math.round(count / 1000)}k`;
}
