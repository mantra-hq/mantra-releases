/**
 * CodePanel - 代码快照面板
 * Story 2.2: Task 4.2 (初始占位)
 * Story 2.5: Task 5.2 (集成 CodeSnapshotView)
 * Story 2.11: AC6, AC7 (无 Git 仓库警告)
 * Story 2.12: AC5 (文件不存在处理)
 *
 * 右侧面板，显示代码变更和文件快照
 */

import { cn } from "@/lib/utils";
import { CodeSnapshotView, NoGitWarning } from "@/components/editor";

export interface CodePanelProps {
  /** 自定义 className */
  className?: string;
  /** 代码内容 */
  code?: string;
  /** 文件路径 */
  filePath?: string;
  /** 历史时间戳 (ISO 8601 或 Unix ms) */
  timestamp?: string | number;
  /** Commit Hash (短格式) */
  commitHash?: string;
  /** Commit 消息 (Story 2.7 AC #6) */
  commitMessage?: string;
  /** 前一个代码内容 (用于 Diff 高亮, Story 2.7 AC #5) */
  previousCode?: string | null;
  /** 是否处于历史模式 (Story 2.7 AC #6) */
  isHistoricalMode?: boolean;
  /** 返回当前回调 (Story 2.7 AC #6) */
  onReturnToCurrent?: () => void;
  /** 无 Git 仓库警告 (Story 2.11 AC6) */
  showNoGitWarning?: boolean;
  /** 项目路径 (用于无 Git 警告显示) */
  projectPath?: string;
  /** 了解更多回调 (Story 2.11 AC7) */
  onLearnMore?: () => void;
  /** 文件未找到标志 (Story 2.12 AC #5) */
  fileNotFound?: boolean;
  /** 未找到的文件路径 (Story 2.12 AC #5) */
  notFoundPath?: string;
  /** 清除文件不存在状态回调 (Story 2.12 AC #5) */
  onDismissNotFound?: () => void;
}

/**
 * 代码面板组件
 *
 * 功能:
 * - 集成 CodeSnapshotView 显示代码快照
 * - 传递历史状态信息 (时间戳、Commit)
 * - 支持 Diff 高亮 (Story 2.7 AC #5)
 * - 支持历史模式 Banner (Story 2.7 AC #6)
 * - 无 Git 仓库时显示友好提示 (Story 2.11 AC6, AC7)
 * - 文件不存在时显示 FileNotFoundBanner (Story 2.12 AC #5)
 * - 空状态时显示友好提示
 */
export function CodePanel({
  className,
  code = "",
  filePath = "",
  timestamp,
  commitHash,
  commitMessage,
  previousCode,
  isHistoricalMode,
  onReturnToCurrent,
  showNoGitWarning = false,
  projectPath,
  onLearnMore,
  fileNotFound = false,
  notFoundPath,
  onDismissNotFound,
}: CodePanelProps) {
  // Story 2.11 AC6: 无 Git 仓库时显示警告
  if (showNoGitWarning && !code) {
    return (
      <div className={cn("h-full", className)}>
        <NoGitWarning projectPath={projectPath} onLearnMore={onLearnMore} />
      </div>
    );
  }

  return (
    <div className={cn("h-full", className)}>
      <CodeSnapshotView
        code={code}
        filePath={filePath}
        timestamp={timestamp}
        commitHash={commitHash}
        commitMessage={commitMessage}
        previousCode={previousCode}
        isHistoricalMode={isHistoricalMode}
        onReturnToCurrent={onReturnToCurrent}
        fileNotFound={fileNotFound}
        notFoundPath={notFoundPath}
        onDismissNotFound={onDismissNotFound}
      />
    </div>
  );
}

export default CodePanel;

