/**
 * ShadowModePreview 组件测试
 * Story 11.13: Task 4 - 影子模式变更预览组件测试
 *
 * 注意：由于 React 18 并发模式和 vitest 异步测试的限制，
 * 更复杂的异步场景通过 McpConfigImportDialog.test.tsx 的集成测试覆盖。
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { ShadowModePreview } from "./ShadowModePreview";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "hub.import.shadowPreviewTrigger": "查看变更预览",
        "hub.import.shadowPreviewLoading": "加载预览中...",
        "hub.import.shadowBefore": "修改前",
        "hub.import.shadowAfter": "修改后",
        "hub.import.shadowBackupTo": "备份至",
        "hub.import.shadowNoChanges": "无配置文件需要修改",
      };
      return translations[key] || key;
    },
  }),
}));

// Mock IPC adapter
const mockInvoke = vi.fn();
vi.mock("@/lib/ipc-adapter", () => ({
  invoke: (...args: unknown[]) => mockInvoke(...args),
}));

const mockConfigs = [
  {
    adapter_id: "claude",
    path: "/home/user/.claude/config.json",
    scope: "user" as const,
    services: [
      {
        name: "git-mcp",
        command: "npx",
        args: ["--yes", "@anthropic/mcp-server-git"],
        env: null,
        source_file: "/home/user/.claude/config.json",
        adapter_id: "claude",
      },
    ],
    parse_errors: [],
  },
];

describe("ShadowModePreview", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockInvoke.mockReset();
    // 默认返回一个永不解决的 promise，以便测试加载状态
    mockInvoke.mockImplementation(() => new Promise(() => {}));
  });

  it("禁用时不应该渲染任何内容", () => {
    render(<ShadowModePreview enabled={false} configs={mockConfigs} />);
    expect(screen.queryByTestId("shadow-mode-preview-trigger")).not.toBeInTheDocument();
  });

  it("启用时应该渲染触发按钮", () => {
    render(<ShadowModePreview enabled={true} configs={mockConfigs} />);
    expect(screen.getByTestId("shadow-mode-preview-trigger")).toBeInTheDocument();
    expect(screen.getByText("查看变更预览")).toBeInTheDocument();
  });

  it("点击触发按钮应该调用预览接口", async () => {
    const user = userEvent.setup();
    render(<ShadowModePreview enabled={true} configs={mockConfigs} />);

    await user.click(screen.getByTestId("shadow-mode-preview-trigger"));

    expect(mockInvoke).toHaveBeenCalledWith("preview_shadow_mode_changes", {
      configs: mockConfigs,
    });
  });

  it("加载中应该显示加载状态", async () => {
    const user = userEvent.setup();
    render(<ShadowModePreview enabled={true} configs={mockConfigs} />);

    await user.click(screen.getByTestId("shadow-mode-preview-trigger"));

    // 在 promise 解决前应该显示加载状态
    expect(screen.getByText("加载预览中...")).toBeInTheDocument();
  });

  it("禁用后触发器应该消失", () => {
    const { rerender } = render(<ShadowModePreview enabled={true} configs={mockConfigs} />);

    expect(screen.getByTestId("shadow-mode-preview-trigger")).toBeInTheDocument();

    rerender(<ShadowModePreview enabled={false} configs={mockConfigs} />);

    expect(screen.queryByTestId("shadow-mode-preview-trigger")).not.toBeInTheDocument();
  });
});
