/**
 * SessionStatsView Tests - 会话级统计视图测试
 * Story 2.34: Code Review - M1 修复
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { SessionStatsView } from "./SessionStatsView";
import type { SessionStatsView as SessionStatsViewType } from "@/types/analytics";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "analytics.sessionStats": "Session Statistics",
        "analytics.metrics.duration": "Duration",
        "analytics.metrics.messages": "Messages",
        "analytics.metrics.toolCalls": "Tool Calls",
        "analytics.metrics.errors": "Errors",
        "common.noData": "No data",
      };
      return translations[key] || key;
    },
    i18n: { language: "en" },
  }),
}));

// Mock analytics IPC
const mockGetSessionStatsView = vi.fn();
vi.mock("@/lib/analytics-ipc", () => ({
  getSessionStatsView: (...args: unknown[]) => mockGetSessionStatsView(...args),
}));

// Mock recharts
vi.mock("recharts", () => ({
  ResponsiveContainer: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="responsive-container">{children}</div>
  ),
  PieChart: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="pie-chart">{children}</div>
  ),
  Pie: () => null,
  Cell: () => null,
  Tooltip: () => null,
}));

// Mock date-fns
vi.mock("date-fns", () => ({
  format: (date: Date, formatStr: string) => {
    return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")} ${String(date.getHours()).padStart(2, "0")}:${String(date.getMinutes()).padStart(2, "0")}`;
  },
}));

vi.mock("date-fns/locale", () => ({
  zhCN: {},
  enUS: {},
}));

const mockStatsView: SessionStatsViewType = {
  metrics: {
    session_id: "test-session",
    tool_type: "claude",
    start_time: 1706000000,
    end_time: 1706003600,
    duration_seconds: 3600,
    message_count: 50,
    tool_call_count: 30,
    tool_error_count: 2,
    tool_types_used: ["file_read", "shell_exec", "file_write"],
    tool_type_counts: { file_read: 15, shell_exec: 10, file_write: 5 },
  },
  tool_call_timeline: [
    { tool_type: "file_read", timestamp: 1706000100, is_error: false, description: "/src/main.ts" },
    { tool_type: "shell_exec", timestamp: 1706000200, is_error: false, description: "npm test" },
    { tool_type: "file_write", timestamp: 1706000300, is_error: true, description: "/src/error.ts" },
  ],
  tool_distribution: { file_read: 15, shell_exec: 10, file_write: 5 },
};

describe("SessionStatsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("loading state", () => {
    it("should show loading skeleton initially", () => {
      mockGetSessionStatsView.mockImplementation(() => new Promise(() => {}));

      render(<SessionStatsView sessionId="test-session" />);

      expect(screen.getByTestId("session-stats-loading")).toBeInTheDocument();
    });
  });

  describe("error state", () => {
    it("should show error message on failure", async () => {
      mockGetSessionStatsView.mockRejectedValue(new Error("Session not found"));

      render(<SessionStatsView sessionId="test-session" />);

      await waitFor(() => {
        expect(screen.getByTestId("session-stats-error")).toBeInTheDocument();
      });
    });
  });

  describe("empty state", () => {
    it("should show empty state when no data returned", async () => {
      mockGetSessionStatsView.mockResolvedValue(null);

      render(<SessionStatsView sessionId="test-session" />);

      await waitFor(() => {
        expect(screen.getByTestId("session-stats-empty")).toBeInTheDocument();
      });
    });
  });

  describe("data display", () => {
    it("should render session stats view with data", async () => {
      mockGetSessionStatsView.mockResolvedValue(mockStatsView);

      render(<SessionStatsView sessionId="test-session" />);

      await waitFor(() => {
        expect(screen.getByTestId("session-stats-view")).toBeInTheDocument();
      });
    });

    it("should display metric cards", async () => {
      mockGetSessionStatsView.mockResolvedValue(mockStatsView);

      render(<SessionStatsView sessionId="test-session" />);

      await waitFor(() => {
        expect(screen.getByTestId("session-metric-duration")).toBeInTheDocument();
        expect(screen.getByTestId("session-metric-messages")).toBeInTheDocument();
        expect(screen.getByTestId("session-metric-tool-calls")).toBeInTheDocument();
        expect(screen.getByTestId("session-metric-errors")).toBeInTheDocument();
      });
    });

    it("should format duration correctly", async () => {
      mockGetSessionStatsView.mockResolvedValue(mockStatsView);

      render(<SessionStatsView sessionId="test-session" />);

      await waitFor(() => {
        // 3600 seconds = 1h
        expect(screen.getByText("1h")).toBeInTheDocument();
      });
    });

    it("should display message count", async () => {
      mockGetSessionStatsView.mockResolvedValue(mockStatsView);

      render(<SessionStatsView sessionId="test-session" />);

      await waitFor(() => {
        expect(screen.getByText("50")).toBeInTheDocument();
      });
    });

    it("should display tool call count", async () => {
      mockGetSessionStatsView.mockResolvedValue(mockStatsView);

      render(<SessionStatsView sessionId="test-session" />);

      await waitFor(() => {
        expect(screen.getByText("30")).toBeInTheDocument();
      });
    });

    it("should display error count", async () => {
      mockGetSessionStatsView.mockResolvedValue(mockStatsView);

      render(<SessionStatsView sessionId="test-session" />);

      await waitFor(() => {
        expect(screen.getByText("2")).toBeInTheDocument();
      });
    });
  });

  describe("IPC calls", () => {
    it("should call getSessionStatsView with correct sessionId", async () => {
      mockGetSessionStatsView.mockResolvedValue(mockStatsView);

      render(<SessionStatsView sessionId="my-session-123" />);

      await waitFor(() => {
        expect(mockGetSessionStatsView).toHaveBeenCalledWith("my-session-123");
      });
    });

    it("should reload data when sessionId changes", async () => {
      mockGetSessionStatsView.mockResolvedValue(mockStatsView);

      const { rerender } = render(<SessionStatsView sessionId="session-1" />);

      await waitFor(() => {
        expect(mockGetSessionStatsView).toHaveBeenCalledWith("session-1");
      });

      rerender(<SessionStatsView sessionId="session-2" />);

      await waitFor(() => {
        expect(mockGetSessionStatsView).toHaveBeenCalledWith("session-2");
      });
    });
  });
});
