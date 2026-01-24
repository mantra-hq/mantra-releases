/**
 * Token Counter Tests
 * Story 10.2: Task 7.3
 *
 * 测试 Token 估算工具的准确性
 */

import { describe, it, expect } from "vitest";
import { estimateTokenCount, formatTokenCount } from "./token-counter";

describe("estimateTokenCount", () => {
  describe("空输入", () => {
    it("空字符串应返回 0", () => {
      expect(estimateTokenCount("")).toBe(0);
    });

    it("undefined 作为空处理应返回 0", () => {
      expect(estimateTokenCount(undefined as unknown as string)).toBe(0);
    });
  });

  describe("中文文本", () => {
    it("纯中文应按 ~1.5 token/字估算", () => {
      const text = "你好世界";
      const count = estimateTokenCount(text);
      // 4 个中文字符 * 1.5 = 6 tokens
      expect(count).toBe(6);
    });

    it("长中文文本估算", () => {
      const text = "这是一段较长的中文文本用于测试";
      const count = estimateTokenCount(text);
      // 15 个中文字符 * 1.5 = 22.5 → 23 (向上取整)
      expect(count).toBe(23);
    });
  });

  describe("英文文本", () => {
    it("纯英文单词应按 ~1.3 token/word 估算", () => {
      const text = "hello world";
      const count = estimateTokenCount(text);
      // 2 个英文单词 * 1.3 = 2.6
      // 1 个空格 / 4 = 0.25
      // 总计 2.85 → 3
      expect(count).toBe(3);
    });

    it("多单词英文文本", () => {
      const text = "The quick brown fox jumps";
      const count = estimateTokenCount(text);
      // 5 个单词 * 1.3 = 6.5
      // 4 个空格 / 4 = 1
      // 总计 7.5 → 8
      expect(count).toBe(8);
    });
  });

  describe("混合内容", () => {
    it("中英混合文本", () => {
      const text = "Hello 你好";
      const count = estimateTokenCount(text);
      // 1 个英文单词 * 1.3 = 1.3
      // 2 个中文字符 * 1.5 = 3
      // 1 个空格 / 4 = 0.25
      // 总计 4.55 → 5
      expect(count).toBe(5);
    });

    it("代码片段估算", () => {
      const text = "function foo() { return 42; }";
      const count = estimateTokenCount(text);
      // 有多个英文单词 (function, foo, return) 和符号
      expect(count).toBeGreaterThan(5);
      expect(count).toBeLessThan(20);
    });

    it("数字和符号", () => {
      const text = "123 + 456 = 789";
      const count = estimateTokenCount(text);
      // 主要是数字和符号，按 other chars 计算
      expect(count).toBeGreaterThan(0);
    });
  });

  describe("边界情况", () => {
    it("只有空格", () => {
      const text = "     ";
      const count = estimateTokenCount(text);
      // 5 个空格 / 4 = 1.25 → 2
      expect(count).toBe(2);
    });

    it("只有符号", () => {
      const text = "!@#$%^&*()";
      const count = estimateTokenCount(text);
      // 10 个符号 / 4 = 2.5 → 3
      expect(count).toBe(3);
    });
  });
});

describe("formatTokenCount", () => {
  describe("小数字", () => {
    it("小于 1000 应显示原数字", () => {
      expect(formatTokenCount(0)).toBe("0");
      expect(formatTokenCount(100)).toBe("100");
      expect(formatTokenCount(999)).toBe("999");
    });
  });

  describe("千级数字", () => {
    it("1000-9999 应显示一位小数的 k", () => {
      expect(formatTokenCount(1000)).toBe("1.0k");
      expect(formatTokenCount(1500)).toBe("1.5k");
      expect(formatTokenCount(2300)).toBe("2.3k");
      expect(formatTokenCount(9999)).toBe("10.0k");
    });
  });

  describe("万级及以上", () => {
    it("10000+ 应显示整数 k", () => {
      expect(formatTokenCount(10000)).toBe("10k");
      expect(formatTokenCount(15000)).toBe("15k");
      expect(formatTokenCount(100000)).toBe("100k");
    });
  });
});
