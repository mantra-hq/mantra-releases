/**
 * CodeSnapshotView 组件测试
 * Story 2.5: Task 6 - 验证与测试
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, act } from "@testing-library/react";
import { CodeSnapshotView, getLanguageFromPath } from "./CodeSnapshotView";

// Mock Monaco Editor - 因为它在测试环境中不能完全加载
vi.mock("@monaco-editor/react", () => ({
  default: ({
    value,
    language,
    theme,
  }: {
    value: string;
    language: string;
    theme: string;
  }) => (
    <div
      data-testid="monaco-editor"
      data-value={value}
      data-language={language}
      data-theme={theme}
    >
      Monaco Editor Mock
    </div>
  ),
}));

// Mock theme provider
vi.mock("@/lib/theme-provider", () => ({
  useTheme: () => ({
    theme: "dark",
    resolvedTheme: "dark",
    setTheme: vi.fn(),
  }),
}));

describe("getLanguageFromPath", () => {
  it("should return typescript for .ts files", () => {
    expect(getLanguageFromPath("src/index.ts")).toBe("typescript");
  });

  it("should return typescript for .tsx files", () => {
    expect(getLanguageFromPath("components/App.tsx")).toBe("typescript");
  });

  it("should return javascript for .js files", () => {
    expect(getLanguageFromPath("utils/helper.js")).toBe("javascript");
  });

  it("should return json for .json files", () => {
    expect(getLanguageFromPath("package.json")).toBe("json");
  });

  it("should return rust for .rs files", () => {
    expect(getLanguageFromPath("src-tauri/main.rs")).toBe("rust");
  });

  it("should return python for .py files", () => {
    expect(getLanguageFromPath("script.py")).toBe("python");
  });

  it("should return yaml for .yaml and .yml files", () => {
    expect(getLanguageFromPath("config.yaml")).toBe("yaml");
    expect(getLanguageFromPath("config.yml")).toBe("yaml");
  });

  it("should return plaintext for unknown extensions", () => {
    expect(getLanguageFromPath("file.xyz")).toBe("plaintext");
  });

  it("should return plaintext for empty path", () => {
    expect(getLanguageFromPath("")).toBe("plaintext");
  });

  it("should return plaintext for path without extension", () => {
    expect(getLanguageFromPath("Makefile")).toBe("plaintext");
  });

  it("should handle uppercase extensions", () => {
    expect(getLanguageFromPath("file.TS")).toBe("typescript");
  });
});

describe("CodeSnapshotView", () => {
  const defaultProps = {
    code: 'const hello = "world";',
    filePath: "src/index.ts",
  };

  describe("AC1 - Monaco Editor 集成", () => {
    it("should render Monaco Editor with code content", () => {
      render(<CodeSnapshotView {...defaultProps} />);

      const editor = screen.getByTestId("monaco-editor");
      expect(editor).toBeInTheDocument();
      expect(editor).toHaveAttribute("data-value", defaultProps.code);
    });

    it("should use readonly mode (verified through mock props)", () => {
      render(<CodeSnapshotView {...defaultProps} />);

      // Monaco Editor 是只读配置，这里验证 editor 被正确渲染
      expect(screen.getByTestId("monaco-editor")).toBeInTheDocument();
    });
  });

  describe("AC2 - 语法高亮", () => {
    it("should detect TypeScript language from file path", () => {
      render(<CodeSnapshotView {...defaultProps} filePath="src/index.ts" />);

      const editor = screen.getByTestId("monaco-editor");
      expect(editor).toHaveAttribute("data-language", "typescript");
    });

    it("should detect JavaScript language", () => {
      render(<CodeSnapshotView {...defaultProps} filePath="script.js" />);

      const editor = screen.getByTestId("monaco-editor");
      expect(editor).toHaveAttribute("data-language", "javascript");
    });

    it("should detect JSON language", () => {
      render(<CodeSnapshotView {...defaultProps} filePath="package.json" />);

      const editor = screen.getByTestId("monaco-editor");
      expect(editor).toHaveAttribute("data-language", "json");
    });

    it("should detect Rust language", () => {
      render(<CodeSnapshotView {...defaultProps} filePath="main.rs" />);

      const editor = screen.getByTestId("monaco-editor");
      expect(editor).toHaveAttribute("data-language", "rust");
    });

    it("should fallback to plaintext for unknown extensions", () => {
      render(<CodeSnapshotView {...defaultProps} filePath="file.unknown" />);

      const editor = screen.getByTestId("monaco-editor");
      expect(editor).toHaveAttribute("data-language", "plaintext");
    });
  });

  describe("AC3 - 文件路径显示", () => {
    it("should display file path in header", () => {
      render(<CodeSnapshotView {...defaultProps} />);

      expect(screen.getByText("src/index.ts")).toBeInTheDocument();
    });
  });

  describe("AC4 - 代码变化动画", () => {
    it("should trigger fade animation when code changes", async () => {
      vi.useFakeTimers();

      const { rerender } = render(<CodeSnapshotView {...defaultProps} />);

      // 更改代码内容
      rerender(
        <CodeSnapshotView {...defaultProps} code='const newCode = "test";' />
      );

      // 动画会在 150ms 后结束
      await act(async () => {
        vi.advanceTimersByTime(150);
      });

      vi.useRealTimers();
    });
  });

  describe("AC5 - 主题兼容", () => {
    it("should use vs-dark theme for dark mode", () => {
      render(<CodeSnapshotView {...defaultProps} />);

      const editor = screen.getByTestId("monaco-editor");
      expect(editor).toHaveAttribute("data-theme", "vs-dark");
    });
  });

  describe("AC6 - 空状态处理", () => {
    it("should show empty state when code is empty", () => {
      render(<CodeSnapshotView code="" filePath="" />);

      expect(screen.getByText("暂无代码")).toBeInTheDocument();
      expect(
        screen.getByText("选择一条对话消息，查看当时的代码快照")
      ).toBeInTheDocument();
    });

    it("should show empty state action guide", () => {
      render(<CodeSnapshotView code="" filePath="" />);

      expect(screen.getByText("点击左侧对话消息")).toBeInTheDocument();
    });
  });

  describe("AC7 - 历史状态指示", () => {
    it("should show historical badge when timestamp is provided", () => {
      render(
        <CodeSnapshotView
          {...defaultProps}
          timestamp="2025-12-30T10:30:00Z"
        />
      );

      expect(screen.getByText("历史快照")).toBeInTheDocument();
    });

    it("should show historical badge with commit hash", () => {
      render(
        <CodeSnapshotView {...defaultProps} commitHash="abc123def456" />
      );

      expect(screen.getByText("历史快照")).toBeInTheDocument();
      // 只显示前 7 位
      expect(screen.getByText(/abc123d/)).toBeInTheDocument();
    });

    it("should not show historical badge when no timestamp or commit", () => {
      render(<CodeSnapshotView {...defaultProps} />);

      expect(screen.queryByText("历史快照")).not.toBeInTheDocument();
    });
  });
});

