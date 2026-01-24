/**
 * ExportDropdown - 导出下拉菜单组件
 * Story 10.7: Task 2/3
 *
 * 提供压缩会话的导出功能
 * 支持 JSONL、Markdown 格式导出和剪贴板复制
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { Download, FileJson, FileText, Copy, ChevronDown } from "lucide-react";
import { toast } from "sonner";
import { save } from "@tauri-apps/plugin-dialog";
import { writeTextFile } from "@tauri-apps/plugin-fs";

import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { cn } from "@/lib/utils";
import type { PreviewMessage } from "@/hooks/useCompressState";
import type { TokenStats } from "@/components/compress/TokenStatistics";
import {
  exportToJsonl,
  exportToMarkdown,
  getExportContent,
  formatExportFilename,
} from "@/lib/compress-exporter";

/**
 * ExportDropdown 组件 Props
 */
export interface ExportDropdownProps {
  /** 预览消息列表 (已过滤删除的) */
  previewMessages: PreviewMessage[];
  /** Token 统计数据 */
  tokenStats: TokenStats;
  /** 会话名称 (用于文件名) */
  sessionName?: string;
  /** 自定义 className */
  className?: string;
}

/**
 * ExportDropdown - 导出下拉菜单
 *
 * AC1: 显示 [📤 导出 ▾] 下拉按钮
 * AC2: 展开菜单显示三个选项: JSONL / Markdown / 复制
 * AC3: 导出为 JSONL
 * AC4: 导出为 Markdown
 * AC6: 复制内容到剪贴板
 */
export function ExportDropdown({
  previewMessages,
  tokenStats,
  sessionName,
  className,
}: ExportDropdownProps) {
  const { t } = useTranslation();

  /**
   * 导出为 JSONL
   * AC3: 弹出系统保存对话框，生成符合 Claude Code 格式的 JSONL 文件
   */
  const handleExportJsonl = React.useCallback(async () => {
    try {
      const defaultFileName = formatExportFilename(sessionName, "jsonl");

      const filePath = await save({
        title: t("compress.export.saveDialogTitle"),
        defaultPath: defaultFileName,
        filters: [{ name: "JSONL", extensions: ["jsonl"] }],
      });

      // AC: 用户取消保存时静默返回，不显示 Toast
      if (!filePath) return;

      const content = exportToJsonl(previewMessages);
      await writeTextFile(filePath, content);

      toast.success(t("compress.export.exportSuccess"));
    } catch (error) {
      console.error("Export JSONL failed:", error);
      toast.error(t("compress.export.exportFailed"));
    }
  }, [previewMessages, sessionName, t]);

  /**
   * 导出为 Markdown
   * AC4: 弹出系统保存对话框，生成人类可读的 Markdown 格式
   */
  const handleExportMarkdown = React.useCallback(async () => {
    try {
      const defaultFileName = formatExportFilename(sessionName, "md");

      const filePath = await save({
        title: t("compress.export.saveDialogTitle"),
        defaultPath: defaultFileName,
        filters: [{ name: "Markdown", extensions: ["md"] }],
      });

      // AC: 用户取消保存时静默返回，不显示 Toast
      if (!filePath) return;

      const content = exportToMarkdown(previewMessages, tokenStats, sessionName);
      await writeTextFile(filePath, content);

      toast.success(t("compress.export.exportSuccess"));
    } catch (error) {
      console.error("Export Markdown failed:", error);
      toast.error(t("compress.export.exportFailed"));
    }
  }, [previewMessages, tokenStats, sessionName, t]);

  /**
   * 复制内容到剪贴板
   * AC6: 将压缩后内容复制到剪贴板 (Markdown 格式)，显示 Toast 提示
   */
  const handleCopy = React.useCallback(async () => {
    try {
      const content = getExportContent(previewMessages);
      await navigator.clipboard.writeText(content);

      // AC6: Toast 2秒后自动消失
      toast.success(t("compress.export.copySuccess"), {
        duration: 2000,
      });
    } catch (error) {
      console.error("Copy to clipboard failed:", error);
      toast.error(t("compress.export.exportFailed"));
    }
  }, [previewMessages, t]);

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          className={cn("gap-1.5", className)}
          data-testid="export-dropdown-trigger"
        >
          <Download className="size-4" />
          <span className="hidden sm:inline">{t("compress.export.button")}</span>
          <ChevronDown className="size-3 opacity-50" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-48">
        <DropdownMenuItem
          onClick={handleExportJsonl}
          data-testid="export-jsonl"
        >
          <FileJson className="size-4 mr-2" />
          {t("compress.export.jsonl")}
        </DropdownMenuItem>
        <DropdownMenuItem
          onClick={handleExportMarkdown}
          data-testid="export-markdown"
        >
          <FileText className="size-4 mr-2" />
          {t("compress.export.markdown")}
        </DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuItem
          onClick={handleCopy}
          data-testid="export-copy"
        >
          <Copy className="size-4 mr-2" />
          {t("compress.export.copy")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

export default ExportDropdown;
