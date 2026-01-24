/**
 * ProjectStatsView Tests - 项目级统计视图测试
 * Story 2.34: Code Review - M1 修复
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import { ProjectStatsView } from "./ProjectStatsView";
import type { ProjectAnalytics } from "@/types/analytics";

// Mock react-i18next
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "analytics.projectStats": "Project Statistics",
        "analytics.metrics.sessions": "Sessions",
        "analytics.metrics.duration": "Duration",
        "analytics.metrics.avgDuration": "Avg Duration",
        "analytics.metrics.activeDays": "Active Days",
        "analytics.metrics.errorRate": "Error Rate",
        "analytics.metrics.toolCalls": "Tool Calls",
        "analytics.metrics.messages": "Messages",
        "analytics.metrics.errors": "Errors",
        "analytics.units.days": "days",
        "analytics.charts.toolDistribution": "Tool Distribution",
        "analytics.charts.callRanking": "Call Ranking",
        "analytics.charts.activityTrend": "Activity Trend",
        "common.noData": "No data",
        "common.loading": "Loading",
      };
      return translations[key] || key;
    },
    i18n: { language: "en" },
  }),
}));

// Mock analytics IPC
const mockGetProjectAnalytics = vi.fn();
vi.mock("@/lib/analytics-ipc", () => ({
  getProjectAnalytics: (...args: unknown[]) => mockGetProjectAnalytics(...args),
}));

// Mock recharts to avoid rendering issues in tests
vi.mock("recharts", () => ({
  ResponsiveContainer: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="responsive-container">{children}</div>
  ),
  PieChart: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="pie-chart">{children}</div>
  ),
  Pie: () => null,
  Cell: () => null,
  AreaChart: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="area-chart">{children}</div>
  ),
  Area: () => null,
  BarChart: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="bar-chart">{children}</div>
  ),
  Bar: () => null,
  XAxis: () => null,
  YAxis: () => null,
  CartesianGrid: () => null,
  Tooltip: () => null,
}));

const mockAnalytics: ProjectAnalytics = {
  project_id: "test-project",
  time_range: "days7",
  total_sessions: 10,
  total_duration_seconds: 3600,
  avg_duration_seconds: 360,
  active_days: 5,
  tool_distribution: { claude: 5, gemini: 3, cursor: 2 },
  total_tool_calls: 100,
  total_tool_errors: 5,
  tool_error_rate: 0.05,
  tool_types_distribution: { file_read: 50, shell_exec: 30, file_write: 20 },
  activity_trend: [
    { date: "2026-01-20", session_count: 2, tool_call_count: 20, duration_seconds: 600 },
    { date: "2026-01-21", session_count: 3, tool_call_count: 30, duration_seconds: 900 },
  ],
  total_messages: 200,
};

describe("ProjectStatsView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("loading state", () => {
    it("should show loading skeleton initially", () => {
      mockGetProjectAnalytics.mockImplementation(() => new Promise(() => {}));

      render(<ProjectStatsView projectId="test-project" />);

      expect(screen.getByTestId("project-stats-loading")).toBeInTheDocument();
    });
  });

  describe("error state", () => {
    it("should show error message on failure", async () => {
      mockGetProjectAnalytics.mockRejectedValue(new Error("Network error"));

      render(<ProjectStatsView projectId="test-project" />);

      await waitFor(() => {
        expect(screen.getByTestId("project-stats-error")).toBeInTheDocument();
      });
    });
  });

  describe("empty state", () => {
    it("should show empty state when no sessions", async () => {
      mockGetProjectAnalytics.mockResolvedValue({
        ...mockAnalytics,
        total_sessions: 0,
      });

      render(<ProjectStatsView projectId="test-project" />);

      await waitFor(() => {
        expect(screen.getByTestId("analytics-empty-no-data")).toBeInTheDocument();
      });
    });
  });

  describe("data display", () => {
    it("should render project stats view with data", async () => {
      mockGetProjectAnalytics.mockResolvedValue(mockAnalytics);

      render(<ProjectStatsView projectId="test-project" projectName="Test Project" />);

      await waitFor(() => {
        expect(screen.getByTestId("project-stats-view")).toBeInTheDocument();
      });
    });

    it("should display project name in header", async () => {
      mockGetProjectAnalytics.mockResolvedValue(mockAnalytics);

      render(<ProjectStatsView projectId="test-project" projectName="My Project" />);

      await waitFor(() => {
        expect(screen.getByText(/My Project/)).toBeInTheDocument();
      });
    });

    it("should display metric cards", async () => {
      mockGetProjectAnalytics.mockResolvedValue(mockAnalytics);

      render(<ProjectStatsView projectId="test-project" />);

      await waitFor(() => {
        expect(screen.getByTestId("metric-sessions")).toBeInTheDocument();
        expect(screen.getByTestId("metric-duration")).toBeInTheDocument();
        expect(screen.getByTestId("metric-active-days")).toBeInTheDocument();
        expect(screen.getByTestId("metric-error-rate")).toBeInTheDocument();
      });
    });

    it("should format session count correctly", async () => {
      mockGetProjectAnalytics.mockResolvedValue(mockAnalytics);

      render(<ProjectStatsView projectId="test-project" />);

      await waitFor(() => {
        expect(screen.getByText("10")).toBeInTheDocument();
      });
    });

    it("should format error rate as percentage", async () => {
      mockGetProjectAnalytics.mockResolvedValue(mockAnalytics);

      render(<ProjectStatsView projectId="test-project" />);

      await waitFor(() => {
        expect(screen.getByText("5.0%")).toBeInTheDocument();
      });
    });
  });

  describe("time range", () => {
    it("should default to 7 days time range (AC4)", async () => {
      mockGetProjectAnalytics.mockResolvedValue(mockAnalytics);

      render(<ProjectStatsView projectId="test-project" />);

      await waitFor(() => {
        expect(mockGetProjectAnalytics).toHaveBeenCalledWith("test-project", "days7");
      });
    });
  });

  describe("import callback", () => {
    it("should call onImport when import button clicked in empty state", async () => {
      const onImport = vi.fn();
      mockGetProjectAnalytics.mockResolvedValue({
        ...mockAnalytics,
        total_sessions: 0,
      });

      render(<ProjectStatsView projectId="test-project" onImport={onImport} />);

      await waitFor(() => {
        expect(screen.getByTestId("analytics-empty-no-data")).toBeInTheDocument();
      });

      // Find and click the import button
      const importButton = screen.getByRole("button");
      importButton.click();

      expect(onImport).toHaveBeenCalled();
    });
  });
});
