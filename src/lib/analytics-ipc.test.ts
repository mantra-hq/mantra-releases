/**
 * analytics-ipc Tests - 统计分析 IPC 测试
 * Story 2.34: Task 5.3
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import {
  getProjectAnalytics,
  getSessionMetrics,
  getSessionStatsView,
} from "./analytics-ipc";

// Mock Tauri invoke
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

import { invoke } from "@tauri-apps/api/core";

const mockInvoke = invoke as ReturnType<typeof vi.fn>;

describe("analytics-ipc", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe("getProjectAnalytics", () => {
    it("should call invoke with correct parameters", async () => {
      const mockAnalytics = {
        project_id: "proj-1",
        time_range: "days30",
        total_sessions: 10,
        total_duration_seconds: 36000,
        avg_duration_seconds: 3600,
        active_days: 5,
        tool_distribution: { claude: 5, gemini: 5 },
        total_tool_calls: 100,
        total_tool_errors: 2,
        tool_error_rate: 0.02,
        tool_types_distribution: { file_read: 50, shell_exec: 30 },
        activity_trend: [],
        total_messages: 200,
      };

      mockInvoke.mockResolvedValue(mockAnalytics);

      const result = await getProjectAnalytics("proj-1", "days30");

      expect(mockInvoke).toHaveBeenCalledWith("get_project_analytics", {
        projectId: "proj-1",
        timeRange: "days30",
      });
      expect(result).toEqual(mockAnalytics);
    });

    it("should default to days30 time range", async () => {
      mockInvoke.mockResolvedValue({});

      await getProjectAnalytics("proj-1");

      expect(mockInvoke).toHaveBeenCalledWith("get_project_analytics", {
        projectId: "proj-1",
        timeRange: "days30",
      });
    });

    it("should handle days7 time range", async () => {
      mockInvoke.mockResolvedValue({});

      await getProjectAnalytics("proj-1", "days7");

      expect(mockInvoke).toHaveBeenCalledWith("get_project_analytics", {
        projectId: "proj-1",
        timeRange: "days7",
      });
    });

    it("should handle all time range", async () => {
      mockInvoke.mockResolvedValue({});

      await getProjectAnalytics("proj-1", "all");

      expect(mockInvoke).toHaveBeenCalledWith("get_project_analytics", {
        projectId: "proj-1",
        timeRange: "all",
      });
    });
  });

  describe("getSessionMetrics", () => {
    it("should call invoke with correct parameters", async () => {
      const mockMetrics = {
        session_id: "sess-1",
        tool_type: "claude",
        start_time: 1700000000,
        end_time: 1700003600,
        duration_seconds: 3600,
        message_count: 20,
        tool_call_count: 15,
        tool_error_count: 1,
        tool_types_used: ["file_read", "shell_exec"],
        tool_type_counts: { file_read: 10, shell_exec: 5 },
      };

      mockInvoke.mockResolvedValue(mockMetrics);

      const result = await getSessionMetrics("sess-1");

      expect(mockInvoke).toHaveBeenCalledWith("get_session_metrics", {
        sessionId: "sess-1",
      });
      expect(result).toEqual(mockMetrics);
    });
  });

  describe("getSessionStatsView", () => {
    it("should call invoke with correct parameters", async () => {
      const mockStatsView = {
        metrics: {
          session_id: "sess-1",
          tool_type: "claude",
          start_time: 1700000000,
          end_time: 1700003600,
          duration_seconds: 3600,
          message_count: 20,
          tool_call_count: 15,
          tool_error_count: 0,
          tool_types_used: ["file_read"],
          tool_type_counts: { file_read: 15 },
        },
        tool_call_timeline: [
          {
            tool_type: "file_read",
            timestamp: 1700000100,
            is_error: false,
            description: "/src/main.rs",
          },
        ],
        tool_distribution: { file_read: 15 },
      };

      mockInvoke.mockResolvedValue(mockStatsView);

      const result = await getSessionStatsView("sess-1");

      expect(mockInvoke).toHaveBeenCalledWith("get_session_stats_view", {
        sessionId: "sess-1",
      });
      expect(result).toEqual(mockStatsView);
    });

    it("should return timeline with errors correctly", async () => {
      const mockStatsView = {
        metrics: {
          session_id: "sess-2",
          tool_type: "cursor",
          start_time: 1700000000,
          end_time: 1700001000,
          duration_seconds: 1000,
          message_count: 5,
          tool_call_count: 3,
          tool_error_count: 1,
          tool_types_used: ["shell_exec"],
          tool_type_counts: { shell_exec: 3 },
        },
        tool_call_timeline: [
          {
            tool_type: "shell_exec",
            timestamp: 1700000200,
            is_error: false,
            description: "npm install",
          },
          {
            tool_type: "shell_exec",
            timestamp: 1700000400,
            is_error: true,
            description: "failing command",
          },
        ],
        tool_distribution: { shell_exec: 2 },
      };

      mockInvoke.mockResolvedValue(mockStatsView);

      const result = await getSessionStatsView("sess-2");

      expect(result.tool_call_timeline[1].is_error).toBe(true);
      expect(result.metrics.tool_error_count).toBe(1);
    });
  });
});
