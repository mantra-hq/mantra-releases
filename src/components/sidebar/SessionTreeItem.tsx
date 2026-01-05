/**
 * SessionTreeItem Component - 会话树节点
 * Story 2.18: Task 4, Story 2.25: AC3
 * Story 2.26: 国际化支持
 *
 * 会话节点，显示会话信息和当前选中状态
 * 使用官方来源图标区分不同 AI 工具
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { formatDistanceToNow } from "date-fns";
import { zhCN, enUS } from "date-fns/locale";
import { cn, formatSessionName } from "@/lib/utils";
import { HighlightText } from "./DrawerSearch";
import { SourceIcon } from "@/components/import/SourceIcons";
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
 * SessionTreeItem 组件
 * 会话节点，点击导航到 Player 页面
 */
export function SessionTreeItem({
  session,
  isCurrent,
  searchKeyword,
  onClick,
}: SessionTreeItemProps) {
  const { i18n } = useTranslation();

  // 格式化相对时间
  const relativeTime = React.useMemo(() => {
    try {
      return formatDistanceToNow(new Date(session.updated_at), {
        addSuffix: true,
        locale: i18n.language === "zh-CN" ? zhCN : enUS,
      });
    } catch {
      return "";
    }
  }, [session.updated_at, i18n.language]);

  // 会话名称（优先使用 title，否则使用 ID 的简短形式）
  const sessionName = formatSessionName(session.id, session.title);

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

      {/* 来源图标 (Story 2.25: AC3) */}
      <SourceIcon
        source={session.source}
        className="h-3.5 w-3.5 shrink-0"
      />

      {/* 会话名称 */}
      <span className="flex-1 truncate" title={sessionName}>
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
