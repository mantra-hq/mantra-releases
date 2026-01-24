/**
 * SourceSelector 测试文件
 * Story 2.9: Task 2
 *
 * 测试来源选择组件
 */

import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { SourceSelector, type ImportSource } from "./SourceSelector";

// Mock Tauri IPC
vi.mock("@/lib/import-ipc", () => ({
  getDefaultPaths: vi.fn().mockResolvedValue({
    claude: "~/.claude",
    gemini: "~/.gemini",
    cursor: "~/.config/Cursor",
    codex: "~/.codex",
  }),
}));

const renderSourceSelector = async (
  value: ImportSource | null,
  onChange: (value: ImportSource) => void
) => {
  render(<SourceSelector value={value} onChange={onChange} />);
  await waitFor(() => {
    expect(screen.getByText("~/.claude")).toBeInTheDocument();
  });
};

describe("SourceSelector", () => {
  // Task 2.2: 显示三种来源卡片
  describe("Source Cards", () => {
    it("displays all three source options", async () => {
      await renderSourceSelector(null, vi.fn());

      expect(screen.getByText("Claude Code")).toBeInTheDocument();
      expect(screen.getByText("Gemini CLI")).toBeInTheDocument();
      expect(screen.getByText("Cursor")).toBeInTheDocument();
    });

    // Task 2.3: 每个卡片显示图标、名称、默认路径说明
    it("displays default paths for each source", async () => {
      await renderSourceSelector(null, vi.fn());
      expect(screen.getByText("~/.gemini")).toBeInTheDocument();
      expect(screen.getByText("~/.config/Cursor")).toBeInTheDocument();
    });

    it("renders source icons", async () => {
      await renderSourceSelector(null, vi.fn());

      // 检查每个来源卡片都有图标区域
      const claudeCard = screen.getByTestId("source-card-claude");
      const geminiCard = screen.getByTestId("source-card-gemini");
      const cursorCard = screen.getByTestId("source-card-cursor");

      expect(claudeCard.querySelector('[data-slot="source-icon"]')).toBeInTheDocument();
      expect(geminiCard.querySelector('[data-slot="source-icon"]')).toBeInTheDocument();
      expect(cursorCard.querySelector('[data-slot="source-icon"]')).toBeInTheDocument();
    });
  });

  // Task 2.4: 选中状态高亮
  describe("Selection State", () => {
    it("highlights selected source card", async () => {
      await renderSourceSelector("claude", vi.fn());

      const claudeCard = screen.getByTestId("source-card-claude");
      expect(claudeCard).toHaveAttribute("data-selected", "true");
    });

    it("calls onChange when source is selected", async () => {
      const onChange = vi.fn();
      await renderSourceSelector(null, onChange);

      const claudeCard = screen.getByTestId("source-card-claude");
      fireEvent.click(claudeCard);

      expect(onChange).toHaveBeenCalledWith("claude");
    });

    it("does not highlight unselected sources", async () => {
      await renderSourceSelector("claude", vi.fn());

      const geminiCard = screen.getByTestId("source-card-gemini");
      const cursorCard = screen.getByTestId("source-card-cursor");

      expect(geminiCard).toHaveAttribute("data-selected", "false");
      expect(cursorCard).toHaveAttribute("data-selected", "false");
    });
  });

  // Task 2.5: 启用状态测试
  describe("Enabled Sources", () => {
    it("all sources are enabled", async () => {
      await renderSourceSelector(null, vi.fn());

      const claudeCard = screen.getByTestId("source-card-claude");
      const geminiCard = screen.getByTestId("source-card-gemini");
      const cursorCard = screen.getByTestId("source-card-cursor");

      expect(claudeCard).toHaveAttribute("data-disabled", "false");
      expect(geminiCard).toHaveAttribute("data-disabled", "false");
      expect(cursorCard).toHaveAttribute("data-disabled", "false");
    });

    it("calls onChange when clicking Gemini source", async () => {
      const onChange = vi.fn();
      await renderSourceSelector(null, onChange);

      const geminiCard = screen.getByTestId("source-card-gemini");
      fireEvent.click(geminiCard);

      expect(onChange).toHaveBeenCalledWith("gemini");
    });

    it("calls onChange when clicking Cursor source", async () => {
      const onChange = vi.fn();
      await renderSourceSelector(null, onChange);

      const cursorCard = screen.getByTestId("source-card-cursor");
      fireEvent.click(cursorCard);

      expect(onChange).toHaveBeenCalledWith("cursor");
    });
  });

  // 无障碍测试
  describe("Accessibility", () => {
    it("has role radiogroup", async () => {
      await renderSourceSelector(null, vi.fn());
      expect(screen.getByRole("radiogroup")).toBeInTheDocument();
    });

    it("has proper aria-label", async () => {
      await renderSourceSelector(null, vi.fn());
      const radiogroup = screen.getByRole("radiogroup");
      expect(radiogroup).toHaveAttribute("aria-label", "选择导入来源");
    });

    it("marks selected source with aria-checked", async () => {
      await renderSourceSelector("claude", vi.fn());
      const claudeCard = screen.getByTestId("source-card-claude");
      expect(claudeCard).toHaveAttribute("aria-checked", "true");
    });
  });
});
