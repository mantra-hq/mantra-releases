/**
 * 工具配置路径管理组件
 * Story 13.1: 工具配置路径可配置化 - Task 6
 *
 * 允许用户通过文件浏览器选择各 AI 编程工具的配置目录
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { FolderCog, RotateCcw, FolderOpen, Loader2 } from "lucide-react";
import {
  getToolConfigPaths,
  setToolConfigPath,
  resetToolConfigPath,
  type ToolConfigPathInfo,
} from "@/lib/tool-config-path-ipc";
import { feedback } from "@/lib/feedback";

export function ToolConfigPathManager() {
  const { t } = useTranslation();
  const [tools, setTools] = useState<ToolConfigPathInfo[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [savingTool, setSavingTool] = useState<string | null>(null);

  const loadPaths = useCallback(async () => {
    try {
      const result = await getToolConfigPaths();
      setTools(result);
    } catch (err) {
      console.error("[ToolConfigPathManager] Failed to load:", err);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadPaths();
  }, [loadPaths]);

  const handleBrowse = async (tool: ToolConfigPathInfo) => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: t("settings.toolConfigPaths.selectDir"),
        defaultPath: tool.overrideDir ?? tool.defaultDir,
      });

      if (!selected) return;

      setSavingTool(tool.toolType);
      // If the selected dir is same as default, reset instead
      if (selected === tool.defaultDir) {
        await resetToolConfigPath(tool.toolType);
      } else {
        await setToolConfigPath(tool.toolType, selected);
      }
      await loadPaths();
      feedback.success(t("settings.toolConfigPaths.saveSuccess"));
    } catch (err) {
      feedback.error(
        t("common.save"),
        (err as Error).message
      );
    } finally {
      setSavingTool(null);
    }
  };

  const handleReset = async (toolType: string) => {
    setSavingTool(toolType);
    try {
      await resetToolConfigPath(toolType);
      await loadPaths();
      feedback.success(t("settings.toolConfigPaths.resetSuccess"));
    } catch (err) {
      feedback.error(
        t("settings.toolConfigPaths.reset"),
        (err as Error).message
      );
    } finally {
      setSavingTool(null);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center gap-2 text-muted-foreground py-4">
        <Loader2 className="h-4 w-4 animate-spin" />
        <span className="text-sm">{t("common.loading")}</span>
      </div>
    );
  }

  return (
    <div>
      <div className="flex items-center gap-2 mb-3">
        <FolderCog className="h-5 w-5 text-blue-500" />
        <h2 className="text-lg font-semibold">
          {t("settings.toolConfigPaths.title")}
        </h2>
      </div>
      <p className="text-sm text-muted-foreground mb-4">
        {t("settings.toolConfigPaths.description")}
      </p>

      <div className="space-y-3">
        {tools.map((tool) => {
          const activeDir = tool.overrideDir ?? tool.defaultDir;
          const isSaving = savingTool === tool.toolType;

          return (
            <div
              key={tool.toolType}
              className="rounded-md border bg-background p-3"
            >
              <div className="flex items-center justify-between mb-1">
                <span className="text-sm font-medium">{tool.displayName}</span>
                <div className="flex items-center gap-1">
                  {tool.overrideDir && (
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={() => handleReset(tool.toolType)}
                      disabled={isSaving}
                      className="h-7 px-2 text-xs"
                    >
                      <RotateCcw className="h-3 w-3 mr-1" />
                      {t("settings.toolConfigPaths.reset")}
                    </Button>
                  )}
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => handleBrowse(tool)}
                    disabled={isSaving}
                    className="h-7 px-2 text-xs"
                  >
                    {isSaving ? (
                      <Loader2 className="h-3 w-3 mr-1 animate-spin" />
                    ) : (
                      <FolderOpen className="h-3 w-3 mr-1" />
                    )}
                    {t("settings.toolConfigPaths.browse")}
                  </Button>
                </div>
              </div>

              {/* 当前生效目录 */}
              <p
                className={`text-xs font-mono truncate ${
                  tool.overrideDir
                    ? "text-blue-400"
                    : "text-muted-foreground"
                }`}
                title={activeDir}
              >
                {activeDir}
              </p>
            </div>
          );
        })}
      </div>
    </div>
  );
}
