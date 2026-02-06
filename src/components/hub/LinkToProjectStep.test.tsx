/**
 * LinkToProjectStep Tests
 * Story 11.29: Task 6 - 关联到项目步骤组件测试
 *
 * 6.1 单元测试：LinkToProjectStep 组件
 * - AC1: 服务列表默认全选
 * - AC2: 显示服务名称、来源图标、已关联状态
 * - AC6: 全部已关联时显示提示
 */

import { describe, it, expect, vi, beforeAll, afterEach } from "vitest";
import { render, screen, cleanup } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { LinkToProjectStep, type LinkableService } from "./LinkToProjectStep";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        "hub.import.allLinkedTitle": "所有服务已关联到当前项目",
        "hub.import.alreadyLinked": "已关联",
        "hub.import.linkDescription": `选择要关联到「${params?.project || ""}」的服务：`,
        "hub.import.linkDescriptionGeneric": "选择要关联到当前项目的服务：",
        "hub.import.linkHint": "关联后，这些服务的工具将对该项目的 AI 会话可用",
        "hub.import.linkSelectedCount": `已选择 ${params?.count || 0} 个服务`,
        "hub.import.selectAll": "全选",
        "hub.import.selectNone": "取消全选",
      };
      return translations[key] || key;
    },
    i18n: { language: "zh-CN" },
  }),
}));

