/**
 * RPC Log Viewer 组件
 * Story 11.11: Task 4 - RpcLogViewer (AC: 3, 5)
 *
 * JSON-RPC 请求/响应日志面板：
 * - 语法高亮的日志展示
 * - 时间戳和耗时统计
 * - 清空、导出、复制功能
 */

import { useState, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  Trash2,
  Download,
  Copy,
  ChevronRight,
  ChevronDown,
  Check,
  X,
  MoreVertical,
  Clock,
  ArrowUpRight,
  ArrowDownLeft,
  Terminal,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { feedback } from "@/lib/feedback";

/**
 * RPC 日志条目
 */
export interface RpcLogEntry {
  id: string;
  timestamp: string;
  method: string;
  request: unknown;
  response: unknown;
  error: { code: number; message: string; data?: unknown } | null;
  duration: number;
}

/**
 * RpcLogViewer 属性
 */
export interface RpcLogViewerProps {
  logs: RpcLogEntry[];
  onClear: () => void;
}

/**
 * 格式化时间戳
 */
function formatTimestamp(timestamp: string): string {
  const date = new Date(timestamp);
  const timeStr = date.toLocaleTimeString("en-US", {
    hour12: false,
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
  });
  const ms = date.getMilliseconds().toString().padStart(3, "0");
  return `${timeStr}.${ms}`;
}

/**
 * 格式化耗时
 */
function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

/**
 * RPC Log Viewer 组件
 */
export function RpcLogViewer({ logs, onClear }: RpcLogViewerProps) {
  const { t } = useTranslation();
  const [expandedLogs, setExpandedLogs] = useState<Set<string>>(new Set());
  const [copiedId, setCopiedId] = useState<string | null>(null);

  // 切换日志展开状态
  const toggleExpand = useCallback((id: string) => {
    setExpandedLogs((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  // 复制单条日志
  const handleCopyLog = useCallback(
    async (log: RpcLogEntry) => {
      const content = {
        timestamp: log.timestamp,
        method: log.method,
        request: log.request,
        response: log.response,
        error: log.error,
        duration: log.duration,
      };
      try {
        await navigator.clipboard.writeText(JSON.stringify(content, null, 2));
        setCopiedId(log.id);
        setTimeout(() => setCopiedId(null), 2000);
        feedback.copied(t("hub.inspector.log"));
      } catch {
        feedback.error(t("common.copy"));
      }
    },
    [t]
  );

  // 导出所有日志
  const handleExport = useCallback(async () => {
    const content = JSON.stringify(logs, null, 2);
    const blob = new Blob([content], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `mcp-inspector-logs-${new Date().toISOString().slice(0, 19).replace(/[:-]/g, "")}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    feedback.success(t("hub.inspector.exportSuccess"));
  }, [logs, t]);

  // 展开所有
  const handleExpandAll = useCallback(() => {
    setExpandedLogs(new Set(logs.map((log) => log.id)));
  }, [logs]);

  // 折叠所有
  const handleCollapseAll = useCallback(() => {
    setExpandedLogs(new Set());
  }, []);

  // 统计信息
  const stats = useMemo(() => {
    const total = logs.length;
    const errors = logs.filter((log) => log.error).length;
    const success = total - errors;
    const avgDuration =
      total > 0 ? logs.reduce((acc, log) => acc + log.duration, 0) / total : 0;
    return { total, errors, success, avgDuration };
  }, [logs]);

  return (
    <div className="flex flex-col h-full" data-testid="rpc-log-viewer">
      {/* 工具栏 */}
      <div className="flex items-center justify-between p-2 border-b bg-muted/30">
        <div className="flex items-center gap-2">
          <Terminal className="h-4 w-4 text-muted-foreground" />
          <span className="text-sm font-medium">{t("hub.inspector.logConsole")}</span>
          <Badge variant="secondary" className="text-xs">
            {stats.total}
          </Badge>
          {stats.errors > 0 && (
            <Badge variant="destructive" className="text-xs">
              {stats.errors} {t("hub.inspector.errors")}
            </Badge>
          )}
        </div>
        <div className="flex items-center gap-1">
          {stats.total > 0 && (
            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <span className="text-xs text-muted-foreground mr-2">
                    Ø {formatDuration(stats.avgDuration)}
                  </span>
                </TooltipTrigger>
                <TooltipContent>
                  <p>{t("hub.inspector.avgDuration")}</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          )}
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon" className="h-7 w-7">
                <MoreVertical className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              <DropdownMenuItem onClick={handleExpandAll} disabled={logs.length === 0}>
                <ChevronDown className="h-4 w-4 mr-2" />
                {t("common.expand")}
              </DropdownMenuItem>
              <DropdownMenuItem onClick={handleCollapseAll} disabled={logs.length === 0}>
                <ChevronRight className="h-4 w-4 mr-2" />
                {t("common.collapse")}
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem onClick={handleExport} disabled={logs.length === 0}>
                <Download className="h-4 w-4 mr-2" />
                {t("hub.inspector.exportLogs")}
              </DropdownMenuItem>
              <DropdownMenuSeparator />
              <DropdownMenuItem
                onClick={onClear}
                disabled={logs.length === 0}
                className="text-destructive focus:text-destructive"
              >
                <Trash2 className="h-4 w-4 mr-2" />
                {t("common.clear")}
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </div>

      {/* 日志列表 */}
      <ScrollArea className="flex-1">
        {logs.length === 0 ? (
          <div className="flex items-center justify-center h-full text-muted-foreground">
            <p className="text-sm">{t("hub.inspector.noLogs")}</p>
          </div>
        ) : (
          <div className="p-2 space-y-1">
            {logs.map((log) => (
              <Collapsible
                key={log.id}
                open={expandedLogs.has(log.id)}
                onOpenChange={() => toggleExpand(log.id)}
              >
                <div
                  className={cn(
                    "rounded-md border text-sm",
                    log.error ? "border-destructive/30" : "border-border"
                  )}
                >
                  <CollapsibleTrigger className="flex items-center gap-2 w-full p-2 hover:bg-muted/50 rounded-t-md">
                    {expandedLogs.has(log.id) ? (
                      <ChevronDown className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                    ) : (
                      <ChevronRight className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                    )}

                    {/* 状态图标 */}
                    {log.error ? (
                      <X className="h-3.5 w-3.5 shrink-0 text-destructive" />
                    ) : log.response ? (
                      <Check className="h-3.5 w-3.5 shrink-0 text-emerald-500" />
                    ) : (
                      <Clock className="h-3.5 w-3.5 shrink-0 text-amber-500 animate-pulse" />
                    )}

                    {/* 方法名 */}
                    <code className="text-xs font-mono font-medium truncate">
                      {log.method}
                    </code>

                    {/* 时间戳 */}
                    <span className="text-xs text-muted-foreground ml-auto shrink-0">
                      {formatTimestamp(log.timestamp)}
                    </span>

                    {/* 耗时 */}
                    {log.duration > 0 && (
                      <Badge
                        variant="outline"
                        className={cn(
                          "text-[10px] px-1 py-0 shrink-0",
                          log.duration > 1000 && "text-amber-500 border-amber-500/30"
                        )}
                      >
                        {formatDuration(log.duration)}
                      </Badge>
                    )}

                    {/* 复制按钮 */}
                    <Button
                      variant="ghost"
                      size="icon"
                      className="h-6 w-6 shrink-0"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleCopyLog(log);
                      }}
                    >
                      {copiedId === log.id ? (
                        <Check className="h-3 w-3 text-emerald-500" />
                      ) : (
                        <Copy className="h-3 w-3" />
                      )}
                    </Button>
                  </CollapsibleTrigger>

                  <CollapsibleContent>
                    <div className="border-t p-2 space-y-2 bg-muted/20">
                      {/* 请求 */}
                      <div className="space-y-1">
                        <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                          <ArrowUpRight className="h-3 w-3" />
                          <span>{t("hub.inspector.request")}</span>
                        </div>
                        <pre className="text-xs font-mono bg-muted p-2 rounded overflow-auto max-h-[150px]">
                          {JSON.stringify(log.request, null, 2)}
                        </pre>
                      </div>

                      {/* 响应或错误 */}
                      {(log.response || log.error) && (
                        <div className="space-y-1">
                          <div className="flex items-center gap-1.5 text-xs text-muted-foreground">
                            <ArrowDownLeft className="h-3 w-3" />
                            <span>
                              {log.error
                                ? t("hub.inspector.error")
                                : t("hub.inspector.response")}
                            </span>
                          </div>
                          <pre
                            className={cn(
                              "text-xs font-mono p-2 rounded overflow-auto max-h-[200px]",
                              log.error ? "bg-destructive/10 text-destructive" : "bg-muted"
                            )}
                          >
                            {JSON.stringify(log.error || log.response, null, 2)}
                          </pre>
                        </div>
                      )}
                    </div>
                  </CollapsibleContent>
                </div>
              </Collapsible>
            ))}
          </div>
        )}
      </ScrollArea>
    </div>
  );
}

export default RpcLogViewer;
