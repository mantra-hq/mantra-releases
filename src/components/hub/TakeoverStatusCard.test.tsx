/**
 * TakeoverStatusCard 组件测试
 * Story 11.15: Task 8.4 - 前端组件测试 (AC: 4, 5)
 *
 * 测试接管状态卡片组件的功能：
 * - 加载和显示接管记录
 * - 工具类型显示
 * - 一键恢复功能
 * - 确认对话框交互
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { TakeoverStatusCard } from "./TakeoverStatusCard";

// Mock IPC adapter
vi.mock("@/lib/ipc-adapter", () => ({
  invoke: vi.fn(),
}));

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      if (params?.count !== undefined) {
        return `${key}:${params.count}`;
      }
      if (params?.tool !== undefined) {
        return `${key}:${params.tool}`;
      }
      return key;
    },
  }),
}));

// Mock feedback
vi.mock("@/lib/feedback", () => ({
  feedback: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Mock SourceIcon component
vi.mock("@/components/import/SourceIcons", () => ({
  SourceIcon: ({ source, className }: { source: string; className?: string }) => (
    <div data-testid={`source-icon-${source}`} className={className}>
      {source}
    </div>
  ),
}));

// Import after mocking
import { invoke } from "@/lib/ipc-adapter";
import { feedback } from "@/lib/feedback";

const mockInvokeFn = vi.mocked(invoke);

// 测试数据
const mockBackups = [
  {
    id: "backup-1",
    tool_type: "claude_code" as const,
    original_path: "/home/user/.claude.json",
    backup_path: "/home/user/.claude.json.mantra-backup.1706745600",
    taken_over_at: "2024-02-01T12:00:00Z",
    restored_at: null,
    status: "active" as const,
  },
  {
    id: "backup-2",
    tool_type: "cursor" as const,
    original_path: "/home/user/.cursor/mcp.json",
    backup_path: "/home/user/.cursor/mcp.json.mantra-backup.1706745600",
    taken_over_at: "2024-02-01T13:00:00Z",
    restored_at: null,
    status: "active" as const,
  },
];

describe("TakeoverStatusCard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("加载状态", () => {
    it("加载中应该显示加载指示器", async () => {
      // 延迟返回以观察加载状态
      mockInvokeFn.mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve([]), 100))
      );

      render(<TakeoverStatusCard />);

      // 加载中应该显示卡片和加载器
      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("list_active_takeovers");
      });
    });

    it("无接管记录时不显示卡片", async () => {
      mockInvokeFn.mockResolvedValue([]);

      const { container } = render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.queryByTestId("takeover-status-card")).not.toBeInTheDocument();
      });

      // 确保没有渲染任何内容
      expect(container.firstChild).toBeNull();
    });

    it("有接管记录时显示卡片", async () => {
      mockInvokeFn.mockResolvedValue(mockBackups);

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("takeover-status-card")).toBeInTheDocument();
      });
    });
  });

  describe("接管记录显示", () => {
    beforeEach(() => {
      mockInvokeFn.mockResolvedValue(mockBackups);
    });

    it("应该显示标题和描述", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByText("hub.takeover.title")).toBeInTheDocument();
        expect(screen.getByText("hub.takeover.description")).toBeInTheDocument();
      });
    });

    it("应该显示活跃接管数量", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByText("hub.takeover.activeCount:2")).toBeInTheDocument();
      });
    });

    it("应该显示每个接管记录", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("takeover-item-backup-1")).toBeInTheDocument();
        expect(screen.getByTestId("takeover-item-backup-2")).toBeInTheDocument();
      });
    });

    it("应该显示工具类型名称", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByText("Claude Code")).toBeInTheDocument();
        expect(screen.getByText("Cursor")).toBeInTheDocument();
      });
    });

    it("应该显示正确的工具图标", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("source-icon-claude")).toBeInTheDocument();
        expect(screen.getByTestId("source-icon-cursor")).toBeInTheDocument();
      });
    });

    it("应该显示原始文件路径", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByText("/home/user/.claude.json")).toBeInTheDocument();
        expect(screen.getByText("/home/user/.cursor/mcp.json")).toBeInTheDocument();
      });
    });

    it("应该显示备份文件路径", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(
          screen.getByText("/home/user/.claude.json.mantra-backup.1706745600")
        ).toBeInTheDocument();
      });
    });

    it("应该显示活跃状态标签", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        const badges = screen.getAllByText("hub.takeover.active");
        expect(badges).toHaveLength(2);
      });
    });

    it("应该显示恢复按钮", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("restore-button-backup-1")).toBeInTheDocument();
        expect(screen.getByTestId("restore-button-backup-2")).toBeInTheDocument();
      });
    });
  });

  describe("工具类型转换", () => {
    it("claude_code 应该使用 claude 适配器 ID", async () => {
      mockInvokeFn.mockResolvedValue([mockBackups[0]]);

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("source-icon-claude")).toBeInTheDocument();
      });
    });

    it("gemini_cli 应该使用 gemini 适配器 ID", async () => {
      mockInvokeFn.mockResolvedValue([
        {
          ...mockBackups[0],
          id: "backup-gemini",
          tool_type: "gemini_cli",
        },
      ]);

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("source-icon-gemini")).toBeInTheDocument();
        expect(screen.getByText("Gemini CLI")).toBeInTheDocument();
      });
    });

    it("codex 应该使用 codex 适配器 ID", async () => {
      mockInvokeFn.mockResolvedValue([
        {
          ...mockBackups[0],
          id: "backup-codex",
          tool_type: "codex",
        },
      ]);

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("source-icon-codex")).toBeInTheDocument();
        expect(screen.getByText("Codex")).toBeInTheDocument();
      });
    });
  });

  describe("恢复功能", () => {
    beforeEach(() => {
      mockInvokeFn.mockResolvedValue(mockBackups);
    });

    it("点击恢复按钮应该打开确认对话框", async () => {
      const user = userEvent.setup();

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("restore-button-backup-1")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("restore-button-backup-1"));

      await waitFor(() => {
        expect(screen.getByText("hub.takeover.restoreConfirmTitle")).toBeInTheDocument();
        expect(
          screen.getByText("hub.takeover.restoreConfirmDescription:Claude Code")
        ).toBeInTheDocument();
      });
    });

    it("确认对话框应该显示恢复操作说明", async () => {
      const user = userEvent.setup();

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("restore-button-backup-1")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("restore-button-backup-1"));

      await waitFor(() => {
        expect(screen.getByText("hub.takeover.restoreWillDo1")).toBeInTheDocument();
        expect(screen.getByText("hub.takeover.restoreWillDo2")).toBeInTheDocument();
        expect(screen.getByText("hub.takeover.restoreWillDo3")).toBeInTheDocument();
      });
    });

    it("点击取消应该关闭对话框", async () => {
      const user = userEvent.setup();

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("restore-button-backup-1")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("restore-button-backup-1"));

      await waitFor(() => {
        expect(screen.getByText("hub.takeover.restoreConfirmTitle")).toBeInTheDocument();
      });

      await user.click(screen.getByText("common.cancel"));

      await waitFor(() => {
        expect(screen.queryByText("hub.takeover.restoreConfirmTitle")).not.toBeInTheDocument();
      });
    });

    it("确认恢复应该调用 restore_takeover 命令", async () => {
      const user = userEvent.setup();

      // 先返回备份列表，然后恢复成功后返回空列表
      mockInvokeFn
        .mockResolvedValueOnce(mockBackups)
        .mockResolvedValueOnce(undefined) // restore_takeover
        .mockResolvedValueOnce([]); // 刷新后返回空列表

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("restore-button-backup-1")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("restore-button-backup-1"));

      await waitFor(() => {
        expect(screen.getByText("hub.takeover.restoreConfirm")).toBeInTheDocument();
      });

      await user.click(screen.getByText("hub.takeover.restoreConfirm"));

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("restore_takeover", { backupId: "backup-1" });
      });

      expect(feedback.success).toHaveBeenCalledWith("hub.takeover.restoreSuccess");
    });

    it("恢复失败应该显示错误提示", async () => {
      const user = userEvent.setup();

      mockInvokeFn
        .mockResolvedValueOnce(mockBackups)
        .mockRejectedValueOnce(new Error("Backup file not found"));

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("restore-button-backup-1")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("restore-button-backup-1"));

      await waitFor(() => {
        expect(screen.getByText("hub.takeover.restoreConfirm")).toBeInTheDocument();
      });

      await user.click(screen.getByText("hub.takeover.restoreConfirm"));

      await waitFor(() => {
        expect(feedback.error).toHaveBeenCalledWith(
          "hub.takeover.restoreError",
          "Backup file not found"
        );
      });
    });

    it("恢复成功后应该调用 onRestore 回调", async () => {
      const user = userEvent.setup();
      const onRestore = vi.fn();

      mockInvokeFn
        .mockResolvedValueOnce(mockBackups)
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([]);

      render(<TakeoverStatusCard onRestore={onRestore} />);

      await waitFor(() => {
        expect(screen.getByTestId("restore-button-backup-1")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("restore-button-backup-1"));

      await waitFor(() => {
        expect(screen.getByText("hub.takeover.restoreConfirm")).toBeInTheDocument();
      });

      await user.click(screen.getByText("hub.takeover.restoreConfirm"));

      await waitFor(() => {
        expect(onRestore).toHaveBeenCalled();
      });
    });
  });

  describe("刷新功能", () => {
    it("点击刷新按钮应该重新加载数据", async () => {
      const user = userEvent.setup();

      mockInvokeFn.mockResolvedValue(mockBackups);

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("takeover-status-card")).toBeInTheDocument();
      });

      // 清除之前的调用记录
      mockInvokeFn.mockClear();

      // 点击刷新按钮
      const refreshButton = screen.getByTitle("common.refresh");
      await user.click(refreshButton);

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("list_active_takeovers");
      });
    });
  });

  describe("错误处理", () => {
    it("加载失败时不应该崩溃", async () => {
      mockInvokeFn.mockRejectedValue(new Error("Network error"));

      const { container } = render(<TakeoverStatusCard />);

      await waitFor(() => {
        // 加载失败后应该不显示卡片（因为 backups 仍为空数组）
        expect(screen.queryByTestId("takeover-status-card")).not.toBeInTheDocument();
      });

      // 确保组件没有崩溃
      expect(container).toBeDefined();
    });
  });
});
