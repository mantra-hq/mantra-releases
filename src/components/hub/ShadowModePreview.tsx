/**
 * 影子模式变更预览组件
 * Story 11.13: Task 4 - 影子模式变更预览 (AC: #4)
 *
 * 显示一个可展开的 "查看变更预览" 区域，展示每个将被修改的配置文件。
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { Badge } from "@/components/ui/badge";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
  Loader2,
  ChevronRight,
  FileCode,
  ArrowRight,
  AlertCircle,
} from "lucide-react";
import { cn } from "@/lib/utils";

// ===== 类型定义 =====

interface ShadowModeChange {
  /** 配置文件路径 */
  file_path: string;
  /** 修改前内容摘要 */
  before_summary: string;
  /** 修改后内容（Mantra Gateway 入口配置） */
  after_content: string;
  /** 将被备份的路径 */
  backup_path: string;
}

interface ShadowModePreviewResult {
  changes: ShadowModeChange[];
  gateway_url: string;
}

interface DetectedConfig {
  adapter_id: string;
  path: string;
  scope?: string;
  services: Array<{
    name: string;
    command: string;
    args: string[] | null;
    env: Record<string, string> | null;
    source_file: string;
    adapter_id: string;
  }>;
  parse_errors: string[];
}

interface ShadowModePreviewProps {
  /** 是否启用影子模式 */
  enabled: boolean;
  /** 扫描到的配置列表 */
  configs: DetectedConfig[];
}

export function ShadowModePreview({ enabled, configs }: ShadowModePreviewProps) {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [previewData, setPreviewData] = useState<ShadowModePreviewResult | null>(null);

  const loadPreview = useCallback(async () => {
    if (!enabled || previewData) return;

    setIsLoading(true);
    setError(null);

    try {
      const result = await invoke<ShadowModePreviewResult>("preview_shadow_mode_changes", {
        configs,
      });
      setPreviewData(result);
    } catch (err) {
      console.error("[ShadowModePreview] Failed to load preview:", err);
      setError((err as Error).message);
    } finally {
      setIsLoading(false);
    }
  }, [enabled, configs, previewData]);

  // 展开时加载预览
  useEffect(() => {
    if (isOpen && enabled && !previewData && !isLoading) {
      loadPreview();
    }
  }, [isOpen, enabled, previewData, isLoading, loadPreview]);

  // 禁用时清除数据
  useEffect(() => {
    if (!enabled) {
      setPreviewData(null);
      setIsOpen(false);
    }
  }, [enabled]);

  if (!enabled) {
    return null;
  }

  return (
    <Collapsible open={isOpen} onOpenChange={setIsOpen} className="mt-3">
      <CollapsibleTrigger
        className="flex items-center gap-2 text-sm text-muted-foreground hover:text-foreground transition-colors"
        data-testid="shadow-mode-preview-trigger"
      >
        <ChevronRight
          className={cn(
            "h-4 w-4 transition-transform duration-200",
            isOpen && "rotate-90"
          )}
        />
        <FileCode className="h-4 w-4" />
        {t("hub.import.shadowPreviewTrigger")}
      </CollapsibleTrigger>

      <CollapsibleContent className="mt-3">
        {isLoading ? (
          <div className="flex items-center justify-center py-4">
            <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
            <span className="ml-2 text-sm text-muted-foreground">
              {t("hub.import.shadowPreviewLoading")}
            </span>
          </div>
        ) : error ? (
          <Alert variant="destructive" className="text-sm">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : previewData && previewData.changes.length > 0 ? (
          <div className="space-y-3 border rounded-lg p-3 bg-muted/30">
            {previewData.changes.map((change, idx) => (
              <div key={idx} className="space-y-2" data-testid={`shadow-change-${idx}`}>
                <div className="flex items-center gap-2 text-xs">
                  <Badge variant="outline" className="font-mono">
                    {change.file_path}
                  </Badge>
                </div>

                <div className="grid grid-cols-[1fr_auto_1fr] gap-2 items-start text-xs">
                  {/* 修改前 */}
                  <div className="p-2 rounded bg-red-500/5 border border-red-500/20">
                    <div className="font-medium text-red-500 mb-1">
                      {t("hub.import.shadowBefore")}
                    </div>
                    <div className="text-muted-foreground">
                      {change.before_summary}
                    </div>
                  </div>

                  <ArrowRight className="h-4 w-4 text-muted-foreground mt-2" />

                  {/* 修改后 */}
                  <div className="p-2 rounded bg-green-500/5 border border-green-500/20">
                    <div className="font-medium text-green-500 mb-1">
                      {t("hub.import.shadowAfter")}
                    </div>
                    <code className="text-[10px] whitespace-pre-wrap break-all text-muted-foreground">
                      {change.after_content.length > 200
                        ? change.after_content.slice(0, 200) + "..."
                        : change.after_content}
                    </code>
                  </div>
                </div>

                <div className="text-[10px] text-muted-foreground">
                  {t("hub.import.shadowBackupTo")}: <code>{change.backup_path}</code>
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div className="text-sm text-muted-foreground py-2">
            {t("hub.import.shadowNoChanges")}
          </div>
        )}
      </CollapsibleContent>
    </Collapsible>
  );
}

export default ShadowModePreview;
