/**
 * TakeoverStatusCard - MCP 接管状态卡片
 * Story 11.15: Task 7.3-7.5 - 接管状态展示与一键恢复 (AC: #4, #5)
 *
 * 显示活跃的 MCP 配置接管记录：
 * - 接管时间
 * - 原始文件路径
 * - 备份文件路径
 * - 一键恢复按钮
 */

import { useState, useEffect, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { zhCN, enUS } from "date-fns/locale";
import { format, isValid, parseISO } from "date-fns";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import {
  Archive,
  RotateCcw,
  FileCode,
  Clock,
  Loader2,
  RefreshCw,
  CheckCircle2,
} from "lucide-react";
import { feedback } from "@/lib/feedback";
import { SourceIcon } from "@/components/import/SourceIcons";

/**
 * 工具类型
 */
type ToolType = "claude_code" | "cursor" | "codex" | "gemini_cli";

/**
 * 接管备份记录
 */
interface TakeoverBackup {
  id: string;
  tool_type: ToolType;
  original_path: string;
  backup_path: string;
  taken_over_at: string;
  restored_at: string | null;
  status: "active" | "restored";
}

/**
 * 获取工具类型显示名称
 */
function getToolLabel(toolType: ToolType): string {
  switch (toolType) {
    case "claude_code":
      return "Claude Code";
    case "cursor":
      return "Cursor";
    case "codex":
      return "Codex";
    case "gemini_cli":
      return "Gemini CLI";
    default:
      return toolType;
  }
}

/**
 * 将工具类型转换为 adapter_id
 */
function toolTypeToAdapterId(toolType: ToolType): string {
  switch (toolType) {
    case "claude_code":
      return "claude";
    case "gemini_cli":
      return "gemini";
    default:
      return toolType;
  }
}

/**
 * 格式化日期时间（使用 i18n locale）
 */
function formatDateTime(isoString: string | undefined | null, locale: string): string {
  // 空值检查
  if (!isoString) {
    return "-";
  }

  // 使用 parseISO 解析 ISO 字符串，比 new Date() 更可靠
  const date = parseISO(isoString);

  // 验证日期是否有效
  if (!isValid(date)) {
    console.warn("[TakeoverStatusCard] Invalid date string:", isoString);
    return isoString; // 回退到原始字符串
  }

  const dateLocale = locale === "zh-CN" ? zhCN : enUS;
  return format(date, "PPp", { locale: dateLocale });
}

export interface TakeoverStatusCardProps {
  onRestore?: () => void;
}

export function TakeoverStatusCard({ onRestore }: TakeoverStatusCardProps) {
  const { t, i18n } = useTranslation();
  const [backups, setBackups] = useState<TakeoverBackup[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [restoringId, setRestoringId] = useState<string | null>(null);

  // 缓存当前语言，避免重复渲染
  const currentLocale = useMemo(() => i18n.language, [i18n.language]);

  // 加载活跃的接管记录
  const loadBackups = useCallback(async () => {
    try {
      const result = await invoke<TakeoverBackup[]>("list_active_takeovers");
      setBackups(result);
    } catch (error) {
      console.error("[TakeoverStatusCard] Failed to load backups:", error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 恢复配置
  const handleRestore = useCallback(async (backupId: string) => {
    setRestoringId(backupId);
    try {
      await invoke("restore_takeover", { backupId });
      // 尝试重启 Gateway 以注销已恢复的服务配置
      try {
        const status = await invoke<{ running: boolean }>("get_gateway_status");
        if (status.running) {
          await invoke("restart_gateway", {});
        }
      } catch {
        // Gateway 操作失败不阻断恢复流程
      }
      feedback.success(t("hub.takeover.restoreSuccess"));
      // 重新加载列表
      await loadBackups();
      onRestore?.();
    } catch (error) {
      console.error("[TakeoverStatusCard] Failed to restore:", error);
      feedback.error(t("hub.takeover.restoreError"), (error as Error).message);
    } finally {
      setRestoringId(null);
    }
  }, [t, loadBackups, onRestore]);

  // 初始加载
  useEffect(() => {
    loadBackups();
  }, [loadBackups]);

  // 没有活跃接管记录时不显示
  if (!isLoading && backups.length === 0) {
    return null;
  }

  return (
    <Card data-testid="takeover-status-card">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 rounded-md bg-blue-500/10">
              <Archive className="h-5 w-5 text-blue-500" />
            </div>
            <div>
              <CardTitle className="text-base">{t("hub.takeover.title")}</CardTitle>
              <CardDescription>{t("hub.takeover.description")}</CardDescription>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <Badge variant="secondary" className="bg-blue-500/10 text-blue-500 border-blue-500/20">
              {t("hub.takeover.activeCount", { count: backups.length })}
            </Badge>
            <Button
              variant="ghost"
              size="icon"
              onClick={loadBackups}
              disabled={isLoading}
              className="h-8 w-8"
              title={t("common.refresh")}
            >
              <RefreshCw className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-3">
        {isLoading ? (
          <div className="flex items-center justify-center py-4">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
          </div>
        ) : (
          backups.map((backup) => (
            <div
              key={backup.id}
              className="flex items-start gap-3 p-3 border rounded-lg bg-muted/30"
              data-testid={`takeover-item-${backup.id}`}
            >
              {/* 工具图标和名称 */}
              <div className="flex items-center gap-2 shrink-0">
                <SourceIcon source={toolTypeToAdapterId(backup.tool_type)} className="h-5 w-5" />
              </div>

              {/* 信息 */}
              <div className="flex-1 min-w-0 space-y-1.5">
                <div className="flex items-center gap-2">
                  <span className="font-medium text-sm">{getToolLabel(backup.tool_type)}</span>
                  <Badge variant="outline" className="text-xs">
                    {t("hub.takeover.active")}
                  </Badge>
                </div>

                {/* 文件路径 */}
                <div className="space-y-1 text-xs text-muted-foreground">
                  <div className="flex items-center gap-1.5">
                    <FileCode className="h-3.5 w-3.5 shrink-0" />
                    <code className="truncate">{backup.original_path}</code>
                  </div>
                  <div className="flex items-center gap-1.5">
                    <Archive className="h-3.5 w-3.5 shrink-0" />
                    <code className="truncate">{backup.backup_path}</code>
                  </div>
                  <div className="flex items-center gap-1.5">
                    <Clock className="h-3.5 w-3.5 shrink-0" />
                    <span>{formatDateTime(backup.taken_over_at, currentLocale)}</span>
                  </div>
                </div>
              </div>

              {/* 恢复按钮 */}
              <AlertDialog>
                <AlertDialogTrigger asChild>
                  <Button
                    variant="outline"
                    size="sm"
                    disabled={restoringId === backup.id}
                    data-testid={`restore-button-${backup.id}`}
                  >
                    {restoringId === backup.id ? (
                      <Loader2 className="h-4 w-4 animate-spin" />
                    ) : (
                      <RotateCcw className="h-4 w-4" />
                    )}
                  </Button>
                </AlertDialogTrigger>
                <AlertDialogContent>
                  <AlertDialogHeader>
                    <AlertDialogTitle>{t("hub.takeover.restoreConfirmTitle")}</AlertDialogTitle>
                    <AlertDialogDescription>
                      {t("hub.takeover.restoreConfirmDescription", {
                        tool: getToolLabel(backup.tool_type),
                      })}
                    </AlertDialogDescription>
                  </AlertDialogHeader>
                  <div className="py-4 space-y-2 text-sm">
                    <div className="flex items-center gap-2">
                      <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                      <span>{t("hub.takeover.restoreWillDo1")}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                      <span>{t("hub.takeover.restoreWillDo2")}</span>
                    </div>
                    <div className="flex items-center gap-2">
                      <CheckCircle2 className="h-4 w-4 text-emerald-500" />
                      <span>{t("hub.takeover.restoreWillDo3")}</span>
                    </div>
                  </div>
                  <AlertDialogFooter>
                    <AlertDialogCancel>{t("common.cancel")}</AlertDialogCancel>
                    <AlertDialogAction onClick={() => handleRestore(backup.id)}>
                      {t("hub.takeover.restoreConfirm")}
                    </AlertDialogAction>
                  </AlertDialogFooter>
                </AlertDialogContent>
              </AlertDialog>
            </div>
          ))
        )}
      </CardContent>
    </Card>
  );
}

export default TakeoverStatusCard;
