/**
 * CodePanel - 代码快照面板
 * Story 2.2: Task 4.2 (初始占位)
 * Story 2.5: Task 5.2 (集成 CodeSnapshotView)
 *
 * 右侧面板，显示代码变更和文件快照
 */

import { cn } from "@/lib/utils";
import { CodeSnapshotView } from "@/components/editor";

export interface CodePanelProps {
  /** 自定义 className */
  className?: string;
  /** 代码内容 */
  code?: string;
  /** 文件路径 */
  filePath?: string;
  /** 历史时间戳 (ISO 8601) */
  timestamp?: string;
  /** Commit Hash (短格式) */
  commitHash?: string;
}

/**
 * 代码面板组件
 *
 * 功能:
 * - 集成 CodeSnapshotView 显示代码快照
 * - 传递历史状态信息 (时间戳、Commit)
 * - 空状态时显示友好提示
 */
export function CodePanel({
  className,
  code = "",
  filePath = "",
  timestamp,
  commitHash,
}: CodePanelProps) {
  return (
    <div className={cn("h-full", className)}>
      <CodeSnapshotView
        code={code}
        filePath={filePath}
        timestamp={timestamp}
        commitHash={commitHash}
      />
    </div>
  );
}

export default CodePanel;

