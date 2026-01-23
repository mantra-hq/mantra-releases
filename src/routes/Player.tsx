/**
 * Player Page - 会话回放页面
 * Story 2.8: Task 1
 * Story 2.10: Task 6 (Message Navigation from Search)
 * Story 2.11: Task 6 (Initial Code Display)
 * Story 2.12: Task 2 (Smart File Selection Logic)
 * Story 2.17: TopBar 面包屑导航
 * Story 2.18: ProjectDrawer 项目抽屉
 * Story 2.32: Git 提交标记集成到时间轴
 *
 * 封装 DualStreamLayout，用于播放会话内容
 *
 * 从后端加载真实会话数据并转换为前端格式
 * 集成 Git Time Machine 实现代码快照功能 (FR-GIT)
 * 进入时自动显示项目代表性文件 (AC1, AC2)
 * 智能文件选择：自动显示最相关的代码文件 (Story 2.12)
 * 统一标签管理：会话点击时打开历史版本标签
 */

import * as React from "react";
import { useParams, useNavigate, useSearchParams } from "react-router-dom";
// Story 9.4: 使用 IPC 适配器支持 E2E 测试环境
import { invoke } from "@/lib/ipc-adapter";
import { DualStreamLayout, type DualStreamLayoutRef } from "@/components/layout";
import { convertSessionToMessages, type MantraSession } from "@/lib/session-utils";
import { getProjectByCwd, getRepresentativeFile, detectGitRepo, syncProject, getCommitsInRange, type CommitInfo } from "@/lib/project-ipc";
import type { NarrativeMessage } from "@/types/message";
import {
  messagesToTimelineEvents,
  getTimelineRange,
  type TimelineEvent,
} from "@/types/timeline";
import { TimberLine } from "@/components/timeline";
import { AlertCircle, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useTimeTravelStore } from "@/stores/useTimeTravelStore";
import { useSearchStore } from "@/stores/useSearchStore";
import { useEditorStore } from "@/stores/useEditorStore";
import { useDetailPanelStore } from "@/stores/useDetailPanelStore";
import { useTimeMachine, type SnapshotResult } from "@/hooks/useTimeMachine";
// Story 8.12: 使用 tool-utils 模块的文件路径工具
import {
  findRecentFilePathEnhanced,
  toRelativePath,
} from "@/lib/tool-utils";
// Story 2.30: 会话日志文件提取
import { extractFileFromSession } from "@/lib/session-file-extractor";
// Story 2.17: TopBar 面包屑导航
import { TopBar } from "@/components/navigation";
import { ImportWizard } from "@/components/import";
import { useCurrentSession } from "@/hooks";
// Story 2.18: ProjectDrawer 项目抽屉
import { ProjectDrawer } from "@/components/sidebar";
import { showSyncResult } from "@/components/sidebar/SyncResultToast";
import { useProjectDrawer } from "@/hooks/useProjectDrawer";
// Story 2.21: Player 空状态组件
// Story 10.11: 移除 ModeSwitch (已集成到 TopBar)
import { PlayerEmptyState, CompressGuideDialog } from "@/components/player";
// Story 10.2/10.3/10.6: 压缩模式组件
import { CompressModeContent } from "@/components/compress";
// Story 10.3: 压缩状态 Provider
import { CompressStateProvider } from "@/hooks/useCompressState";
// Story 2.29 V2: 隐藏空会话设置
import { useHideEmptyProjects } from "@/hooks/useHideEmptyProjects";
// Story 2.34: 统计视图组件
import { useAppModeStore } from "@/stores/useAppModeStore";
import { ProjectStatsView, SessionStatsView, StatsLevelTabs, type StatsLevel } from "@/components/analytics";
// Story 10.1: 压缩模式 hook
import { useCompressMode } from "@/hooks";


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
  const resetTimeTravel = useTimeTravelStore((state) => state.reset);

  // 编辑器标签管理
  const openTab = useEditorStore((state) => state.openTab);
  const closeAllTabs = useEditorStore((state) => state.closeAllTabs);

  // 右侧面板 Tab 管理 (修复 Bash 详情后其他消息点击无响应问题)
  const setActiveRightTab = useDetailPanelStore((state) => state.setActiveRightTab);
  const resetDetailPanel = useDetailPanelStore((state) => state.reset);

  // Story 2.30: 会话日志回退函数
  const sessionFallback = React.useCallback(
    (filePath: string, timestamp: number): SnapshotResult | null => {
      if (messages.length === 0) return null;

      // 找到最接近 timestamp 的消息索引
      const targetTime = timestamp;
      let nearestIndex = 0;
      let nearestDiff = Math.abs(new Date(messages[0].timestamp).getTime() - targetTime);

      for (let i = 1; i < messages.length; i++) {
        const diff = Math.abs(new Date(messages[i].timestamp).getTime() - targetTime);
        if (diff < nearestDiff) {
          nearestDiff = diff;
          nearestIndex = i;
        }
      }

      // 尝试从会话消息中提取文件内容
      const result = extractFileFromSession(messages, filePath, nearestIndex);
      if (result) {
        return {
          content: result.content,
          commit_hash: "",
          commit_message: "",
          commit_timestamp: Math.floor(result.timestamp / 1000),
          source: "session",
        };
      }

      return null;
    },
    [messages]
  );

  // Git Time Machine Hook (FR-GIT-002, FR-GIT-003, Story 2.30: 会话回退)
  const { fetchSnapshot } = useTimeMachine(repoPath, sessionFallback);

  // Story 2.12 AC4: 记录最近有效的文件路径，用于无文件路径时保持视图
  const lastValidFileRef = React.useRef<string | null>(null);
  // 记录每个文件的上一个内容（用于 Diff）- 按文件路径区分，避免不同文件之间错误 diff
  const previousContentMapRef = React.useRef<Map<string, string>>(new Map());

  // Story 2.17: 获取当前会话和项目信息
  const {
    session: currentSession,
    project: currentProject,
    sessions: projectSessions,
    refetch: refetchCurrentSession,
  } = useCurrentSession(sessionId);

  // Story 2.17: ImportWizard 状态
  const [importOpen, setImportOpen] = React.useState(false);

  // 同步状态
  const [isSyncing, setIsSyncing] = React.useState(false);

  // Story 2.18: ProjectDrawer 状态
  // Story 2.21 AC #2: 无 sessionId 时默认展开抽屉
  // Story 1.12: 改用逻辑项目视图
  const {
    isOpen: drawerOpen,
    setIsOpen: setDrawerOpen,
    logicalProjects,
    isLoading: projectsLoading,
    getLogicalProjectSessions: fetchLogicalProjectSessions,
    refetchProjects,
  } = useProjectDrawer({ defaultOpen: !sessionId });

  // Bug Fix V6: 根据 currentProject.id 查找对应的逻辑项目
  // 使用逻辑项目的 display_name 和 has_git_repo 替代存储层项目的数据
  // 这样关联路径后，导航栏项目名和 Git 状态能正确更新
  const currentLogicalProject = React.useMemo(() => {
    if (!currentProject?.id) return null;
    return logicalProjects.find(lp => lp.project_ids.includes(currentProject.id)) ?? null;
  }, [currentProject?.id, logicalProjects]);

  // Story 2.29 V2: 隐藏空会话设置（与 ProjectDrawer 同步）
  const [hideEmptySessions] = useHideEmptyProjects();

  // Story 2.34 + 10.11: 统一应用模式状态（回放/统计/压缩）
  const appMode = useAppModeStore((state) => state.mode);
  const setAppMode = useAppModeStore((state) => state.setMode);

  // Story 2.34: 统计视图层级状态（项目/会话）
  const [statsLevel, setStatsLevel] = React.useState<StatsLevel>("session");

  // Story 10.1 + 10.11: 压缩模式引导弹窗状态（保留首次引导逻辑）
  const {
    isFirstTimeCompress,
    hideGuide,
    dismissGuide,
  } = useCompressMode({ sessionId: sessionId ?? "" });

  // Story 10.11 AC4: URL 参数同步 - 初始化时读取 mode 参数
  // 用于支持 deep link：/player/123?mode=compress
  React.useEffect(() => {
    const modeParam = searchParams.get("mode");
    if (modeParam && ["playback", "analytics", "compress"].includes(modeParam)) {
      // 无 sessionId 时不允许进入压缩模式
      if (modeParam === "compress" && !sessionId) {
        setAppMode("playback");
      } else {
        setAppMode(modeParam as "playback" | "analytics" | "compress");
      }
    }
    // 只在首次加载时执行
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Story 10.11 AC4: URL 参数同步 - 模式变化时更新 URL
  React.useEffect(() => {
    const currentModeParam = searchParams.get("mode");
    if (currentModeParam !== appMode) {
      const newParams = new URLSearchParams(searchParams);
      newParams.set("mode", appMode);
      setSearchParams(newParams, { replace: true });
    }
  }, [appMode, searchParams, setSearchParams]);

  // Bug Fix: 切换项目时重置右侧内容区域状态
  // 当 sessionId 变化时，清空所有关联的 store 状态，避免显示旧项目内容
  React.useEffect(() => {
    // 重置时间旅行状态（代码内容、文件路径等）
    resetTimeTravel();
    // 重置右侧面板状态（终端、工具详情等）
    resetDetailPanel();
    // 关闭所有编辑器标签页
    closeAllTabs();
    // 重置 ref 状态
    lastValidFileRef.current = null;
    previousContentMapRef.current.clear();
    // 重置滚动目标标记
    hasScrolledToTargetRef.current = false;
  }, [sessionId, resetTimeTravel, resetDetailPanel, closeAllTabs]);

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

  // Story 2.32: 加载 Git 提交并合并到时间轴事件
  React.useEffect(() => {
    // 只有当有 repoPath 和有效的时间范围时才加载
    if (!repoPath || timelineRange.startTime >= timelineRange.endTime) {
      return;
    }

    let cancelled = false;

    async function loadGitCommits() {
      try {
        // 将毫秒转换为秒
        const startSec = Math.floor(timelineRange.startTime / 1000);
        const endSec = Math.ceil(timelineRange.endTime / 1000);

        const commits = await getCommitsInRange(repoPath!, startSec, endSec);

        if (cancelled) return;

        if (commits.length > 0) {
          // 将 commits 转换为 TimelineEvent
          const commitEvents: TimelineEvent[] = commits.map((commit: CommitInfo) => ({
            timestamp: new Date(commit.committed_at).getTime(),
            type: 'git-commit' as const,
            commitHash: commit.commit_hash,
            label: commit.message.split('\n')[0], // 取首行作为标签
          }));

          // 从消息生成的事件（不包含 git-commit）
          const messageEventsOnly = messagesToTimelineEvents(messages);

          // 合并消息事件和 Git 事件，按时间排序
          const allEvents = [...messageEventsOnly, ...commitEvents]
            .sort((a, b) => a.timestamp - b.timestamp);

          setTimelineEvents(allEvents);
          console.log(`[Player] 加载了 ${commits.length} 个 Git 提交到时间轴`);
        }
      } catch (err) {
        // 静默失败，仅显示消息事件 (AC3: 无 Git 仓库优雅降级)
        console.warn("[Player] 加载 Git 提交失败:", err);
      }
    }

    loadGitCommits();

    return () => {
      cancelled = true;
    };
  }, [repoPath, timelineRange.startTime, timelineRange.endTime, messages]);

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
      // 使用会话的真正 title（如果有的话）
      const sessionTitle = currentSession?.metadata?.title;
      addRecentSession({
        projectId: sessionCwd, // 使用 cwd 作为临时 projectId
        projectName,
        sessionId,
        sessionName: sessionTitle || sessionId,
        accessedAt: Date.now(),
      });
    }
  }, [sessionId, sessionCwd, loading, messages.length, currentSession, addRecentSession]);

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

  // Story 1.9: 当项目信息变化时更新 repoPath（修复 cwd 更新后 git 仓库信息不同步问题）
  // Bug Fix V6: 优先使用逻辑项目的 physical_path 检测 Git 仓库
  React.useEffect(() => {
    // 优先使用逻辑项目的物理路径（关联后会更新）
    const effectivePath = currentLogicalProject?.physical_path ?? currentProject?.cwd;
    const effectiveHasGit = currentLogicalProject?.has_git_repo ?? currentProject?.has_git_repo;

    if (!effectivePath) return;

    // 如果逻辑项目标记有 Git 仓库，或者存储层项目有 Git 信息
    if (effectiveHasGit && currentProject?.git_repo_path) {
      setRepoPath(currentProject.git_repo_path);
      setHasNoGit(false);
      console.log("[Player] Git 仓库信息已更新 (from currentProject):", currentProject.git_repo_path);

      // 重新加载代表性文件
      getRepresentativeFile(currentProject.git_repo_path)
        .then((repFile) => {
          if (repFile) {
            setCode(repFile.content, repFile.path);
            console.log("[Player] 代表性文件已重新加载:", repFile.path);
          }
        })
        .catch((err) => {
          console.warn("[Player] 代表性文件加载失败:", err);
        });
    } else if (currentLogicalProject?.has_git_repo && !currentProject?.git_repo_path) {
      // Bug Fix V6: 逻辑项目标记有 Git，但存储层没有 - 重新检测
      // 这发生在关联新路径后，physical_path 更新但存储层还没同步
      detectGitRepo(currentLogicalProject.physical_path)
        .then((detected) => {
          if (detected) {
            setRepoPath(detected);
            setHasNoGit(false);
            console.log("[Player] Git 仓库重新检测成功:", detected);

            // 加载代表性文件
            getRepresentativeFile(detected)
              .then((repFile) => {
                if (repFile) {
                  setCode(repFile.content, repFile.path);
                  console.log("[Player] 代表性文件已加载:", repFile.path);
                }
              })
              .catch((err) => {
                console.warn("[Player] 代表性文件加载失败:", err);
              });
          }
        })
        .catch((err) => {
          console.warn("[Player] Git 仓库检测失败:", err);
        });
    } else {
      setRepoPath(null);
      setHasNoGit(true);
    }
  }, [currentProject?.id, currentProject?.git_repo_path, currentProject?.has_git_repo, currentLogicalProject?.physical_path, currentLogicalProject?.has_git_repo, setCode]);

  // 消息选中回调 (Story 2.7 AC #1, #6, FR-GIT-002, Story 2.12)
  // 统一标签管理：会话点击时打开历史版本标签
  const handleMessageSelect = React.useCallback(
    async (messageId: string, message: NarrativeMessage) => {
      setSelectedMessageId(messageId);
      // 同步更新时间轴位置
      const msgTime = new Date(message.timestamp).getTime();
      setCurrentTime(msgTime);

      // 更新时间旅行状态，触发 isHistoricalMode = true (Story 2.7 AC #6)
      const messageIndex = messages.findIndex((m) => m.id === messageId);
      jumpToMessage(messageIndex, messageId, msgTime);

      // Story 2.12: 增强的文件路径提取 (AC #1, #2, #3, #4, #6, #7)
      if (repoPath) {
        const fileResult = findRecentFilePathEnhanced(messages, messageIndex);

        // 只有当文件直接来自当前消息时才切换（非 history 来源）
        // 从历史中找到的文件不应该影响当前视图
        const shouldSwitchFile = fileResult && fileResult.source !== "history";

        if (shouldSwitchFile && fileResult) {
          // AC #6: 使用增强的绝对路径转相对路径逻辑
          const relativePath = toRelativePath(fileResult.path, repoPath);

          // 记录有效文件路径 (AC #4)
          lastValidFileRef.current = relativePath;

          // 获取代码快照 (FR-GIT-002, FR-GIT-003)
          const snapshot = await fetchSnapshot(relativePath, msgTime);

          if (snapshot) {
            // 获取该文件的上一个版本内容（仅同文件才进行 diff）
            const previousContent = previousContentMapRef.current.get(relativePath);
            
            // 统一标签管理：打开历史版本标签
            openTab(relativePath, {
              preview: true,
              commitHash: snapshot.commit_hash,
              timestamp: snapshot.commit_timestamp * 1000,
              content: snapshot.content,
              previousContent: previousContent,
            });

            // 切换右侧面板到代码 Tab (修复 Bash 详情后其他消息点击无响应问题)
            setActiveRightTab("code");

            // 更新该文件的 previousContent 用于下次 Diff
            previousContentMapRef.current.set(relativePath, snapshot.content);
          }

          console.log(
            `[Player] 文件选择: ${relativePath} (来源: ${fileResult.source}, 置信度: ${fileResult.confidence})`
          );
        } else if (fileResult && !shouldSwitchFile) {
          // 历史来源的文件，不切换，保持当前视图
          console.log(
            `[Player] 跳过历史文件: ${fileResult.path} (来源: ${fileResult.source})`
          );
        } else if (lastValidFileRef.current) {
          // AC #4: 无文件路径时保持当前视图，仅更新时间点
          console.log("[Player] 无文件路径，保持当前视图:", lastValidFileRef.current);
          // 不调用 fetchSnapshot，保持当前代码内容
        }
      }
    },
    [messages, jumpToMessage, repoPath, fetchSnapshot, openTab, setActiveRightTab]
  );

  // 时间轴 Seek 回调 (Story 2.6, 2.7, FR-GIT-002, Story 2.12, Story 2.32)
  // 统一标签管理：时间轴拖动时打开历史版本标签
  const handleTimelineSeek = React.useCallback(
    async (timestamp: number) => {
      setCurrentTime(timestamp);
      // 更新时间旅行状态 (Story 2.7 AC #2)
      setStoreCurrentTime(timestamp);

      // Story 2.32: 找到该时间点之前（或最近的）消息事件
      // Git 提交点击时也应该联动到最近的消息
      const messageEvents = timelineEvents.filter(e => e.messageIndex !== undefined);
      let nearestMessageEvent: TimelineEvent | null = null;
      
      // 优先找时间点之前或等于的最近消息
      const eventsBeforeOrAt = messageEvents.filter(e => e.timestamp <= timestamp);
      if (eventsBeforeOrAt.length > 0) {
        nearestMessageEvent = eventsBeforeOrAt.reduce((nearest, event) => {
          return event.timestamp > nearest.timestamp ? event : nearest;
        });
      } else {
        // 如果没有之前的消息，找之后最近的
        const eventsAfter = messageEvents.filter(e => e.timestamp > timestamp);
        if (eventsAfter.length > 0) {
          nearestMessageEvent = eventsAfter.reduce((nearest, event) => {
            return event.timestamp < nearest.timestamp ? event : nearest;
          });
        }
      }

      if (nearestMessageEvent && nearestMessageEvent.messageIndex !== undefined) {
        const msg = messages[nearestMessageEvent.messageIndex];
        if (msg) {
          setSelectedMessageId(msg.id);
          layoutRef.current?.scrollToMessage(msg.id);
          // 更新时间旅行状态 (Story 2.7 AC #7)
          jumpToMessage(nearestMessageEvent.messageIndex, msg.id, timestamp);

          // Story 2.12: 增强的文件路径提取
          if (repoPath) {
            const fileResult = findRecentFilePathEnhanced(
              messages,
              nearestMessageEvent.messageIndex!
            );

            // 只有当文件直接来自当前消息时才切换（非 history 来源）
            const shouldSwitchFile = fileResult && fileResult.source !== "history";

            if (shouldSwitchFile && fileResult) {
              const relativePath = toRelativePath(fileResult.path, repoPath);
              lastValidFileRef.current = relativePath;

              // 获取快照并打开历史标签
              const snapshot = await fetchSnapshot(relativePath, timestamp);
              if (snapshot) {
                // 获取该文件的上一个版本内容（仅同文件才进行 diff）
                const previousContent = previousContentMapRef.current.get(relativePath);
                
                openTab(relativePath, {
                  preview: true,
                  commitHash: snapshot.commit_hash,
                  timestamp: snapshot.commit_timestamp * 1000,
                  content: snapshot.content,
                  previousContent: previousContent,
                });
                // 切换右侧面板到代码 Tab
                setActiveRightTab("code");
                // 更新该文件的 previousContent 用于下次 Diff
                previousContentMapRef.current.set(relativePath, snapshot.content);
              }
            } else if (lastValidFileRef.current) {
              // AC #4: 无文件路径时保持当前视图
              console.log("[Player] 时间轴 Seek: 无文件路径，保持当前视图");
            }
          }
        }
      }
    },
    [timelineEvents, messages, setStoreCurrentTime, jumpToMessage, repoPath, fetchSnapshot, openTab, setActiveRightTab]
  );

  // 返回首页
  const handleBack = React.useCallback(() => {
    navigate("/");
  }, [navigate]);

  // Story 2.17 & 2.18: TopBar 回调函数
  // 打开 ProjectDrawer (AC2, AC3)
  const handleDrawerOpen = React.useCallback(() => {
    setDrawerOpen(true);
  }, [setDrawerOpen]);

  // 会话切换 (AC9)
  const handleSessionSelect = React.useCallback(
    (newSessionId: string) => {
      if (newSessionId !== sessionId) {
        navigate(`/player/${newSessionId}`);
      }
    },
    [sessionId, navigate]
  );

  // 同步项目 (AC10)
  const handleSync = React.useCallback(async () => {
    if (!currentProject?.id || isSyncing) return;

    setIsSyncing(true);
    try {
      const result = await syncProject(currentProject.id);
      showSyncResult(currentProject.name, result);

      // 同步成功后刷新数据
      if (result.new_sessions.length > 0 || result.updated_sessions.length > 0) {
        refetchCurrentSession();
        refetchProjects();
      }
    } catch (error) {
      showSyncResult(currentProject.name, null, error as Error);
    } finally {
      setIsSyncing(false);
    }
  }, [currentProject, isSyncing, refetchCurrentSession, refetchProjects]);

  // 导入完成回调
  const handleImportComplete = React.useCallback(() => {
    setImportOpen(false);
    refetchCurrentSession();
    refetchProjects();
  }, [refetchCurrentSession, refetchProjects]);

  // 导入对话框关闭回调（无论如何关闭都刷新项目列表）
  const handleImportOpenChange = React.useCallback((open: boolean) => {
    setImportOpen(open);
    // 关闭对话框时刷新项目列表
    if (!open) {
      refetchProjects();
    }
  }, [refetchProjects]);

  // Story 2.18: 抽屉中的会话选择回调 (AC10, AC11)
  // Story 2.21 AC #3: 选择会话后抽屉自动关闭
  const handleDrawerSessionSelect = React.useCallback(
    (newSessionId: string, _projectId: string) => {
      if (newSessionId !== sessionId) {
        setDrawerOpen(false); // AC #3: 关闭抽屉
        navigate(`/player/${newSessionId}`);
      }
    },
    [sessionId, navigate, setDrawerOpen]
  );

  // Story 2.18: 抽屉中的导入按钮回调
  const handleDrawerImport = React.useCallback(() => {
    setImportOpen(true);
  }, []);

  // Story 2.18 Task 9: 全局快捷键 Cmd/Ctrl+Shift+P 打开抽屉 (AC1)
  React.useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Cmd+Shift+P (macOS) 或 Ctrl+Shift+P (Windows/Linux)
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key.toLowerCase() === "p") {
        e.preventDefault();
        setDrawerOpen(true);
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [setDrawerOpen]);

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

  // 无效 sessionId 错误处理 → Story 2.21: 改为显示空状态 (AC #1, #4)
  // Story 2.34: 统计模式下即使无 sessionId 也可显示项目统计
  if (!sessionId) {
    return (
      <div className="h-screen flex flex-col bg-background">
        {/* Story 2.21 AC #4: 简化版 TopBar（无会话信息） */}
        <TopBar
          mode="minimal"
          onDrawerOpen={() => setDrawerOpen(true)}
          onImport={() => setImportOpen(true)}
        />
        {/* Story 2.34: 根据模式显示不同内容 */}
        <main className="flex-1 min-h-0">
          {appMode === "analytics" && logicalProjects.length > 0 ? (
            /* 统计模式下显示第一个项目的统计（或提示选择项目） */
            /* Story 1.12: 使用第一个逻辑项目的第一个存储层项目 ID */
            <ProjectStatsView
              projectId={logicalProjects[0]?.project_ids[0] ?? ""}
              projectName={logicalProjects[0]?.display_name}
              onImport={() => setImportOpen(true)}
            />
          ) : (
            /* 回放模式或无项目时显示空状态 */
            <PlayerEmptyState
              hasProjects={logicalProjects.length > 0}
              onOpenDrawer={() => setDrawerOpen(true)}
              onImport={() => setImportOpen(true)}
            />
          )}
        </main>
        {/* Import Wizard Modal */}
        <ImportWizard
          open={importOpen}
          onOpenChange={handleImportOpenChange}
          onComplete={handleImportComplete}
        />
        {/* ProjectDrawer 项目抽屉 - Story 1.12: 使用逻辑项目视图 */}
        <ProjectDrawer
          isOpen={drawerOpen}
          onOpenChange={setDrawerOpen}
          logicalProjects={logicalProjects}
          isLoading={projectsLoading}
          currentSessionId={undefined}
          onSessionSelect={handleDrawerSessionSelect}
          onImportClick={handleDrawerImport}
          getLogicalProjectSessions={fetchLogicalProjectSessions}
          onProjectsChange={() => {
            // Story 1.9: 刷新项目列表和当前会话信息
            refetchProjects();
            refetchCurrentSession();
          }}
        />
      </div>
    );
  }

  // 加载中状态
  if (loading) {
    return (
      <div className="h-screen flex flex-col bg-background">
        <TopBar mode="loading" onBack={handleBack} />
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
        <TopBar mode="error" onBack={handleBack} />
        <main className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <AlertCircle className="w-12 h-12 text-destructive mx-auto mb-4" />
            <h2 className="text-lg font-semibold text-foreground mb-2">加载失败</h2>
            <p className="text-sm text-muted-foreground mb-4">{error}</p>
            <Button onClick={handleBack}>返回首页</Button>
          </div>
        </main>
      </div>
    );
  }

  // 空消息状态
  if (messages.length === 0) {
    return (
      <div className="h-screen flex flex-col bg-background">
        <TopBar mode="error" onBack={handleBack} />
        <main className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <AlertCircle className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
            <h2 className="text-lg font-semibold text-foreground mb-2">会话为空</h2>
            <p className="text-sm text-muted-foreground mb-4">这个会话没有任何消息</p>
            <Button onClick={handleBack}>返回首页</Button>
          </div>
        </main>
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col bg-background">
      {/* Story 2.17: TopBar 面包屑导航 */}
      {/* Bug Fix V6: 使用逻辑项目的 display_name 替代存储层项目名 */}
      <TopBar
        sessionId={sessionId}
        sessionName={projectSessions.find(s => s.id === sessionId)?.name ?? `Session ${sessionId.slice(0, 8)}`}
        messageCount={messages.length}
        projectId={currentProject?.id ?? ""}
        projectName={currentLogicalProject?.display_name ?? currentProject?.name ?? sessionCwd?.split("/").pop() ?? "项目"}
        sessions={projectSessions}
        onDrawerOpen={handleDrawerOpen}
        onSessionSelect={handleSessionSelect}
        onSync={handleSync}
        onImport={() => setImportOpen(true)}
        isSyncing={isSyncing}
        hideEmptySessions={hideEmptySessions}
      />

      {/* Main Content - Story 2.34: 根据模式切换显示 */}
      <main className="flex-1 min-h-0 flex flex-col">
        {appMode === "analytics" ? (
          /* 统计模式: 显示统计视图 */
          <div className="flex-1 min-h-0 overflow-hidden flex flex-col">
            {/* Story 2.34: 统计层级切换 Tabs (仅当有 sessionId 和 project 时显示) */}
            {sessionId && currentProject?.id && (
              <div className="flex-shrink-0 px-6 pt-4">
                <StatsLevelTabs
                  value={statsLevel}
                  onChange={setStatsLevel}
                  projectName={currentProject.name}
                  sessionName={projectSessions.find(s => s.id === sessionId)?.name}
                />
              </div>
            )}

            {/* 统计视图内容 */}
            <div className="flex-1 min-h-0 overflow-auto">
              {sessionId && currentProject?.id ? (
                /* 有选中会话和项目时，根据 statsLevel 显示 */
                statsLevel === "session" ? (
                  <SessionStatsView sessionId={sessionId} />
                ) : (
                  <ProjectStatsView
                    projectId={currentProject.id}
                    projectName={currentProject.name}
                    onImport={() => setImportOpen(true)}
                  />
                )
              ) : currentProject?.id ? (
                /* 仅选中项目时显示项目统计 (AC2) */
                <ProjectStatsView
                  projectId={currentProject.id}
                  projectName={currentProject.name}
                  onImport={() => setImportOpen(true)}
                />
              ) : (
                /* 无选中时显示空状态 */
                <ProjectStatsView
                  projectId=""
                  onImport={() => setImportOpen(true)}
                />
              )}
            </div>
          </div>
        ) : (
          /* Story 10.11: 回放/压缩模式 - ModeSwitch 已移至 TopBar */
          /* 内容区域 - 根据统一的 appMode 切换 */
          <>
            {appMode === "playback" ? (
              /* 回放模式: 显示 DualStreamLayout */
              <>
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
                    // Story 2.13: 文件浏览器
                    repoPath={repoPath ?? undefined}
                  />
                </div>
                {/* 直接在 Player 层渲染 TimberLine (AC #5: 回放模式显示时间轴) */}
                {messages.length > 0 && (
                  <TimberLine
                    startTime={timelineRange.startTime}
                    endTime={timelineRange.endTime}
                    currentTime={currentTime}
                    events={timelineEvents}
                    onSeek={handleTimelineSeek}
                  />
                )}
              </>
            ) : appMode === "compress" ? (
              /* Story 10.11: 压缩模式 - 显示原始消息列表 + 压缩预览 */
              /* Story 10.3: 使用 CompressStateProvider 包裹实现左右面板状态共享 */
              /* Story 10.9: 使用 CompressModeContent 处理持久化和 beforeunload */
              <CompressStateProvider>
                <CompressModeContent
                  layoutRef={layoutRef}
                  messages={messages}
                  sessionId={sessionId}
                />
              </CompressStateProvider>
              /* 压缩模式不显示 TimberLine */
            ) : null}

            {/* Story 10.1 AC #2: 首次使用引导弹窗 */}
            <CompressGuideDialog
              open={isFirstTimeCompress}
              onClose={hideGuide} // 临时隐藏弹窗，下次还会显示
              onDismissForever={dismissGuide}
            />
          </>
        )}
      </main>

      {/* Story 2.17: Import Wizard Modal */}
      <ImportWizard
        open={importOpen}
        onOpenChange={handleImportOpenChange}
        onComplete={handleImportComplete}
      />

      {/* Story 2.18 + 1.12: ProjectDrawer 项目抽屉 - 使用逻辑项目视图 */}
      <ProjectDrawer
        isOpen={drawerOpen}
        onOpenChange={setDrawerOpen}
        logicalProjects={logicalProjects}
        isLoading={projectsLoading}
        currentSessionId={sessionId}
        currentPhysicalPath={currentProject?.cwd}
        onSessionSelect={handleDrawerSessionSelect}
        onImportClick={handleDrawerImport}
        getLogicalProjectSessions={fetchLogicalProjectSessions}
        onProjectsChange={() => {
          // Story 1.9: 刷新项目列表和当前会话信息（修复项目 cwd 更新后导航栏不同步问题）
          refetchProjects();
          refetchCurrentSession();
        }}
        onCurrentProjectRemoved={() => navigate("/player")}
      />
    </div>
  );
}

