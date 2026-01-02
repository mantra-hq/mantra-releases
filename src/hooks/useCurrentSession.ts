/**
 * useCurrentSession Hook - 获取当前会话及其项目信息
 * Story 2.17: Task 5
 *
 * 封装获取当前会话、项目信息和同项目会话列表的逻辑
 */

import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Project } from "@/types/project";
import type { MantraSession } from "@/lib/session-utils";
import type { SessionSummary } from "@/components/navigation";

/**
 * 后端 SessionSummary 类型 (来自 Rust)
 */
interface RustSessionSummary {
  id: string;
  source: string;
  created_at: string;
  updated_at: string;
  message_count: number;
}

/**
 * useCurrentSession 返回值类型
 */
export interface UseCurrentSessionResult {
  /** 当前会话信息 */
  session: MantraSession | null;
  /** 当前项目信息 */
  project: Project | null;
  /** 同项目会话列表 (用于 SessionDropdown) */
  sessions: SessionSummary[];
  /** 是否正在加载 */
  isLoading: boolean;
  /** 错误信息 */
  error: string | null;
  /** 重新获取数据 */
  refetch: () => void;
}

/**
 * 将 Rust SessionSummary 转换为前端 SessionSummary
 */
function convertToSessionSummary(
  session: MantraSession
): SessionSummary {
  // 从 metadata 或 id 生成会话名称
  const name =
    session.metadata?.title ||
    `Session ${session.id.slice(0, 8)}`;

  return {
    id: session.id,
    name,
    messageCount: session.messages.length,
    lastActiveAt: new Date(session.updated_at).getTime(),
  };
}

/**
 * 从 RustSessionSummary 转换为前端 SessionSummary
 */
function convertRustSessionSummary(
  rustSession: RustSessionSummary
): SessionSummary {
  return {
    id: rustSession.id,
    name: `Session ${rustSession.id.slice(0, 8)}`,
    messageCount: rustSession.message_count,
    lastActiveAt: new Date(rustSession.updated_at).getTime(),
  };
}

/**
 * useCurrentSession Hook
 * 获取当前会话、项目信息和同项目会话列表
 *
 * @param sessionId - 当前会话 ID
 */
export function useCurrentSession(
  sessionId: string | undefined
): UseCurrentSessionResult {
  const [session, setSession] = useState<MantraSession | null>(null);
  const [project, setProject] = useState<Project | null>(null);
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  /**
   * 获取会话和项目数据
   */
  const fetchData = useCallback(async () => {
    if (!sessionId) {
      setSession(null);
      setProject(null);
      setSessions([]);
      setIsLoading(false);
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      // 1. 获取当前会话
      const sessionData = await invoke<MantraSession | null>("get_session", {
        sessionId,
      });

      if (!sessionData) {
        setError("会话不存在");
        setSession(null);
        setProject(null);
        setSessions([]);
        setIsLoading(false);
        return;
      }

      setSession(sessionData);

      // 2. 根据 cwd 获取项目信息
      const projectData = await invoke<Project | null>("get_project_by_cwd", {
        cwd: sessionData.cwd,
      });

      setProject(projectData);

      // 3. 获取同项目会话列表
      if (projectData) {
        const projectSessions = await invoke<RustSessionSummary[]>(
          "get_project_sessions",
          { projectId: projectData.id }
        );

        // 转换为前端格式
        const sessionSummaries = projectSessions.map(convertRustSessionSummary);

        // 按最后活动时间降序排列
        sessionSummaries.sort((a, b) => b.lastActiveAt - a.lastActiveAt);

        setSessions(sessionSummaries);
      } else {
        // 无项目时，只显示当前会话
        setSessions([convertToSessionSummary(sessionData)]);
      }
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : "获取会话信息失败";
      setError(errorMessage);
      console.error("[useCurrentSession] Error:", err);
    } finally {
      setIsLoading(false);
    }
  }, [sessionId]);

  // 组件挂载或 sessionId 变化时获取数据
  useEffect(() => {
    fetchData();
  }, [fetchData]);

  return {
    session,
    project,
    sessions,
    isLoading,
    error,
    refetch: fetchData,
  };
}