// Radix UI PointerEvent polyfill
beforeAll(() => {
  class MockPointerEvent extends MouseEvent {
    constructor(type: string, props: PointerEventInit = {}) {
      super(type, props);
      Object.assign(this, {
        pointerId: props.pointerId ?? 0,
        width: props.width ?? 1,
        height: props.height ?? 1,
        pressure: props.pressure ?? 0,
        tangentialPressure: props.tangentialPressure ?? 0,
        tiltX: props.tiltX ?? 0,
        tiltY: props.tiltY ?? 0,
        twist: props.twist ?? 0,
        pointerType: props.pointerType ?? "mouse",
        isPrimary: props.isPrimary ?? true,
      });
    }
  }
  window.PointerEvent = MockPointerEvent as unknown as typeof PointerEvent;
  window.HTMLElement.prototype.scrollIntoView = vi.fn();
  window.HTMLElement.prototype.hasPointerCapture = vi.fn();
  window.HTMLElement.prototype.releasePointerCapture = vi.fn();
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

// ===== 测试数据 =====

const mockServices: LinkableService[] = [
  { id: "s1", name: "git-mcp", adapterId: "claude", alreadyLinked: false },
  { id: "s2", name: "postgres", adapterId: "cursor", alreadyLinked: false },
  { id: "s3", name: "context7", adapterId: "codex", alreadyLinked: true },
];

const allLinkedServices: LinkableService[] = [
  { id: "s1", name: "git-mcp", adapterId: "claude", alreadyLinked: true },
  { id: "s2", name: "postgres", adapterId: "cursor", alreadyLinked: true },
];

// ===== 测试 =====

describe("LinkToProjectStep", () => {
  describe("AC6: 所有服务已关联", () => {
    it("所有服务已关联时显示完成提示", () => {
      render(
        <LinkToProjectStep
          services={allLinkedServices}
          projectName="Mantra"
          selectedIds={new Set()}
          onSelectionChange={vi.fn()}
          allLinked={true}
        />
      );

      expect(screen.getByText("所有服务已关联到当前项目")).toBeInTheDocument();
      // 每个服务都显示已关联标签
      const linkedBadges = screen.getAllByText("已关联");
      expect(linkedBadges).toHaveLength(2);
    });

    it("所有服务已关联时显示服务名称", () => {
      render(
        <LinkToProjectStep
          services={allLinkedServices}
          projectName="Mantra"
          selectedIds={new Set()}
          onSelectionChange={vi.fn()}
          allLinked={true}
        />
      );

      expect(screen.getByText("git-mcp")).toBeInTheDocument();
      expect(screen.getByText("postgres")).toBeInTheDocument();
    });
  });

  describe("AC1: 服务选择列表", () => {
    it("显示服务列表和项目名称", () => {
      render(
        <LinkToProjectStep
          services={mockServices}
          projectName="Mantra"
          selectedIds={new Set(["s1", "s2"])}
          onSelectionChange={vi.fn()}
          allLinked={false}
        />
      );

      expect(screen.getByText(/选择要关联到「Mantra」的服务/)).toBeInTheDocument();
      expect(screen.getByText("git-mcp")).toBeInTheDocument();
      expect(screen.getByText("postgres")).toBeInTheDocument();
      expect(screen.getByText("context7")).toBeInTheDocument();
    });

    it("无项目名称时显示通用描述", () => {
      render(
        <LinkToProjectStep
          services={mockServices}
          selectedIds={new Set(["s1", "s2"])}
          onSelectionChange={vi.fn()}
          allLinked={false}
        />
      );

      expect(screen.getByText("选择要关联到当前项目的服务：")).toBeInTheDocument();
    });

    it("显示已选择的服务数量", () => {
      render(
        <LinkToProjectStep
          services={mockServices}
          selectedIds={new Set(["s1", "s2"])}
          onSelectionChange={vi.fn()}
          allLinked={false}
        />
      );

      expect(screen.getByText("已选择 2 个服务")).toBeInTheDocument();
    });

    it("显示提示信息", () => {
      render(
        <LinkToProjectStep
          services={mockServices}
          selectedIds={new Set(["s1"])}
          onSelectionChange={vi.fn()}
          allLinked={false}
        />
      );

      expect(screen.getByText("关联后，这些服务的工具将对该项目的 AI 会话可用")).toBeInTheDocument();
    });
  });

  describe("AC2: 已关联服务显示", () => {
    it("已关联的服务显示已关联标签", () => {
      render(
        <LinkToProjectStep
          services={mockServices}
          selectedIds={new Set(["s1", "s2"])}
          onSelectionChange={vi.fn()}
          allLinked={false}
        />
      );

      // context7 已关联
      expect(screen.getByText("已关联")).toBeInTheDocument();
    });

    it("已关联的服务复选框被禁用", () => {
      render(
        <LinkToProjectStep
          services={mockServices}
          selectedIds={new Set(["s1", "s2"])}
          onSelectionChange={vi.fn()}
          allLinked={false}
        />
      );

      // 查找 context7 的 checkbox (data-testid)
      const checkbox = screen.getByTestId("link-checkbox-context7");
      expect(checkbox).toBeDisabled();
    });
  });

  describe("服务选择交互", () => {
    it("点击复选框触发 onSelectionChange", async () => {
      const user = userEvent.setup();
      const onSelectionChange = vi.fn();

      render(
        <LinkToProjectStep
          services={mockServices}
          selectedIds={new Set(["s1", "s2"])}
          onSelectionChange={onSelectionChange}
          allLinked={false}
        />
      );

      // 取消选择 git-mcp
      const checkbox = screen.getByTestId("link-checkbox-git-mcp");
      await user.click(checkbox);

      expect(onSelectionChange).toHaveBeenCalledWith(new Set(["s2"]));
    });

    it("全选按钮选中所有可关联服务", async () => {
      const user = userEvent.setup();
      const onSelectionChange = vi.fn();

      render(
        <LinkToProjectStep
          services={mockServices}
          selectedIds={new Set()}
          onSelectionChange={onSelectionChange}
          allLinked={false}
        />
      );

      const selectAllButton = screen.getByText("全选");
      await user.click(selectAllButton);

      // 应该选中 s1 和 s2（s3 已关联，不在可选范围）
      expect(onSelectionChange).toHaveBeenCalledWith(new Set(["s1", "s2"]));
    });

    it("取消全选按钮清空选择", async () => {
      const user = userEvent.setup();
      const onSelectionChange = vi.fn();

      render(
        <LinkToProjectStep
          services={mockServices}
          selectedIds={new Set(["s1", "s2"])}
          onSelectionChange={onSelectionChange}
          allLinked={false}
        />
      );

      const selectNoneButton = screen.getByText("取消全选");
      await user.click(selectNoneButton);

      expect(onSelectionChange).toHaveBeenCalledWith(new Set());
    });
  });
});
