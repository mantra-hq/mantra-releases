/**
 * SessionCard Component - 会话卡片组件
 * Story 2.8: Task 3
 *
 * 展示会话信息，点击进入 Player 视图
 */

import * as React from "react";
import { MessageSquare, Sparkles, Terminal, HelpCircle } from "lucide-react";
import { formatDistanceToNow } from "date-fns";
import { zhCN } from "date-fns/locale";
import { cn } from "@/lib/utils";
import type { Session, SessionSource } from "@/types/project";

/**
 * SessionCard Props
 */
export interface SessionCardProps {
  /** 会话数据 */
  session: Session;
  /** 点击回调 */
  onClick: () => void;
}

/**
 * 来源图标映射
 */
const sourceIcons: Record<SessionSource, React.ReactNode> = {
  claude: <Sparkles className="w-5 h-5 text-orange-500" />,
  gemini: <MessageSquare className="w-5 h-5 text-blue-500" />,
  cursor: <Terminal className="w-5 h-5 text-purple-500" />,
  unknown: <HelpCircle className="w-5 h-5 text-gray-500" />,
};

/**
 * 来源名称映射
 */
const sourceNames: Record<SessionSource, string> = {
  claude: "Claude",
  gemini: "Gemini",
  cursor: "Cursor",
  unknown: "Unknown",
};

/**
 * 生成会话显示标题
 * 使用来源名称 + 创建时间作为标题
 */
function getSessionTitle(session: Session): string {
  const date = new Date(session.created_at);
  const timeStr = date.toLocaleString("zh-CN", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
  return `${sourceNames[session.source]} 会话 · ${timeStr}`;
}

/**
 * SessionCard 组件
 * 显示会话标题、来源、消息数和时间
 */
export function SessionCard({ session, onClick }: SessionCardProps) {
  // 格式化相对时间
  const relativeTime = React.useMemo(() => {
    return formatDistanceToNow(new Date(session.created_at), {
      addSuffix: true,
      locale: zhCN,
    });
  }, [session.created_at]);

  // 生成标题
  const title = React.useMemo(() => getSessionTitle(session), [session]);

  return (
    <button
      type="button"
      onClick={onClick}
      data-testid="session-card"
      className={cn(
        "w-full flex items-center gap-3 p-3 rounded-md",
        "cursor-pointer text-left",
        "transition-colors duration-150",
        "hover:bg-muted",
        "focus:outline-none focus-visible:ring-2 focus-visible:ring-primary focus-visible:ring-offset-2"
      )}
    >
      {/* 来源图标 */}
      <div className="flex-shrink-0" title={sourceNames[session.source]}>
        {sourceIcons[session.source]}
      </div>

      {/* 会话信息 */}
      <div className="flex-1 min-w-0">
        <span className="block text-sm font-medium text-foreground truncate">
          {title}
        </span>
        <div className="flex items-center gap-2 text-xs text-muted-foreground">
          <span>{session.message_count} 条消息</span>
          <span>·</span>
          <span>{relativeTime}</span>
        </div>
      </div>
    </button>
  );
}
