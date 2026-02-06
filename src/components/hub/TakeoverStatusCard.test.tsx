/**
 * TakeoverStatusCard 组件测试
 * Story 11.16: 接管状态模块系统性重构
 *
 * 测试接管状态卡片组件的功能：
 * - 加载和显示接管记录
 * - 按 scope 分组显示（用户级/项目级）
 * - 折叠/展开功能
 * - 文件预览功能
 * - 一键恢复功能
 * - 确认对话框交互
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
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
    i18n: { language: "en" },
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

// 默认的 BackupStats mock
const mockBackupStats = {
  totalCount: 0,
  totalSize: 0,
  groups: [],
};

// 测试数据 - 包含 TakeoverBackupIntegrity 所需的完整性字段
const mockUserBackups = [
  {
    id: "backup-1",
    toolType: "claude_code" as const,
    scope: "user" as const,
    projectPath: null,
    originalPath: "/home/user/.claude.json",
    backupPath: "/home/user/.claude.json.mantra-backup.1706745600",
    backupHash: null,
    takenOverAt: "2024-02-01T12:00:00Z",
    restoredAt: null,
    status: "active" as const,
    backupFileExists: true,
    originalFileExists: true,
    hashValid: null,
  },
  {
    id: "backup-2",
    toolType: "cursor" as const,
    scope: "user" as const,
    projectPath: null,
    originalPath: "/home/user/.cursor/mcp.json",
    backupPath: "/home/user/.cursor/mcp.json.mantra-backup.1706745600",
    backupHash: null,
    takenOverAt: "2024-02-01T13:00:00Z",
    restoredAt: null,
    status: "active" as const,
    backupFileExists: true,
    originalFileExists: true,
    hashValid: null,
  },
];

const mockProjectBackups = [
  {
    id: "backup-3",
    toolType: "claude_code" as const,
    scope: "project" as const,
    projectPath: "/home/user/my-project",
    originalPath: "/home/user/my-project/.mcp.json",
    backupPath: "/home/user/my-project/.mcp.json.mantra-backup.1706745600",
    backupHash: null,
    takenOverAt: "2024-02-01T14:00:00Z",
    restoredAt: null,
    status: "active" as const,
    backupFileExists: true,
    originalFileExists: true,
    hashValid: null,
  },
];

const mockAllBackups = [...mockUserBackups, ...mockProjectBackups];

/**
 * 创建按命令名分发的 invoke mock
 * 组件在 loadBackups 中依次调用 list_takeover_backups_with_version 和 get_backup_stats
 */
function setupInvokeMock(
  backups: unknown[] = [],
  overrides: Record<string, unknown> = {},
) {
  mockInvokeFn.mockImplementation((cmd: string, ..._args: unknown[]) => {
    if (cmd in overrides) {
      const val = overrides[cmd];
      if (val instanceof Error) return Promise.reject(val);
      return Promise.resolve(val);
    }
    switch (cmd) {
      case "list_takeover_backups_with_version":
        return Promise.resolve(backups);
      case "get_backup_stats":
        return Promise.resolve(mockBackupStats);
      default:
        return Promise.resolve(undefined);
    }
  });
}

