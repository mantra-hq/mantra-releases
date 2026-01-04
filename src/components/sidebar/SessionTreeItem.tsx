/**
 * SessionTreeItem Component - 会话树节点
 * Story 2.18: Task 4
 *
 * 会话节点，显示会话信息和当前选中状态
 */

import * as React from "react";
import { MessageCircle } from "lucide-react";
import { formatDistanceToNow } from "date-fns";
import { zhCN } from "date-fns/locale";
import { cn } from "@/lib/utils";
import { HighlightText } from "./DrawerSearch";
import type { SessionSummary } from "./types";

/**
 * SessionTreeItem Props
 */
export interface SessionTreeItemProps {
  /** 会话信息 */
  session: SessionSummary;
  /** 是否当前选中 */
  isCurrent: boolean;
  /** 搜索关键词（用于高亮） */
  searchKeyword?: string;
  /** 点击回调 */
  onClick: () => void;
}

/**
 * 获取会话来源图标颜色
 */
function getSourceColor(source: string): string {
  switch (source) {
    case "claude":
      return "text-orange-500";
    case "gemini":
      return "text-blue-500";
    case "cursor":
      return "text-purple-500";
    default:
      return "text-muted-foreground";
  }
}

/**
 * 格式化会话名称
 * 优先使用会话 title，否则使用 ID 的最后部分作为简短名称
 */
function formatSessionName(session: SessionSummary): string {
  // 优先使用 title
  if (session.title) {
    return session.title;
  }
  // 如果 ID 包含下划线或连字符，取最后一部分
  const parts = session.id.split(/[-_]/);
  if (parts.length > 1) {
    return parts[parts.length - 1].slice(0, 8);
  }
  // 否则取前 8 个字符
  return session.id.slice(0, 8);
}

/**
 * SessionTreeItem 组件
 * 会话节点，点击导航到 Player 页面
 */
export function SessionTreeItem({
  session,
  isCurrent,
  searchKeyword,
  onClick,
}: SessionTreeItemProps) {
  // 格式化相对时间
  const relativeTime = React.useMemo(() => {
    try {
      return formatDistanceToNow(new Date(session.updated_at), {
        addSuffix: true,
        locale: zhCN,
      });
    } catch {
      return "";
    }
  }, [session.updated_at]);

  // 会话名称（优先使用 title，否则使用 ID 的简短形式）
  const sessionName = formatSessionName(session);

  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "w-full flex items-center gap-2 pl-10 pr-4 py-1.5",
        "hover:bg-muted/50 transition-colors cursor-pointer",
        "text-left text-sm",
        isCurrent && "bg-muted"
      )}
      data-testid={`session-tree-item-${session.id}`}
    >
      {/* 当前选中标记 (AC8) */}
      {isCurrent ? (
        <span className="w-2 h-2 rounded-full bg-primary shrink-0" />
      ) : (
        <span className="w-2 shrink-0" />
      )}

      {/* 会话图标 */}
      <MessageCircle
        className={cn("h-3.5 w-3.5 shrink-0", getSourceColor(session.source))}
      />

      {/* 会话名称 */}
      <span className="flex-1 truncate">
        <HighlightText text={sessionName} keyword={searchKeyword} />
      </span>

      {/* 消息数量 */}
      <span className="text-xs text-muted-foreground shrink-0">
        {session.message_count}
      </span>

      {/* 相对时间 */}
      <span className="text-xs text-muted-foreground shrink-0 w-16 text-right truncate">
        {relativeTime}
      </span>
    </button>
  );
}
