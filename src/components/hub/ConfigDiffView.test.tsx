/**
 * ConfigDiffView 组件测试
 * Story 11.13: Task 2 - 冲突差异对比组件测试
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ConfigDiffView } from "./ConfigDiffView";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        "hub.import.diffField": "字段",
        "hub.import.diffExisting": "已有配置",
        "hub.import.diffCandidate": `候选 ${params?.index || 0}`,
        "hub.import.diffCommand": "命令",
        "hub.import.diffArgs": "参数",
        "hub.import.diffEnv": "环境变量",
      };
      return translations[key] || key;
    },
  }),
}));

describe("ConfigDiffView", () => {
  const mockGetSourceText = (adapterId: string) => {
    switch (adapterId) {
      case "claude":
        return "Claude Code";
      case "cursor":
        return "Cursor";
      default:
        return adapterId;
    }
  };

  const mockExisting = {
    command: "npx",
    args: ["--yes", "@anthropic/mcp-server-git"],
    env: null,
  };

  const mockCandidates = [
    {
      name: "git-mcp",
      command: "npx",
      args: ["--yes", "@anthropic/mcp-server-git-v2"],
      env: { GIT_TOKEN: "xxx" },
      source_file: "/path/to/config.json",
      adapter_id: "claude",
    },
  ];

  it("应该渲染服务名称的 data-testid", () => {
    render(
      <ConfigDiffView
        serviceName="git-mcp"
        existing={mockExisting}
        candidates={mockCandidates}
        getSourceText={mockGetSourceText}
      />
    );

    expect(screen.getByTestId("config-diff-git-mcp")).toBeInTheDocument();
  });

  it("应该渲染字段标签", () => {
    render(
      <ConfigDiffView
        serviceName="git-mcp"
        existing={mockExisting}
        candidates={mockCandidates}
        getSourceText={mockGetSourceText}
      />
    );

    expect(screen.getByText("命令")).toBeInTheDocument();
    expect(screen.getByText("参数")).toBeInTheDocument();
    expect(screen.getByText("环境变量")).toBeInTheDocument();
  });

  it("应该渲染已有配置列", () => {
    render(
      <ConfigDiffView
        serviceName="git-mcp"
        existing={mockExisting}
        candidates={mockCandidates}
        getSourceText={mockGetSourceText}
      />
    );

    expect(screen.getByText("已有配置")).toBeInTheDocument();
  });

  it("应该渲染候选配置列", () => {
    render(
      <ConfigDiffView
        serviceName="git-mcp"
        existing={mockExisting}
        candidates={mockCandidates}
        getSourceText={mockGetSourceText}
      />
    );

    expect(screen.getByText(/候选 1/)).toBeInTheDocument();
    expect(screen.getByText(/Claude Code/)).toBeInTheDocument();
  });

  it("没有已有配置时不应该渲染已有配置列", () => {
    render(
      <ConfigDiffView
        serviceName="git-mcp"
        existing={null}
        candidates={mockCandidates}
        getSourceText={mockGetSourceText}
      />
    );

    expect(screen.queryByText("已有配置")).not.toBeInTheDocument();
  });

  it("应该渲染命令值", () => {
    render(
      <ConfigDiffView
        serviceName="git-mcp"
        existing={mockExisting}
        candidates={mockCandidates}
        getSourceText={mockGetSourceText}
      />
    );

    // 两列都应该显示 npx 命令
    const npxElements = screen.getAllByText("npx");
    expect(npxElements.length).toBeGreaterThanOrEqual(1);
  });

  it("应该渲染多个候选配置", () => {
    const multipleCandidates = [
      ...mockCandidates,
      {
        name: "git-mcp",
        command: "node",
        args: ["server.js"],
        env: null,
        source_file: "/path/to/cursor.json",
        adapter_id: "cursor",
      },
    ];

    render(
      <ConfigDiffView
        serviceName="git-mcp"
        existing={mockExisting}
        candidates={multipleCandidates}
        getSourceText={mockGetSourceText}
      />
    );

    expect(screen.getByText(/候选 1/)).toBeInTheDocument();
    expect(screen.getByText(/候选 2/)).toBeInTheDocument();
    expect(screen.getByText(/Cursor/)).toBeInTheDocument();
  });

  it("应该正确渲染空数组和空对象", () => {
    const candidatesWithEmptyValues = [
      {
        name: "test-mcp",
        command: "test",
        args: [],
        env: {},
        source_file: "/path/to/config.json",
        adapter_id: "claude",
      },
    ];

    render(
      <ConfigDiffView
        serviceName="test-mcp"
        existing={{ command: "test", args: null, env: null }}
        candidates={candidatesWithEmptyValues}
        getSourceText={mockGetSourceText}
      />
    );

    // 空数组应该显示为 []
    expect(screen.getByText("[]")).toBeInTheDocument();
    // 空对象应该显示为 {}
    expect(screen.getByText("{}")).toBeInTheDocument();
  });

  it("null 值应该显示为 -", () => {
    render(
      <ConfigDiffView
        serviceName="git-mcp"
        existing={{ command: "npx", args: null, env: null }}
        candidates={[
          {
            name: "git-mcp",
            command: "npx",
            args: null,
            env: null,
            source_file: "/path/to/config.json",
            adapter_id: "claude",
          },
        ]}
        getSourceText={mockGetSourceText}
      />
    );

    // 应该有多个 - 表示 null 值
    const dashElements = screen.getAllByText("-");
    expect(dashElements.length).toBeGreaterThanOrEqual(2);
  });
});