describe("TakeoverStatusCard", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("加载状态", () => {
    it("加载中应该显示加载指示器", async () => {
      mockInvokeFn.mockImplementation(
        () => new Promise((resolve) => setTimeout(() => resolve([]), 100))
      );

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("list_takeover_backups_with_version");
      });
    });

    it("无接管记录时不显示卡片", async () => {
      setupInvokeMock([]);

      const { container } = render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.queryByTestId("takeover-status-card")).not.toBeInTheDocument();
      });

      expect(container.firstChild).toBeNull();
    });

    it("有接管记录时显示卡片", async () => {
      setupInvokeMock(mockUserBackups);

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("takeover-status-card")).toBeInTheDocument();
      });
    });
  });

  describe("接管记录显示", () => {
    beforeEach(() => {
      setupInvokeMock(mockUserBackups);
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

    it("应该显示恢复按钮", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("restore-button-backup-1")).toBeInTheDocument();
        expect(screen.getByTestId("restore-button-backup-2")).toBeInTheDocument();
      });
    });
  });

  describe("按 scope 分组 (Story 11.16: AC3)", () => {
    beforeEach(() => {
      setupInvokeMock(mockAllBackups);
    });

    it("应该显示用户级配置分组", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByText("hub.takeover.userLevel")).toBeInTheDocument();
      });
    });

    it("应该显示项目级配置分组", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByText("hub.takeover.projectLevel")).toBeInTheDocument();
      });
    });

    it("用户级分组应该默认展开", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        // 用户级配置应该可见
        expect(screen.getByTestId("takeover-item-backup-1")).toBeInTheDocument();
        expect(screen.getByTestId("takeover-item-backup-2")).toBeInTheDocument();
      });
    });

    it("项目级分组应该默认收起", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        // 项目级配置默认不可见
        expect(screen.queryByTestId("takeover-item-backup-3")).not.toBeInTheDocument();
      });
    });

    it("点击项目级分组应该展开显示项目", async () => {
      const user = userEvent.setup();

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByText("hub.takeover.projectLevel")).toBeInTheDocument();
      });

      // 点击项目级分组展开
      await user.click(screen.getByText("hub.takeover.projectLevel"));

      await waitFor(() => {
        // 项目名称应该可见
        expect(screen.getByText("my-project")).toBeInTheDocument();
      });
    });
  });

  describe("工具类型转换", () => {
    it("claude_code 应该使用 claude 适配器 ID", async () => {
      setupInvokeMock([mockUserBackups[0]]);

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("source-icon-claude")).toBeInTheDocument();
      });
    });

    it("gemini_cli 应该使用 gemini 适配器 ID", async () => {
      setupInvokeMock([
        {
          ...mockUserBackups[0],
          id: "backup-gemini",
          toolType: "gemini_cli",
        },
      ]);

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("source-icon-gemini")).toBeInTheDocument();
        expect(screen.getByText("Gemini CLI")).toBeInTheDocument();
      });
    });

    it("codex 应该使用 codex 适配器 ID", async () => {
      setupInvokeMock([
        {
          ...mockUserBackups[0],
          id: "backup-codex",
          toolType: "codex",
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
      setupInvokeMock(mockUserBackups);
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

      // 使用 implementation 按命令分发，恢复后刷新返回空列表
      let restoreCalled = false;
      mockInvokeFn.mockImplementation((cmd: string) => {
        switch (cmd) {
          case "list_takeover_backups_with_version":
            return Promise.resolve(restoreCalled ? [] : mockUserBackups);
          case "get_backup_stats":
            return Promise.resolve(mockBackupStats);
          case "restore_takeover":
            restoreCalled = true;
            return Promise.resolve(undefined);
          default:
            return Promise.resolve(undefined);
        }
      });

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

      mockInvokeFn.mockImplementation((cmd: string) => {
        switch (cmd) {
          case "list_takeover_backups_with_version":
            return Promise.resolve(mockUserBackups);
          case "get_backup_stats":
            return Promise.resolve(mockBackupStats);
          case "restore_takeover":
            return Promise.reject(new Error("Backup file not found"));
          default:
            return Promise.resolve(undefined);
        }
      });

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

      let restoreCalled = false;
      mockInvokeFn.mockImplementation((cmd: string) => {
        switch (cmd) {
          case "list_takeover_backups_with_version":
            return Promise.resolve(restoreCalled ? [] : mockUserBackups);
          case "get_backup_stats":
            return Promise.resolve(mockBackupStats);
          case "restore_takeover":
            restoreCalled = true;
            return Promise.resolve(undefined);
          case "get_gateway_status":
            return Promise.resolve({ running: false });
          default:
            return Promise.resolve(undefined);
        }
      });

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

    it("恢复成功后应该刷新列表", async () => {
      const user = userEvent.setup();

      let restoreCalled = false;
      mockInvokeFn.mockImplementation((cmd: string) => {
        switch (cmd) {
          case "list_takeover_backups_with_version":
            return Promise.resolve(restoreCalled ? [] : mockUserBackups);
          case "get_backup_stats":
            return Promise.resolve(mockBackupStats);
          case "restore_takeover":
            restoreCalled = true;
            return Promise.resolve(undefined);
          default:
            return Promise.resolve(undefined);
        }
      });

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("restore-button-backup-1")).toBeInTheDocument();
      });

      await user.click(screen.getByTestId("restore-button-backup-1"));

      await waitFor(() => {
        expect(screen.getByText("hub.takeover.restoreConfirm")).toBeInTheDocument();
      });

      await user.click(screen.getByText("hub.takeover.restoreConfirm"));

      // 恢复后 loadBackups 被调用（list_takeover_backups_with_version 被调用 2 次）
      await waitFor(() => {
        const calls = mockInvokeFn.mock.calls.filter(
          (c) => c[0] === "list_takeover_backups_with_version"
        );
        expect(calls.length).toBeGreaterThanOrEqual(2);
      });
    });
  });

  describe("文件预览功能 (Story 11.16: AC5)", () => {
    beforeEach(() => {
      setupInvokeMock(mockUserBackups);
    });

    it("应该显示预览按钮", async () => {
      render(<TakeoverStatusCard />);

      await waitFor(() => {
        // 每条记录有两个预览按钮（当前配置和原始备份）
        const previewButtons = screen.getAllByTitle("hub.takeover.preview");
        expect(previewButtons.length).toBeGreaterThanOrEqual(4); // 2 records × 2 buttons
      });
    });

    it("点击预览按钮应该调用 read_config_file_content", async () => {
      const user = userEvent.setup();

      setupInvokeMock(mockUserBackups, {
        read_config_file_content: '{"key": "value"}',
      });

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("takeover-item-backup-1")).toBeInTheDocument();
      });

      // 点击第一个预览按钮
      const previewButtons = screen.getAllByTitle("hub.takeover.preview");
      await user.click(previewButtons[0]);

      await waitFor(() => {
        expect(mockInvokeFn).toHaveBeenCalledWith("read_config_file_content", {
          path: "/home/user/.claude.json",
        });
      });
    });

    it("预览抽屉应该显示文件内容", async () => {
      const user = userEvent.setup();

      setupInvokeMock(mockUserBackups, {
        read_config_file_content: '{"key": "value"}',
      });

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("takeover-item-backup-1")).toBeInTheDocument();
      });

      const previewButtons = screen.getAllByTitle("hub.takeover.preview");
      await user.click(previewButtons[0]);

      await waitFor(() => {
        expect(screen.getByText("hub.takeover.filePreview")).toBeInTheDocument();
        // 语法高亮将内容拆分到多个 span 中，使用子字符串匹配
        expect(screen.getByText(/"key"/)).toBeInTheDocument();
        expect(screen.getByText(/"value"/)).toBeInTheDocument();
      });
    });

    it("文件读取失败应该显示错误信息", async () => {
      const user = userEvent.setup();

      setupInvokeMock(mockUserBackups, {
        read_config_file_content: new Error("File not found"),
      });

      render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.getByTestId("takeover-item-backup-1")).toBeInTheDocument();
      });

      const previewButtons = screen.getAllByTitle("hub.takeover.preview");
      await user.click(previewButtons[0]);

      await waitFor(() => {
        expect(screen.getByText("hub.takeover.previewError")).toBeInTheDocument();
        expect(screen.getByText("File not found")).toBeInTheDocument();
      });
    });
  });

  describe("错误处理", () => {
    it("加载失败时不应该崩溃", async () => {
      mockInvokeFn.mockRejectedValue(new Error("Network error"));

      const { container } = render(<TakeoverStatusCard />);

      await waitFor(() => {
        expect(screen.queryByTestId("takeover-status-card")).not.toBeInTheDocument();
      });

      expect(container).toBeDefined();
    });
  });
});
