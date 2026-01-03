/**
 * PlayerEmptyState Component Tests
 * Story 2.21: Task 1.3
 *
 * 测试 Player 空状态组件
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { PlayerEmptyState } from "./PlayerEmptyState";

describe("PlayerEmptyState", () => {
  describe("有项目时的 UI (AC #4-8)", () => {
    it("应该显示 Play 图标 (AC #5)", () => {
      render(
        <PlayerEmptyState
          hasProjects={true}
          onOpenDrawer={vi.fn()}
          onImport={vi.fn()}
        />
      );

      const icon = screen.getByTestId("empty-state-icon");
      expect(icon).toBeInTheDocument();
    });

    it("应该显示主标题「选择一个会话开始回放」(AC #6)", () => {
      render(
        <PlayerEmptyState
          hasProjects={true}
          onOpenDrawer={vi.fn()}
          onImport={vi.fn()}
        />
      );

      expect(screen.getByText("选择一个会话开始回放")).toBeInTheDocument();
    });

    it("应该显示副标题 (AC #7)", () => {
      render(
        <PlayerEmptyState
          hasProjects={true}
          onOpenDrawer={vi.fn()}
          onImport={vi.fn()}
        />
      );

      expect(
        screen.getByText("从左侧项目列表中选择，或导入新的 AI 编程会话")
      ).toBeInTheDocument();
    });

    it("应该显示双按钮 (AC #8)", () => {
      render(
        <PlayerEmptyState
          hasProjects={true}
          onOpenDrawer={vi.fn()}
          onImport={vi.fn()}
        />
      );

      expect(screen.getByText("打开项目列表")).toBeInTheDocument();
      expect(screen.getByText("导入项目")).toBeInTheDocument();
    });

    it("点击「打开项目列表」应该触发 onOpenDrawer (AC #8)", () => {
      const onOpenDrawer = vi.fn();
      render(
        <PlayerEmptyState
          hasProjects={true}
          onOpenDrawer={onOpenDrawer}
          onImport={vi.fn()}
        />
      );

      fireEvent.click(screen.getByText("打开项目列表"));
      expect(onOpenDrawer).toHaveBeenCalledTimes(1);
    });

    it("点击「导入项目」应该触发 onImport (AC #8)", () => {
      const onImport = vi.fn();
      render(
        <PlayerEmptyState
          hasProjects={true}
          onOpenDrawer={vi.fn()}
          onImport={onImport}
        />
      );

      fireEvent.click(screen.getByText("导入项目"));
      expect(onImport).toHaveBeenCalledTimes(1);
    });
  });

  describe("无项目时的 UI (AC #9)", () => {
    it("应该显示 Folder 图标", () => {
      render(
        <PlayerEmptyState
          hasProjects={false}
          onOpenDrawer={vi.fn()}
          onImport={vi.fn()}
        />
      );

      const icon = screen.getByTestId("empty-state-icon");
      expect(icon).toBeInTheDocument();
    });

    it("应该显示主标题「还没有导入任何项目」(AC #9)", () => {
      render(
        <PlayerEmptyState
          hasProjects={false}
          onOpenDrawer={vi.fn()}
          onImport={vi.fn()}
        />
      );

      expect(screen.getByText("还没有导入任何项目")).toBeInTheDocument();
    });

    it("应该显示副标题 (AC #9)", () => {
      render(
        <PlayerEmptyState
          hasProjects={false}
          onOpenDrawer={vi.fn()}
          onImport={vi.fn()}
        />
      );

      expect(
        screen.getByText("导入你的 AI 编程会话，开始探索和回放心法")
      ).toBeInTheDocument();
    });

    it("应该显示单个 CTA「导入第一个项目」(AC #9)", () => {
      render(
        <PlayerEmptyState
          hasProjects={false}
          onOpenDrawer={vi.fn()}
          onImport={vi.fn()}
        />
      );

      expect(screen.getByText(/导入第一个项目/)).toBeInTheDocument();
      // 不应该显示「打开项目列表」按钮
      expect(screen.queryByText("打开项目列表")).not.toBeInTheDocument();
    });

    it("应该显示支持说明 (AC #9)", () => {
      render(
        <PlayerEmptyState
          hasProjects={false}
          onOpenDrawer={vi.fn()}
          onImport={vi.fn()}
        />
      );

      expect(screen.getByText(/Claude Code/)).toBeInTheDocument();
      expect(screen.getByText(/Cursor/)).toBeInTheDocument();
      expect(screen.getByText(/Gemini CLI/)).toBeInTheDocument();
      expect(screen.getByText(/Codex/)).toBeInTheDocument();
    });

    it("点击「导入第一个项目」应该触发 onImport (AC #9)", () => {
      const onImport = vi.fn();
      render(
        <PlayerEmptyState
          hasProjects={false}
          onOpenDrawer={vi.fn()}
          onImport={onImport}
        />
      );

      fireEvent.click(screen.getByText(/导入第一个项目/));
      expect(onImport).toHaveBeenCalledTimes(1);
    });
  });

  describe("data-testid", () => {
    it("应该有 player-empty-state testid", () => {
      render(
        <PlayerEmptyState
          hasProjects={true}
          onOpenDrawer={vi.fn()}
          onImport={vi.fn()}
        />
      );

      expect(screen.getByTestId("player-empty-state")).toBeInTheDocument();
    });
  });
});
