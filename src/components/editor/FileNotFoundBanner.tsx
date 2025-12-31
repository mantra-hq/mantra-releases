/**
 * FileNotFoundBanner - 文件不存在提示组件
 * Story 2.12: Task 3 - AC #5
 *
 * 当选中的文件在历史 commit 中不存在时，显示友好的提示信息
 * 并提供保持当前视图或关闭提示的选项
 */

import * as React from "react";
import { FileQuestion, History, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { formatDistanceToNow } from "date-fns";
import { zhCN } from "date-fns/locale";
import { cn } from "@/lib/utils";

export interface FileNotFoundBannerProps {
  /** 文件路径 */
  filePath: string;
  /** 时间戳 (Unix 毫秒，可选) */
  timestamp?: number;
  /** 关闭回调 */
  onDismiss?: () => void;
  /** 保持当前视图回调 */
  onKeepCurrent?: () => void;
  /** 自定义 className */
  className?: string;
}

/**
 * 文件不存在提示 Banner
 *
 * AC #5: 选中的文件在该 commit 不存在时，显示友好提示：
 * "文件 {path} 在该时间点不存在"，并保持上一个有效状态
 */
export function FileNotFoundBanner({
  filePath,
  timestamp,
  onDismiss,
  onKeepCurrent,
  className,
}: FileNotFoundBannerProps) {
  // 格式化时间为相对时间
  const timeAgo = React.useMemo(() => {
    if (!timestamp) return null;
    try {
      return formatDistanceToNow(new Date(timestamp), {
        addSuffix: true,
        locale: zhCN,
      });
    } catch {
      return null;
    }
  }, [timestamp]);

  return (
    <div
      role="alert"
      className={cn(
        "relative z-10 bg-amber-500/10 border-b border-amber-500/30 backdrop-blur-sm",
        className
      )}
    >
      <div className="flex items-center gap-3 px-4 py-3">
        {/* 图标 */}
        <div className="flex-shrink-0">
          <div className="w-10 h-10 rounded-full bg-amber-500/20 flex items-center justify-center">
            <FileQuestion className="w-5 h-5 text-amber-500" />
          </div>
        </div>

        {/* 文本内容 */}
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium text-amber-200">
            文件在该时间点不存在
          </p>
          <p className="text-xs text-amber-200/70 truncate mt-0.5">
            <code className="bg-amber-500/20 px-1 rounded">{filePath}</code>
            {timeAgo && (
              <>
                <span className="mx-2">•</span>
                <span className="inline-flex items-center gap-1">
                  <History className="w-3 h-3" />
                  {timeAgo}
                </span>
              </>
            )}
          </p>
        </div>

        {/* 操作按钮 */}
        <div className="flex items-center gap-2">
          {onKeepCurrent && (
            <Button
              variant="ghost"
              size="sm"
              onClick={onKeepCurrent}
              className="text-amber-200 hover:text-amber-100 hover:bg-amber-500/20"
            >
              保持当前视图
            </Button>
          )}
          {onDismiss && (
            <Button
              variant="ghost"
              size="icon"
              onClick={onDismiss}
              className="text-amber-200/50 hover:text-amber-200 h-8 w-8"
              aria-label="关闭提示"
            >
              <X className="w-4 h-4" />
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}

export default FileNotFoundBanner;
