/**
 * SyncResultToast Tests
 * Story 2.19: Task 3.5
 *
 * 测试同步结果 Toast 通知函数
 */

import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, waitFor, cleanup } from "@testing-library/react";
import { Toaster } from "@/components/ui/sonner";
import { showSyncResult, type SyncResult } from "./SyncResultToast";

// Mock next-themes
vi.mock("next-themes", () => ({
  useTheme: () => ({ theme: "dark" }),
}));

describe("showSyncResult", () => {
  beforeEach(() => {
    render(<Toaster />);
  });

  afterEach(() => {
    cleanup();
  });

  it("shows success message with new and updated sessions (AC7)", async () => {
    const result: SyncResult = {
      new_sessions: [
        { id: "sess-1", source: "claude", created_at: "", updated_at: "", message_count: 5 },
        { id: "sess-2", source: "gemini", created_at: "", updated_at: "", message_count: 3 },
      ],
      updated_sessions: [
        { session_id: "sess-3", old_message_count: 5, new_message_count: 10 },
      ],
      unchanged_count: 5,
    };

    showSyncResult("test-project", result);

    await waitFor(() => {
      expect(screen.getByText(/同步完成/)).toBeInTheDocument();
    });

    await waitFor(() => {
      expect(screen.getByText(/2 个新会话/)).toBeInTheDocument();
      expect(screen.getByText(/1 个会话有新消息/)).toBeInTheDocument();
    });
  });

  it("shows only new sessions when no updates (AC7)", async () => {
    const result: SyncResult = {
      new_sessions: [
        { id: "sess-1", source: "claude", created_at: "", updated_at: "", message_count: 5 },
      ],
      updated_sessions: [],
      unchanged_count: 3,
    };

    showSyncResult("my-project", result);

    await waitFor(() => {
      expect(screen.getByText(/1 个新会话/)).toBeInTheDocument();
    });
  });

  it("shows only updates when no new sessions (AC7)", async () => {
    const result: SyncResult = {
      new_sessions: [],
      updated_sessions: [
        { session_id: "sess-1", old_message_count: 5, new_message_count: 8 },
        { session_id: "sess-2", old_message_count: 10, new_message_count: 15 },
      ],
      unchanged_count: 2,
    };

    showSyncResult("another-project", result);

    await waitFor(() => {
      expect(screen.getByText(/2 个会话有新消息/)).toBeInTheDocument();
    });
  });

  it("shows 'up to date' message when no changes (AC8)", async () => {
    const result: SyncResult = {
      new_sessions: [],
      updated_sessions: [],
      unchanged_count: 5,
    };

    showSyncResult("project", result);

    await waitFor(() => {
      expect(screen.getByText(/已是最新/)).toBeInTheDocument();
    });
  });

  it("handles error state", async () => {
    const error = new Error("Network error");

    showSyncResult("test-project", null, error);

    await waitFor(() => {
      expect(screen.getByText(/同步失败/)).toBeInTheDocument();
    });
  });
});
