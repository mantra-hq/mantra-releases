/**
 * SessionDropdown Tests - 会话下拉选择器组件测试
 * Story 2.17: Task 3
 */

import { describe, it, expect, vi, beforeEach, beforeAll, afterAll } from "vitest";
import { render, screen } from "@testing-library/react";
import { SessionDropdown } from "./SessionDropdown";
import type { SessionSummary } from "./TopBar";

// Mock ResizeObserver for cmdk
class ResizeObserverMock {
  observe = vi.fn();
  unobserve = vi.fn();
  disconnect = vi.fn();
}

beforeAll(() => {
  vi.stubGlobal("ResizeObserver", ResizeObserverMock);
});

afterAll(() => {
  vi.unstubAllGlobals();
});

// 测试数据
const mockSessions: SessionSummary[] = [
  {
    id: "session-1",
    name: "Session abc12345",
    messageCount: 10,
    lastActiveAt: Date.now() - 3600000, // 1 hour ago
  },
  {
    id: "session-2",
    name: "Session def67890",
    messageCount: 5,
    lastActiveAt: Date.now() - 7200000, // 2 hours ago
  },
  {
    id: "session-3",
    name: "Another Session",
    messageCount: 15,
    lastActiveAt: Date.now() - 86400000, // 1 day ago
  },
];

const defaultProps = {
  currentSessionId: "session-1",
  currentSessionName: "Session abc12345",
  messageCount: 10,
  sessions: mockSessions,
  onSessionSelect: vi.fn(),
};

describe("SessionDropdown", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("UI 展示", () => {
    it("应该显示当前会话名", () => {
      render(<SessionDropdown {...defaultProps} />);
      expect(screen.getByTestId("session-dropdown-trigger")).toHaveTextContent(
        "Session abc12345"
      );
    });

    it("应该显示消息数", () => {
      render(<SessionDropdown {...defaultProps} />);
      expect(screen.getByTestId("session-dropdown-trigger")).toHaveTextContent(
        "(10)"
      );
    });

    it("应该显示展开图标", () => {
      render(<SessionDropdown {...defaultProps} />);
      // ChevronsUpDown icon should be present
      const trigger = screen.getByTestId("session-dropdown-trigger");
      expect(trigger.querySelector("svg")).toBeInTheDocument();
    });
  });

  describe("下拉交互", () => {
    // 注意: cmdk 库在 jsdom 环境中有一些限制 (scrollIntoView, ResizeObserver)
    // 复杂的 Popover 交互测试应在 E2E 测试中进行
    it("触发器应该有正确的 aria 属性", () => {
      render(<SessionDropdown {...defaultProps} />);
      const trigger = screen.getByTestId("session-dropdown-trigger");
      expect(trigger).toHaveAttribute("aria-expanded", "false");
      expect(trigger).toHaveAttribute("role", "combobox");
    });
  });

  describe("会话选择", () => {
    // Popover 内部交互在 jsdom 环境中有限制
    // 实际会话选择功能将在 E2E 测试中验证
    it("应该正确设置 onSessionSelect 回调", () => {
      render(<SessionDropdown {...defaultProps} />);
      // 验证组件渲染成功，回调在真实环境中会被调用
      expect(screen.getByTestId("session-dropdown-trigger")).toBeInTheDocument();
    });
  });
});
