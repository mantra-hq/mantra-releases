/**
 * SourceSelector 测试文件
 * Story 2.9: Task 2
 *
 * 测试来源选择组件
 */

import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { SourceSelector } from "./SourceSelector";

describe("SourceSelector", () => {
  // Task 2.2: 显示三种来源卡片
  describe("Source Cards", () => {
    it("displays all three source options", () => {
      render(<SourceSelector value={null} onChange={vi.fn()} />);

      expect(screen.getByText("Claude Code")).toBeInTheDocument();
      expect(screen.getByText("Gemini CLI")).toBeInTheDocument();
      expect(screen.getByText("Cursor")).toBeInTheDocument();
    });

    // Task 2.3: 每个卡片显示图标、名称、默认路径说明
    it("displays default paths for each source", () => {
      render(<SourceSelector value={null} onChange={vi.fn()} />);

      expect(screen.getByText("~/.claude/projects")).toBeInTheDocument();
      expect(screen.getByText("~/.gemini/project_temp/chats")).toBeInTheDocument();
      expect(screen.getByText("~/.cursor/projects")).toBeInTheDocument();
    });

    it("renders source icons", () => {
      render(<SourceSelector value={null} onChange={vi.fn()} />);

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
    it("highlights selected source card", () => {
      render(<SourceSelector value="claude" onChange={vi.fn()} />);

      const claudeCard = screen.getByTestId("source-card-claude");
      expect(claudeCard).toHaveAttribute("data-selected", "true");
    });

    it("calls onChange when source is selected", () => {
      const onChange = vi.fn();
      render(<SourceSelector value={null} onChange={onChange} />);

      const claudeCard = screen.getByTestId("source-card-claude");
      fireEvent.click(claudeCard);

      expect(onChange).toHaveBeenCalledWith("claude");
    });

    it("does not highlight unselected sources", () => {
      render(<SourceSelector value="claude" onChange={vi.fn()} />);

      const geminiCard = screen.getByTestId("source-card-gemini");
      const cursorCard = screen.getByTestId("source-card-cursor");

      expect(geminiCard).toHaveAttribute("data-selected", "false");
      expect(cursorCard).toHaveAttribute("data-selected", "false");
    });
  });

  // Task 2.5: 禁用暂不支持的来源
  describe("Disabled Sources", () => {
    it("marks Gemini as post-mvp/disabled", () => {
      render(<SourceSelector value={null} onChange={vi.fn()} />);

      const geminiCard = screen.getByTestId("source-card-gemini");
      expect(geminiCard).toHaveAttribute("data-disabled", "true");
      // 检查徽章在 Gemini 卡片内
      expect(geminiCard).toHaveTextContent("即将推出");
    });

    it("marks Cursor as post-mvp/disabled", () => {
      render(<SourceSelector value={null} onChange={vi.fn()} />);

      const cursorCard = screen.getByTestId("source-card-cursor");
      expect(cursorCard).toHaveAttribute("data-disabled", "true");
    });

    it("does not call onChange when clicking disabled source", () => {
      const onChange = vi.fn();
      render(<SourceSelector value={null} onChange={onChange} />);

      const geminiCard = screen.getByTestId("source-card-gemini");
      fireEvent.click(geminiCard);

      expect(onChange).not.toHaveBeenCalled();
    });

    it("keeps Claude enabled", () => {
      render(<SourceSelector value={null} onChange={vi.fn()} />);

      const claudeCard = screen.getByTestId("source-card-claude");
      expect(claudeCard).toHaveAttribute("data-disabled", "false");
    });
  });

  // 无障碍测试
  describe("Accessibility", () => {
    it("has role radiogroup", () => {
      render(<SourceSelector value={null} onChange={vi.fn()} />);

      expect(screen.getByRole("radiogroup")).toBeInTheDocument();
    });

    it("has proper aria-label", () => {
      render(<SourceSelector value={null} onChange={vi.fn()} />);

      const radiogroup = screen.getByRole("radiogroup");
      expect(radiogroup).toHaveAttribute("aria-label", "选择导入来源");
    });

    it("marks selected source with aria-checked", () => {
      render(<SourceSelector value="claude" onChange={vi.fn()} />);

      const claudeCard = screen.getByTestId("source-card-claude");
      expect(claudeCard).toHaveAttribute("aria-checked", "true");
    });
  });
});
