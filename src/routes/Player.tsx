/**
 * Player Page - 会话回放页面
 * Story 2.8: Task 1
 * Story 2.10: Task 6 (Message Navigation from Search)
 * Story 2.11: Task 6 (Initial Code Display)
 *
 * 封装 DualStreamLayout，用于播放会话内容
 *
 * 从后端加载真实会话数据并转换为前端格式
 * 集成 Git Time Machine 实现代码快照功能 (FR-GIT)
 * 进入时自动显示项目代表性文件 (AC1, AC2)
 */

import * as React from "react";
import { useParams, useNavigate, useSearchParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { ThemeToggle } from "@/components/theme-toggle";
import { DualStreamLayout, type DualStreamLayoutRef } from "@/components/layout";
import { convertSessionToMessages, type MantraSession } from "@/lib/session-utils";
import { getProjectByCwd, getRepresentativeFile, detectGitRepo } from "@/lib/project-ipc";
import type { NarrativeMessage } from "@/types/message";
import {
  messagesToTimelineEvents,
  getTimelineRange,
  type TimelineEvent,
} from "@/types/timeline";
import { TimberLine } from "@/components/timeline";
import { ArrowLeft, AlertCircle, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useTimeTravelStore } from "@/stores/useTimeTravelStore";
import { useSearchStore } from "@/stores/useSearchStore";
import { useTimeMachine } from "@/hooks/useTimeMachine";


/**
 * 从消息内容块中提取文件路径
 * 优先从 tool_use 的 file_path 参数中提取
 */
function extractFilePathFromMessage(message: NarrativeMessage): string | null {
  for (const block of message.content) {
    // 从 tool_use 提取
    if (block.type === "tool_use" && block.toolInput) {
      const input = block.toolInput as Record<string, unknown>;
      // 常见的文件路径参数名
      const pathKeys = ["file_path", "filePath", "path", "filename"];
      for (const key of pathKeys) {
        if (typeof input[key] === "string" && input[key]) {
          return input[key] as string;
        }
      }
    }
    // 从 associatedFilePath 提取
    if (block.associatedFilePath) {
      return block.associatedFilePath;
    }
  }
  return null;
}

/**
 * 从消息列表中提取最近的文件路径
 * 从指定索引向前搜索
 */
function findRecentFilePath(
  messages: NarrativeMessage[],
  fromIndex: number
): string | null {
  // 从当前消息向前搜索
  for (let i = fromIndex; i >= 0; i--) {
    const filePath = extractFilePathFromMessage(messages[i]);
    if (filePath) {
      return filePath;
    }
  }
  return null;
}

/**
 * Player 页面组件
 * 展示会话回放的 DualStreamLayout
 */
export default function Player() {
  const { sessionId } = useParams<{ sessionId: string }>();
  const navigate = useNavigate();
  const [searchParams, setSearchParams] = useSearchParams();

  // 从 URL 获取 messageId (Story 2.10: 从全局搜索跳转)
  const targetMessageId = searchParams.get("messageId");

  // DualStreamLayout ref 用于程序化滚动
  const layoutRef = React.useRef<DualStreamLayoutRef>(null);

  // 是否已处理初始滚动
  const hasScrolledToTargetRef = React.useRef(false);

  // 选中的消息 ID
  const [selectedMessageId, setSelectedMessageId] = React.useState<string | undefined>();

  // 会话数据状态
  const [messages, setMessages] = React.useState<NarrativeMessage[]>([]);
  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);
  const [sessionCwd, setSessionCwd] = React.useState<string | undefined>();

  // Git 仓库路径状态 (FR-GIT-001)
  const [repoPath, setRepoPath] = React.useState<string | null>(null);
  // 无 Git 仓库标记 (Story 2.11 AC6)
  const [hasNoGit, setHasNoGit] = React.useState<boolean>(false);

  // 时间轴状态 (Story 2.6)
  const [timelineEvents, setTimelineEvents] = React.useState<TimelineEvent[]>([]);
  const [timelineRange, setTimelineRange] = React.useState<{ startTime: number; endTime: number }>({
    startTime: Date.now(),
    endTime: Date.now(),
  });
  const [currentTime, setCurrentTime] = React.useState<number>(Date.now());

  // 时间旅行 Store (Story 2.7 AC #6)
  const jumpToMessage = useTimeTravelStore((state) => state.jumpToMessage);
  const setStoreCurrentTime = useTimeTravelStore((state) => state.setCurrentTime);

  // Git Time Machine Hook (FR-GIT-002, FR-GIT-003)
  const { fetchSnapshot } = useTimeMachine(repoPath);

  // 加载会话数据
  React.useEffect(() => {
    if (!sessionId) {
      setLoading(false);
      return;
    }

    let cancelled = false;

    async function loadSession() {
      try {
        setLoading(true);
        setError(null);

        const session = await invoke<MantraSession | null>("get_session", {
          sessionId,
        });

        if (cancelled) return;

        if (!session) {
          setError("会话不存在");
          setMessages([]);
        } else {
          const narrativeMessages = convertSessionToMessages(session);
          setMessages(narrativeMessages);
          setSessionCwd(session.cwd);

          // 计算时间轴数据 (Story 2.6)
          const events = messagesToTimelineEvents(narrativeMessages);
          setTimelineEvents(events);
          const range = getTimelineRange(events);
          setTimelineRange(range);
          // 初始化当前时间为第一条消息的时间
          setCurrentTime(range.startTime);
        }
      } catch (err) {
        if (cancelled) return;
        console.error("Failed to load session:", err);
        setError(err instanceof Error ? err.message : "加载会话失败");
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    loadSession();

    return () => {
      cancelled = true;
    };
  }, [sessionId]);

  // 处理从全局搜索跳转的消息定位 (Story 2.10: Task 6)
  const addRecentSession = useSearchStore((state) => state.addRecentSession);

  React.useEffect(() => {
    if (
      !loading &&
      messages.length > 0 &&
      targetMessageId &&
      !hasScrolledToTargetRef.current
    ) {
      // 标记已处理
      hasScrolledToTargetRef.current = true;

      // 滚动到目标消息
      const targetIndex = messages.findIndex((m) => m.id === targetMessageId);
      if (targetIndex >= 0) {
        const targetMessage = messages[targetIndex];
        setSelectedMessageId(targetMessageId);

        // 延迟滚动，确保 DOM 已渲染
        setTimeout(() => {
          layoutRef.current?.scrollToMessage(targetMessageId);
        }, 100);

        // 更新时间轴
        const msgTime = new Date(targetMessage.timestamp).getTime();
        setCurrentTime(msgTime);
        jumpToMessage(targetIndex, targetMessageId, msgTime);
      }

      // 清除 URL 参数，避免刷新时重复滚动
      setSearchParams({}, { replace: true });
    }
  }, [loading, messages, targetMessageId, jumpToMessage, setSearchParams]);

  // 记录最近访问的会话 (Story 2.10)
  React.useEffect(() => {
    if (sessionId && sessionCwd && !loading && messages.length > 0) {
      // 从 cwd 提取项目名
      const projectName = sessionCwd.split("/").pop() || sessionCwd;
      addRecentSession({
        projectId: sessionCwd, // 使用 cwd 作为临时 projectId
        projectName,
        sessionId,
        sessionName: `Session ${sessionId.slice(0, 8)}`,
        accessedAt: Date.now(),
      });
    }
  }, [sessionId, sessionCwd, loading, messages.length, addRecentSession]);

  // Git 仓库检测 + 初始代码加载 (FR-GIT-001, Story 2.11 AC1, AC2)
  const setCode = useTimeTravelStore((state) => state.setCode);

  React.useEffect(() => {
    if (!sessionCwd) {
      setRepoPath(null);
      return;
    }

    let cancelled = false;

    async function detectRepoAndLoadInitialCode() {
      try {
        // 1. 从 Project 获取 Git 仓库信息 (Story 2.11 AC8 - 避免重复检测)
        // sessionCwd is guaranteed to be defined here due to early return above
        const project = await getProjectByCwd(sessionCwd!);

        if (cancelled) return;

        if (project?.has_git_repo && project.git_repo_path) {
          setRepoPath(project.git_repo_path);
          console.log("[Player] Git 仓库检测成功 (from Project):", project.git_repo_path);

          // 2. 获取代表性文件作为初始代码 (Story 2.11 AC1, AC2)
          try {
            const repFile = await getRepresentativeFile(project.git_repo_path);
            if (cancelled) return;

            if (repFile) {
              // 设置初始代码到 store，启用"当前代码"模式
              setCode(repFile.content, repFile.path);
              console.log("[Player] 初始代码加载成功:", repFile.path);
            }
          } catch (repErr) {
            console.warn("[Player] 代表性文件加载失败:", repErr);
          }
        } else {
          // 没有 Git 仓库 - 回退到 detect_git_repo 命令 (使用封装的 IPC)
          const detected = await detectGitRepo(sessionCwd!);
          if (!cancelled) {
            setRepoPath(detected);
            if (detected) {
              console.log("[Player] Git 仓库检测成功 (from detect):", detected);
              setHasNoGit(false);
              // 加载代表性文件
              try {
                const repFile = await getRepresentativeFile(detected);
                if (!cancelled && repFile) {
                  setCode(repFile.content, repFile.path);
                  console.log("[Player] 初始代码加载成功:", repFile.path);
                }
              } catch (repErr) {
                console.warn("[Player] 代表性文件加载失败:", repErr);
              }
            } else {
              console.log("[Player] 未检测到 Git 仓库 (AC6):", sessionCwd);
              setHasNoGit(true); // 触发 NoGitWarning 渲染
            }
          }
        }
      } catch (err) {
        if (!cancelled) {
          console.warn("[Player] Git 仓库检测失败:", err);
          setRepoPath(null);
        }
      }
    }

    detectRepoAndLoadInitialCode();

    return () => {
      cancelled = true;
    };
  }, [sessionCwd, setCode]);

  // 消息选中回调 (Story 2.7 AC #1, #6, FR-GIT-002)
  const handleMessageSelect = React.useCallback(
    (messageId: string, message: NarrativeMessage) => {
      setSelectedMessageId(messageId);
      // 同步更新时间轴位置
      const msgTime = new Date(message.timestamp).getTime();
      setCurrentTime(msgTime);

      // 更新时间旅行状态，触发 isHistoricalMode = true (Story 2.7 AC #6)
      const messageIndex = messages.findIndex((m) => m.id === messageId);
      jumpToMessage(messageIndex, messageId, msgTime);

      // 提取文件路径并获取代码快照 (FR-GIT-002, FR-GIT-003)
      if (repoPath) {
        const filePath = findRecentFilePath(messages, messageIndex);
        if (filePath) {
          // 将绝对路径转换为相对路径（相对于仓库根目录）
          const relativePath = filePath.startsWith(repoPath)
            ? filePath.slice(repoPath.length).replace(/^[/\\]/, "")
            : filePath;
          fetchSnapshot(relativePath, msgTime);
        }
      }
    },
    [messages, jumpToMessage, repoPath, fetchSnapshot]
  );

  // 时间轴 Seek 回调 (Story 2.6, 2.7, FR-GIT-002)
  const handleTimelineSeek = React.useCallback(
    (timestamp: number) => {
      setCurrentTime(timestamp);
      // 更新时间旅行状态 (Story 2.7 AC #2)
      setStoreCurrentTime(timestamp);

      // 找到最近的消息并选中
      const nearestEvent = timelineEvents.reduce<TimelineEvent | null>((nearest, event) => {
        if (!nearest) return event;
        const currentDiff = Math.abs(event.timestamp - timestamp);
        const nearestDiff = Math.abs(nearest.timestamp - timestamp);
        return currentDiff < nearestDiff ? event : nearest;
      }, null);

      if (nearestEvent && nearestEvent.messageIndex !== undefined) {
        const msg = messages[nearestEvent.messageIndex];
        if (msg) {
          setSelectedMessageId(msg.id);
          layoutRef.current?.scrollToMessage(msg.id);
          // 更新时间旅行状态 (Story 2.7 AC #7)
          jumpToMessage(nearestEvent.messageIndex, msg.id, timestamp);

          // 提取文件路径并获取代码快照 (FR-GIT-002, FR-GIT-003)
          if (repoPath) {
            const filePath = findRecentFilePath(messages, nearestEvent.messageIndex);
            if (filePath) {
              const relativePath = filePath.startsWith(repoPath)
                ? filePath.slice(repoPath.length).replace(/^[/\\]/, "")
                : filePath;
              fetchSnapshot(relativePath, timestamp);
            }
          }
        }
      }
    },
    [timelineEvents, messages, setStoreCurrentTime, jumpToMessage, repoPath, fetchSnapshot]
  );

  // 返回 Dashboard
  const handleBack = React.useCallback(() => {
    navigate("/");
  }, [navigate]);

  // DEV: 滚动到指定消息的调试函数 (可在 DevTools console 中调用 scrollToMessage('id'))
  React.useEffect(() => {
    if (import.meta.env.DEV) {
      (window as unknown as { scrollToMessage: (id: string) => void }).scrollToMessage = (id: string) => {
        layoutRef.current?.scrollToMessage(id);
      };
    }
    return () => {
      if (import.meta.env.DEV) {
        delete (window as unknown as { scrollToMessage?: unknown }).scrollToMessage;
      }
    };
  }, []);

  // 无效 sessionId 错误处理
  if (!sessionId) {
    return (
      <div className="h-screen flex flex-col bg-background">
        <header className="shrink-0 sticky top-0 z-50 w-full border-b border-border bg-background/95 backdrop-blur">
          <div className="flex h-14 items-center px-4">
            <Button variant="ghost" size="icon" onClick={handleBack}>
              <ArrowLeft className="h-5 w-5" />
            </Button>
            <span className="text-xl font-bold text-foreground ml-2">
              Mantra <span className="text-primary">心法</span>
            </span>
          </div>
        </header>
        <main className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <AlertCircle className="w-12 h-12 text-destructive mx-auto mb-4" />
            <h2 className="text-lg font-semibold text-foreground mb-2">会话未找到</h2>
            <p className="text-sm text-muted-foreground mb-4">请从项目列表中选择一个会话</p>
            <Button onClick={handleBack}>返回 Dashboard</Button>
          </div>
        </main>
      </div>
    );
  }

  // 加载中状态
  if (loading) {
    return (
      <div className="h-screen flex flex-col bg-background">
        <header className="shrink-0 sticky top-0 z-50 w-full border-b border-border bg-background/95 backdrop-blur">
          <div className="flex h-14 items-center px-4">
            <Button variant="ghost" size="icon" onClick={handleBack}>
              <ArrowLeft className="h-5 w-5" />
            </Button>
            <span className="text-xl font-bold text-foreground ml-2">
              Mantra <span className="text-primary">心法</span>
            </span>
          </div>
        </header>
        <main className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <Loader2 className="w-12 h-12 text-primary mx-auto mb-4 animate-spin" />
            <p className="text-sm text-muted-foreground">加载会话中...</p>
          </div>
        </main>
      </div>
    );
  }

  // 错误状态
  if (error) {
    return (
      <div className="h-screen flex flex-col bg-background">
        <header className="shrink-0 sticky top-0 z-50 w-full border-b border-border bg-background/95 backdrop-blur">
          <div className="flex h-14 items-center px-4">
            <Button variant="ghost" size="icon" onClick={handleBack}>
              <ArrowLeft className="h-5 w-5" />
            </Button>
            <span className="text-xl font-bold text-foreground ml-2">
              Mantra <span className="text-primary">心法</span>
            </span>
          </div>
        </header>
        <main className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <AlertCircle className="w-12 h-12 text-destructive mx-auto mb-4" />
            <h2 className="text-lg font-semibold text-foreground mb-2">加载失败</h2>
            <p className="text-sm text-muted-foreground mb-4">{error}</p>
            <Button onClick={handleBack}>返回 Dashboard</Button>
          </div>
        </main>
      </div>
    );
  }

  // 空消息状态
  if (messages.length === 0) {
    return (
      <div className="h-screen flex flex-col bg-background">
        <header className="shrink-0 sticky top-0 z-50 w-full border-b border-border bg-background/95 backdrop-blur">
          <div className="flex h-14 items-center px-4">
            <Button variant="ghost" size="icon" onClick={handleBack}>
              <ArrowLeft className="h-5 w-5" />
            </Button>
            <span className="text-xl font-bold text-foreground ml-2">
              Mantra <span className="text-primary">心法</span>
            </span>
          </div>
        </header>
        <main className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <AlertCircle className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
            <h2 className="text-lg font-semibold text-foreground mb-2">会话为空</h2>
            <p className="text-sm text-muted-foreground mb-4">这个会话没有任何消息</p>
            <Button onClick={handleBack}>返回 Dashboard</Button>
          </div>
        </main>
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col bg-background">
      {/* Header with Theme Toggle */}
      <header className="shrink-0 sticky top-0 z-50 w-full border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex h-14 items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="icon"
              onClick={handleBack}
              className="mr-2"
            >
              <ArrowLeft className="h-5 w-5" />
            </Button>
            <span className="text-xl font-bold text-foreground">
              Mantra <span className="text-primary">心法</span>
            </span>
            {sessionCwd && (
              <span className="text-sm text-muted-foreground ml-4 font-mono truncate max-w-md">
                {sessionCwd}
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <ThemeToggle />
          </div>
        </div>
      </header>

      {/* Main Content - DualStreamLayout */}
      <main className="flex-1 min-h-0 flex flex-col">
        <div className="flex-1 min-h-0 overflow-hidden">
          <DualStreamLayout
            ref={layoutRef}
            messages={messages}
            selectedMessageId={selectedMessageId}
            onMessageSelect={handleMessageSelect}
            // TimberLine 时间轴 Props (Story 2.6)
            showTimeline={false}
            timelineStartTime={timelineRange.startTime}
            timelineEndTime={timelineRange.endTime}
            timelineCurrentTime={currentTime}
            timelineEvents={timelineEvents}
            onTimelineSeek={handleTimelineSeek}
            // Story 2.11 AC6: 无 Git 仓库时显示警告
            showNoGitWarning={hasNoGit}
            projectPath={sessionCwd}
          />
        </div>
        {/* 直接在 Player 层渲染 TimberLine */}
        {messages.length > 0 && (
          <TimberLine
            startTime={timelineRange.startTime}
            endTime={timelineRange.endTime}
            currentTime={currentTime}
            events={timelineEvents}
            onSeek={handleTimelineSeek}
          />
        )}
      </main>
    </div>
  );
}
