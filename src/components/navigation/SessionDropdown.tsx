/**
 * SessionDropdown Component - 会话下拉选择器
 * Story 2.17: Task 3
 *
 * 使用 Popover + Command 实现搜索和选择
 */

import * as React from "react";
import { Check, ChevronsUpDown, MessageSquare } from "lucide-react";
import { formatDistanceToNow } from "date-fns";
import { zhCN } from "date-fns/locale";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import type { SessionSummary } from "./TopBar";

/**
 * SessionDropdown Props
 */
export interface SessionDropdownProps {
  /** 当前会话 ID */
  currentSessionId: string;
  /** 当前会话名称 */
  currentSessionName: string;
  /** 当前会话消息数 */
  messageCount: number;
  /** 同项目会话列表 */
  sessions: SessionSummary[];
  /** 会话选择回调 */
  onSessionSelect: (sessionId: string) => void;
}

/**
 * SessionDropdown 组件
 * 会话下拉选择器，支持搜索过滤
 */
export function SessionDropdown({
  currentSessionId,
  currentSessionName,
  messageCount,
  sessions,
  onSessionSelect,
}: SessionDropdownProps) {
  const [open, setOpen] = React.useState(false);

  // 处理会话选择
  const handleSelect = React.useCallback(
    (sessionId: string) => {
      if (sessionId !== currentSessionId) {
        onSessionSelect(sessionId);
      }
      setOpen(false);
    },
    [currentSessionId, onSessionSelect]
  );

  // 格式化相对时间 (AC7)
  const formatRelativeTime = (timestamp: number) => {
    return formatDistanceToNow(new Date(timestamp), {
      addSuffix: true,
      locale: zhCN,
    });
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="ghost"
          size="sm"
          role="combobox"
          aria-expanded={open}
          aria-label="选择会话"
          data-testid="session-dropdown-trigger"
          className={cn(
            "h-8 px-2 gap-1.5 min-w-0",
            "text-muted-foreground hover:text-foreground",
            "transition-colors"
          )}
        >
          <MessageSquare className="h-4 w-4 shrink-0" />
          <span className="truncate max-w-[120px] md:max-w-[200px] lg:max-w-[300px]">
            {currentSessionName}
          </span>
          <span className="text-xs text-muted-foreground shrink-0">
            ({messageCount})
          </span>
          <ChevronsUpDown className="h-3 w-3 shrink-0 opacity-50" />
        </Button>
      </PopoverTrigger>
      <PopoverContent
        className="w-[320px] p-0"
        align="start"
        data-testid="session-dropdown-content"
      >
        <Command>
          {/* 搜索输入框 (AC8) */}
          <CommandInput
            placeholder="搜索会话..."
            data-testid="session-search-input"
          />
          <CommandList>
            <CommandEmpty>未找到匹配的会话</CommandEmpty>
            <CommandGroup>
              {sessions.map((session) => (
                <CommandItem
                  key={session.id}
                  value={session.name}
                  onSelect={() => handleSelect(session.id)}
                  data-testid={`session-item-${session.id}`}
                  className="cursor-pointer"
                >
                  {/* 当前会话标记 (AC6) */}
                  <Check
                    className={cn(
                      "h-4 w-4 shrink-0",
                      session.id === currentSessionId
                        ? "opacity-100"
                        : "opacity-0"
                    )}
                  />
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <span className="truncate font-medium">
                        {session.name}
                      </span>
                    </div>
                    {/* 会话元信息 (AC7) */}
                    <div className="flex items-center gap-2 text-xs text-muted-foreground">
                      <span>{session.messageCount} 条消息</span>
                      <span>·</span>
                      <span>{formatRelativeTime(session.lastActiveAt)}</span>
                    </div>
                  </div>
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
