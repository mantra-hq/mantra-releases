/**
 * TokenCompareBar Component Tests
 * Story 10.6: Task 7.2
 *
 * 测试 Token 对比进度条组件
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { TokenCompareBar } from "./TokenCompareBar";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "compress.tokenStats.original": "Original",
        "compress.tokenStats.compressed": "Compressed",
      };
      return translations[key] || key;
    },
  }),
}));

describe("TokenCompareBar", () => {
  describe("基础渲染 (AC #2)", () => {
    it("应渲染双行进度条", () => {
      render(
        <TokenCompareBar
          originalTokens={1000}
          compressedTokens={750}
        />
      );

      expect(screen.getByTestId("token-compare-bar")).toBeInTheDocument();
      expect(screen.getByTestId("original-bar")).toBeInTheDocument();
      expect(screen.getByTestId("compressed-bar")).toBeInTheDocument();
    });

    it("应显示原始和压缩后标签", () => {
      render(
        <TokenCompareBar
          originalTokens={1000}
          compressedTokens={750}
        />
      );

      expect(screen.getByText("Original")).toBeInTheDocument();
      expect(screen.getByText("Compressed")).toBeInTheDocument();
    });
  });

  describe("进度条宽度计算 (AC #2)", () => {
    it("原始进度条应为 100%", () => {
      render(
        <TokenCompareBar
          originalTokens={1000}
          compressedTokens={750}
        />
      );

      const originalBar = screen.getByTestId("original-bar");
      expect(originalBar).toHaveStyle({ width: "100%" });
    });

    it("压缩后进度条宽度应正确计算", () => {
      render(
        <TokenCompareBar
          originalTokens={1000}
          compressedTokens={750}
        />
      );

      const compressedBar = screen.getByTestId("compressed-bar");
      expect(compressedBar).toHaveStyle({ width: "75%" });
    });

    it("压缩后 Token 为 0 时进度条宽度应为 0%", () => {
      render(
        <TokenCompareBar
          originalTokens={1000}
          compressedTokens={0}
        />
      );

      const compressedBar = screen.getByTestId("compressed-bar");
      expect(compressedBar).toHaveStyle({ width: "0%" });
    });

    it("压缩后 Token 大于原始时进度条宽度应为 100%", () => {
      render(
        <TokenCompareBar
          originalTokens={1000}
          compressedTokens={1500}
        />
      );

      const compressedBar = screen.getByTestId("compressed-bar");
      // 使用 Math.min(100, percentage) 限制最大为 100%
      expect(compressedBar).toHaveStyle({ width: "100%" });
    });
  });

  describe("百分比标签显示 (AC #2)", () => {
    it("节省时应显示负百分比", () => {
      render(
        <TokenCompareBar
          originalTokens={1000}
          compressedTokens={750}
        />
      );

      expect(screen.getByText("-25%")).toBeInTheDocument();
    });

    it("无节省时应显示 0%", () => {
      render(
        <TokenCompareBar
          originalTokens={1000}
          compressedTokens={1000}
        />
      );

      expect(screen.getByText("0%")).toBeInTheDocument();
    });

    it("增加时应显示 0% (不显示正数)", () => {
      render(
        <TokenCompareBar
          originalTokens={1000}
          compressedTokens={1200}
        />
      );

      // savedPercentage 为负数时，显示 0%
      expect(screen.getByText("0%")).toBeInTheDocument();
    });
  });

  describe("CSS transition 类 (AC #3)", () => {
    it("进度条应有 transition 类", () => {
      render(
        <TokenCompareBar
          originalTokens={1000}
          compressedTokens={750}
        />
      );

      const compressedBar = screen.getByTestId("compressed-bar");
      expect(compressedBar).toHaveClass("transition-all", "duration-300");
    });
  });

  describe("边界情况", () => {
    it("原始 Token 为 0 时应正常渲染", () => {
      render(
        <TokenCompareBar
          originalTokens={0}
          compressedTokens={0}
        />
      );

      expect(screen.getByTestId("token-compare-bar")).toBeInTheDocument();
      // 原始为 0 时，percentage 应为 100%
      const compressedBar = screen.getByTestId("compressed-bar");
      expect(compressedBar).toHaveStyle({ width: "100%" });
    });

    it("应支持自定义 className", () => {
      render(
        <TokenCompareBar
          originalTokens={1000}
          compressedTokens={750}
          className="custom-class"
        />
      );

      const container = screen.getByTestId("token-compare-bar");
      expect(container).toHaveClass("custom-class");
    });
  });
});
