/**
 * ImportWizard 测试文件
 * Story 2.9: Task 1
 * Story 2.23: Quick Navigation (requires Router context)
 *
 * 测试导入向导 Modal 的核心功能
 */

import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { MemoryRouter } from "react-router-dom";
import { ImportWizard } from "./ImportWizard";
import { useImportStore } from "@/stores";
import { act } from "@testing-library/react";

// Mock Tauri IPC
vi.mock("@/lib/import-ipc", () => ({
  scanLogDirectory: vi.fn(),
  selectLogFiles: vi.fn(),
  parseLogFiles: vi.fn(),
  importSessionsWithProgress: vi.fn(),
  cancelImport: vi.fn(),
  getDefaultPaths: vi.fn().mockResolvedValue({
    claude: "~/.claude",
    gemini: "~/.gemini",
    cursor: "~/.config/Cursor",
    codex: "~/.codex",
  }),
}));

// Mock project-ipc
vi.mock("@/lib/project-ipc", () => ({
  getImportedProjectPaths: vi.fn().mockResolvedValue([]),
  getImportedSessionIds: vi.fn().mockResolvedValue([]),
  getProject: vi.fn().mockResolvedValue(null),
}));

/**
 * 包装组件以提供 Router 上下文
 */
function renderWithRouter(ui: React.ReactElement) {
  return render(
    <MemoryRouter>
      {ui}
    </MemoryRouter>
  );
}

describe("ImportWizard", () => {
  beforeEach(() => {
    // 重置 store
    act(() => {
      useImportStore.getState().reset();
    });
  });
  // Task 1.1 & 1.2: Modal 容器
  describe("Modal Container", () => {
    it("renders dialog when open", () => {
      renderWithRouter(<ImportWizard open={true} onOpenChange={vi.fn()} />);

      expect(screen.getByRole("dialog")).toBeInTheDocument();
      expect(screen.getByText("导入日志")).toBeInTheDocument();
    });

    it("does not render dialog when closed", () => {
      renderWithRouter(<ImportWizard open={false} onOpenChange={vi.fn()} />);

      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });

    it("calls onOpenChange when close button is clicked", () => {
      const onOpenChange = vi.fn();
      renderWithRouter(<ImportWizard open={true} onOpenChange={onOpenChange} />);

      // 点击关闭按钮
      const closeButton = screen.getByRole("button", { name: /close/i });
      fireEvent.click(closeButton);

      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  // Task 1.3: 步骤指示器
  describe("Step Indicator", () => {
    it("shows all four steps", () => {
      renderWithRouter(<ImportWizard open={true} onOpenChange={vi.fn()} />);

      expect(screen.getByText("选择来源")).toBeInTheDocument();
      expect(screen.getByText("选择文件")).toBeInTheDocument();
      expect(screen.getByText("导入中")).toBeInTheDocument();
      expect(screen.getByText("完成")).toBeInTheDocument();
    });

    it("marks current step as active", () => {
      renderWithRouter(
        <ImportWizard open={true} onOpenChange={vi.fn()} initialStep="source" />
      );

      const sourceStep = screen.getByTestId("step-source");
      expect(sourceStep).toHaveAttribute("data-state", "active");
    });

    it("marks completed steps correctly", () => {
      renderWithRouter(
        <ImportWizard open={true} onOpenChange={vi.fn()} initialStep="files" />
      );

      const sourceStep = screen.getByTestId("step-source");
      expect(sourceStep).toHaveAttribute("data-state", "completed");

      const filesStep = screen.getByTestId("step-files");
      expect(filesStep).toHaveAttribute("data-state", "active");
    });
  });

  // Task 1.4: 步骤 1 - 选择来源
  describe("Step 1: Source Selection", () => {
    it("shows source selection content on first step", () => {
      renderWithRouter(
        <ImportWizard open={true} onOpenChange={vi.fn()} initialStep="source" />
      );

      expect(screen.getByTestId("source-selector")).toBeInTheDocument();
    });
  });

  // Task 1.5: 步骤 2 - 选择文件
  describe("Step 2: File Selection", () => {
    it("shows file selector content on second step", () => {
      renderWithRouter(
        <ImportWizard open={true} onOpenChange={vi.fn()} initialStep="files" />
      );

      expect(screen.getByTestId("file-selector")).toBeInTheDocument();
    });
  });

  // Task 1.6: 步骤 3 - 导入进度
  describe("Step 3: Import Progress", () => {
    it("shows progress content on third step", () => {
      renderWithRouter(
        <ImportWizard
          open={true}
          onOpenChange={vi.fn()}
          initialStep="progress"
        />
      );

      expect(screen.getByTestId("import-progress")).toBeInTheDocument();
    });
  });

  // Task 1.7: 步骤 4 - 完成确认
  describe("Step 4: Completion", () => {
    it("shows completion content on final step", () => {
      renderWithRouter(
        <ImportWizard
          open={true}
          onOpenChange={vi.fn()}
          initialStep="complete"
        />
      );

      expect(screen.getByTestId("import-complete")).toBeInTheDocument();
    });
  });

  // 导航测试
  describe("Navigation", () => {
    it("has back button disabled on first step", () => {
      renderWithRouter(
        <ImportWizard open={true} onOpenChange={vi.fn()} initialStep="source" />
      );

      const backButton = screen.queryByTestId("back-button");
      // 第一步不显示返回按钮或被禁用
      expect(
        backButton === null || backButton.hasAttribute("disabled")
      ).toBeTruthy();
    });

    it("has next button on first step", () => {
      renderWithRouter(
        <ImportWizard open={true} onOpenChange={vi.fn()} initialStep="source" />
      );

      expect(screen.getByTestId("next-button")).toBeInTheDocument();
    });
  });

  // 无障碍测试
  describe("Accessibility", () => {
    it("has proper aria-label on dialog", () => {
      renderWithRouter(<ImportWizard open={true} onOpenChange={vi.fn()} />);

      const dialog = screen.getByRole("dialog");
      expect(dialog).toHaveAttribute("aria-labelledby");
    });

    it("has aria-current on active step", () => {
      renderWithRouter(
        <ImportWizard open={true} onOpenChange={vi.fn()} initialStep="files" />
      );

      const filesStep = screen.getByTestId("step-files");
      expect(filesStep).toHaveAttribute("aria-current", "step");
    });
  });
});
