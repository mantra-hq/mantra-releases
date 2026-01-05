/**
 * NoGitWarning - Git 仓库未关联警告组件
 * Story 2.11: Task 7 (AC6, AC7)
 * Story 2.26: 国际化支持
 *
 * 当项目未检测到 Git 仓库时显示友好的警告提示
 * 提供操作引导指导用户如何关联 Git 仓库
 */

import { useTranslation } from "react-i18next";
import { GitBranch, AlertTriangle, ExternalLink } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";

export interface NoGitWarningProps {
  /** 自定义 className */
  className?: string;
  /** 项目路径 (可选，用于显示) */
  projectPath?: string;
  /** 了解更多回调 (AC7) */
  onLearnMore?: () => void;
}

/**
 * Git 未关联警告组件
 *
 * 功能:
 * - 显示警告状态 UI (图标 + 标题 + 说明)
 * - 提供帮助信息
 * - 提供"了解更多"链接 (AC7)
 */
export function NoGitWarning({ className, projectPath, onLearnMore }: NoGitWarningProps) {
  const { t, i18n } = useTranslation();

  const handleLearnMore = () => {
    if (onLearnMore) {
      onLearnMore();
    } else {
      // 默认打开 Git 文档
      const gitDocUrl = i18n.language === "zh-CN"
        ? "https://git-scm.com/book/zh/v2"
        : "https://git-scm.com/book/en/v2";
      window.open(gitDocUrl, "_blank");
    }
  };

  return (
    <div
      className={cn(
        "flex h-full flex-col items-center justify-center",
        "p-8 text-center",
        className
      )}
    >
      {/* 图标 */}
      <div className="mb-4 rounded-full bg-amber-500/10 p-4">
        <GitBranch className="size-12 text-amber-500/70" />
      </div>

      {/* 标题 */}
      <h3 className="mb-2 text-base font-semibold text-foreground">
        {t("editor.noGitRepo")}
      </h3>

      {/* 说明文字 (AC6) */}
      <p className="mb-2 max-w-[320px] text-sm text-muted-foreground">
        {t("editor.noGitRepoDesc")}
      </p>

      {/* 对话时间旅行提示 */}
      <p className="mb-4 max-w-[320px] text-xs text-emerald-500/80">
        {t("editor.timeTravelStillWorks")}
      </p>

      {/* 项目路径提示 */}
      {projectPath && (
        <div className="mb-4 max-w-[400px] overflow-hidden">
          <code className="text-xs text-muted-foreground/80 font-mono truncate block">
            {projectPath}
          </code>
        </div>
      )}

      {/* 帮助信息 */}
      <div
        className={cn(
          "flex items-center gap-2 mb-4",
          "rounded-md bg-amber-500/10 px-3 py-2",
          "text-xs text-amber-600 dark:text-amber-400"
        )}
      >
        <AlertTriangle className="size-4" />
        <span>{t("editor.ensureGitFolder")}</span>
      </div>

      {/* 了解更多链接 (AC7) */}
      <Button
        variant="outline"
        size="sm"
        onClick={handleLearnMore}
        className="gap-2"
      >
        {t("common.learnMore")}
        <ExternalLink className="size-4" />
      </Button>
    </div>
  );
}

export default NoGitWarning;
