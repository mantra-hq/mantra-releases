/**
 * ImportProgress 测试文件
 * Story 2.9: Task 4
 * Story 2.23: Cancel Import UI
 *
 * 测试导入进度展示组件
 */

import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { ImportProgress, type ImportProgressData, type ImportError } from "./ImportProgress";

/** 测试用进度数据 */
const mockProgress: ImportProgressData = {
  current: 5,
  total: 10,
  currentFile: "conversation_123.json",
  successCount: 4,
  failureCount: 1,
};

/** 测试用错误数据 */
const mockErrors: ImportError[] = [
  {
    filePath: "/path/to/error_file.json",
    error: "invalid_json",
    message: "JSON 格式无效",
  },
];

describe("ImportProgress", () => {
  // Task 4.2: 总体进度条
  describe("Progress Bar", () => {
    it("displays progress bar", () => {
      render(<ImportProgress progress={mockProgress} errors={[]} />);

      expect(screen.getByRole("progressbar")).toBeInTheDocument();
    });

    it("shows correct progress percentage", () => {
      render(<ImportProgress progress={mockProgress} errors={[]} />);

      // 5/10 = 50%
      const progressbar = screen.getByRole("progressbar");
      expect(progressbar).toHaveAttribute("aria-valuenow", "50");
    });

    it("displays progress text", () => {
      render(<ImportProgress progress={mockProgress} errors={[]} />);

      expect(screen.getByText("5 / 10")).toBeInTheDocument();
    });
  });

  // Task 4.3: 当前处理文件名
  describe("Current File", () => {
    it("displays current file name", () => {
      render(<ImportProgress progress={mockProgress} errors={[]} />);

      expect(screen.getByText("conversation_123.json")).toBeInTheDocument();
    });

    it("shows processing label", () => {
      render(<ImportProgress progress={mockProgress} errors={[]} />);

      expect(screen.getByText(/正在处理/)).toBeInTheDocument();
    });
  });

  // Task 4.4: 成功/失败计数
  describe("Statistics", () => {
    it("displays success count", () => {
      render(<ImportProgress progress={mockProgress} errors={[]} />);

      expect(screen.getByTestId("success-count")).toHaveTextContent("4");
    });

    it("displays failure count", () => {
      render(<ImportProgress progress={mockProgress} errors={[]} />);

      expect(screen.getByTestId("failure-count")).toHaveTextContent("1");
    });

    it("shows success label", () => {
      render(<ImportProgress progress={mockProgress} errors={[]} />);

      expect(screen.getByText("成功")).toBeInTheDocument();
    });

    it("shows failure label", () => {
      render(<ImportProgress progress={mockProgress} errors={[]} />);

      expect(screen.getByText("失败")).toBeInTheDocument();
    });
  });

  // Task 4.5: 错误文件列表
  describe("Error List", () => {
    it("shows errors section when there are errors", () => {
      render(<ImportProgress progress={mockProgress} errors={mockErrors} />);

      expect(screen.getByTestId("error-list")).toBeInTheDocument();
    });

    it("hides errors section when no errors", () => {
      render(<ImportProgress progress={mockProgress} errors={[]} />);

      expect(screen.queryByTestId("error-list")).not.toBeInTheDocument();
    });

    it("displays error file path", () => {
      render(<ImportProgress progress={mockProgress} errors={mockErrors} />);

      expect(screen.getByText("error_file.json")).toBeInTheDocument();
    });

    it("displays error message", () => {
      render(<ImportProgress progress={mockProgress} errors={mockErrors} />);

      expect(screen.getByText("JSON 格式无效")).toBeInTheDocument();
    });

    it("can expand/collapse error list", () => {
      const errors: ImportError[] = [
        ...mockErrors,
        { filePath: "/path/to/file2.json", error: "parse_error", message: "解析失败" },
      ];

      render(<ImportProgress progress={mockProgress} errors={errors} />);

      // 错误列表应该可以折叠
      const toggleButton = screen.getByTestId("error-toggle");
      expect(toggleButton).toBeInTheDocument();

      // 点击展开/折叠
      fireEvent.click(toggleButton);
    });
  });

  // 无障碍测试
  describe("Accessibility", () => {
    it("has aria-valuenow on progress bar", () => {
      render(<ImportProgress progress={mockProgress} errors={[]} />);

      const progressbar = screen.getByRole("progressbar");
      expect(progressbar).toHaveAttribute("aria-valuenow");
    });

    it("has aria-valuemin and aria-valuemax", () => {
      render(<ImportProgress progress={mockProgress} errors={[]} />);

      const progressbar = screen.getByRole("progressbar");
      expect(progressbar).toHaveAttribute("aria-valuemin", "0");
      expect(progressbar).toHaveAttribute("aria-valuemax", "100");
    });
  });

  // 边界情况
  describe("Edge Cases", () => {
    it("handles zero total gracefully", () => {
      const zeroProgress = { ...mockProgress, total: 0, current: 0 };
      render(<ImportProgress progress={zeroProgress} errors={[]} />);

      // 不应该崩溃
      expect(screen.getByRole("progressbar")).toBeInTheDocument();
    });

    it("handles completed state", () => {
      const completedProgress = {
        ...mockProgress,
        current: 10,
        total: 10,
        currentFile: "",
      };
      render(<ImportProgress progress={completedProgress} errors={[]} />);

      // 100% 完成
      const progressbar = screen.getByRole("progressbar");
      expect(progressbar).toHaveAttribute("aria-valuenow", "100");
    });
  });

  // Story 2.23: 取消导入按钮
  describe("Cancel Import Button", () => {
    it("shows cancel button when onCancel is provided", () => {
      render(
        <ImportProgress
          progress={mockProgress}
          errors={[]}
          onCancel={vi.fn()}
        />
      );

      expect(screen.getByTestId("cancel-import-button")).toBeInTheDocument();
    });

    it("does not show cancel button when onCancel is not provided", () => {
      render(<ImportProgress progress={mockProgress} errors={[]} />);

      expect(screen.queryByTestId("cancel-import-button")).not.toBeInTheDocument();
    });

    it("calls onCancel when confirmed in dialog", () => {
      const onCancel = vi.fn();
      render(
        <ImportProgress
          progress={mockProgress}
          errors={[]}
          onCancel={onCancel}
        />
      );

      // 点击取消按钮打开对话框
      fireEvent.click(screen.getByTestId("cancel-import-button"));

      // 点击确认取消
      fireEvent.click(screen.getByText("确认取消"));
      expect(onCancel).toHaveBeenCalled();
    });

    it("disables cancel button when isCancelling is true", () => {
      render(
        <ImportProgress
          progress={mockProgress}
          errors={[]}
          onCancel={vi.fn()}
          isCancelling={true}
        />
      );

      const button = screen.getByTestId("cancel-import-button");
      expect(button).toBeDisabled();
    });

    it("shows loading spinner when cancelling", () => {
      render(
        <ImportProgress
          progress={mockProgress}
          errors={[]}
          onCancel={vi.fn()}
          isCancelling={true}
        />
      );

      // i18n key: import.cancelling -> "正在取消"
      expect(screen.getByText("正在取消")).toBeInTheDocument();
    });
  });
});
