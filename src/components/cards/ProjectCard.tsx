/**
 * ProjectCard Component - 项目卡片组件
 * Story 2.8: Task 2
 *
 * 展示项目信息，支持展开/折叠显示会话列表
 * 展开时按需加载会话数据
 */

import * as React from "react";
import * as Collapsible from "@radix-ui/react-collapsible";
import { Folder, ChevronDown, Loader2 } from "lucide-react";
import { formatDistanceToNow } from "date-fns";
import { zhCN } from "date-fns/locale";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "@/lib/utils";
import type { Project, Session } from "@/types/project";
import { SessionCard } from "./SessionCard";

/**
 * ProjectCard Props
 */
export interface ProjectCardProps {
  /** 项目数据 */
  project: Project;
  /** 是否展开 */
  isExpanded: boolean;
  /** 展开/折叠切换回调 */
  onToggle: () => void;
  /** 点击会话回调 */
  onSessionClick: (sessionId: string) => void;
}

/**
 * ProjectCard 组件
 * 显示项目名称、会话数、最后活动时间，支持展开显示会话列表
 */
export function ProjectCard({
  project,
  isExpanded,
  onToggle,
  onSessionClick,
}: ProjectCardProps) {
  // 会话列表状态 (按需加载)
  const [sessions, setSessions] = React.useState<Session[]>([]);
  const [sessionsLoading, setSessionsLoading] = React.useState(false);
  const [sessionsLoaded, setSessionsLoaded] = React.useState(false);

  // 格式化相对时间
  const relativeTime = React.useMemo(() => {
    return formatDistanceToNow(new Date(project.last_activity), {
      addSuffix: true,
      locale: zhCN,
    });
  }, [project.last_activity]);

  // 展开时加载会话
  React.useEffect(() => {
    if (isExpanded && !sessionsLoaded && !sessionsLoading) {
      setSessionsLoading(true);
      invoke<Session[]>("get_project_sessions", { projectId: project.id })
        .then((data) => {
          setSessions(data);
          setSessionsLoaded(true);
        })
        .catch((err) => {
          console.error("获取会话列表失败:", err);
        })
        .finally(() => {
          setSessionsLoading(false);
        });
    }
  }, [isExpanded, sessionsLoaded, sessionsLoading, project.id]);

  return (
    <Collapsible.Root
      open={isExpanded}
      onOpenChange={onToggle}
      data-testid="project-card"
      className={cn(
        "bg-card border border-border rounded-lg overflow-hidden",
        "transition-colors duration-150"
      )}
    >
      {/* 卡片头部 - 可点击触发展开/折叠 */}
      <Collapsible.Trigger asChild>
        <button
          type="button"
          className={cn(
            "w-full flex items-center justify-between p-4",
            "cursor-pointer",
            "transition-colors duration-150",
            "hover:bg-muted/50",
            "focus:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-inset",
            "text-left"
          )}
          aria-label={project.name}
        >
          {/* 左侧: 图标 + 项目信息 */}
          <div className="flex items-center gap-3">
            {/* 项目图标 */}
            <div
              className={cn(
                "w-10 h-10 flex items-center justify-center",
                "bg-primary/10 rounded-lg",
                "text-primary"
              )}
            >
              <Folder className="w-5 h-5" />
            </div>

            {/* 项目名称和元信息 */}
            <div className="flex flex-col">
              <span className="text-base font-semibold text-foreground">
                {project.name}
              </span>
              <div className="flex items-center gap-3 text-sm text-muted-foreground">
                <span>{project.session_count} 会话</span>
                <span>·</span>
                <span>{relativeTime}</span>
              </div>
            </div>
          </div>

          {/* 右侧: 展开指示器 */}
          <ChevronDown
            data-expanded={isExpanded}
            className={cn(
              "w-5 h-5 text-muted-foreground",
              "transition-transform duration-200",
              isExpanded && "rotate-180"
            )}
          />
        </button>
      </Collapsible.Trigger>

      {/* 会话列表 - 展开时显示 */}
      <Collapsible.Content className="data-[state=open]:animate-collapsible-down data-[state=closed]:animate-collapsible-up">
        <div className="border-t border-border p-2 space-y-1">
          {/* 加载中 */}
          {sessionsLoading && (
            <div className="px-3 py-4 flex items-center justify-center gap-2 text-sm text-muted-foreground">
              <Loader2 className="w-4 h-4 animate-spin" />
              <span>加载会话...</span>
            </div>
          )}

          {/* 会话列表 */}
          {!sessionsLoading &&
            sessions.map((session) => (
              <SessionCard
                key={session.id}
                session={session}
                onClick={() => onSessionClick(session.id)}
              />
            ))}

          {/* 空状态 */}
          {!sessionsLoading && sessionsLoaded && sessions.length === 0 && (
            <div className="px-3 py-4 text-sm text-muted-foreground text-center">
              暂无会话
            </div>
          )}
        </div>
      </Collapsible.Content>
    </Collapsible.Root>
  );
}
