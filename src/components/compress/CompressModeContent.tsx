/**
 * CompressModeContent - 压缩模式内容包装组件
 * Story 10.9: Task 3
 *
 * 在 CompressStateProvider 内部使用，处理：
 * - 状态持久化逻辑
 * - beforeunload 事件监听
 */

import * as React from "react";
import { DualStreamLayout, type DualStreamLayoutRef } from "@/components/layout";
import { OriginalMessageList, CompressPreviewList, TokenStatistics } from "@/components/compress";
import { useCompressPersistence } from "@/hooks/useCompressPersistence";
import { useCompressState } from "@/hooks/useCompressState";
import type { NarrativeMessage } from "@/types/message";

interface CompressModeContentProps {
  /** 布局 ref */
  layoutRef: React.RefObject<DualStreamLayoutRef | null>;
  /** 消息列表 */
  messages: NarrativeMessage[];
  /** 当前会话 ID */
  sessionId: string;
}

/**
 * 压缩模式内容组件
 * 必须在 CompressStateProvider 内部使用
 */
export function CompressModeContent({
  layoutRef,
  messages,
  sessionId,
}: CompressModeContentProps) {
  // Story 10.9: 压缩状态持久化
  useCompressPersistence({
    sessionId,
    isCompressMode: true, // 这个组件只在压缩模式下渲染
  });

  // Story 10.9 AC5: beforeunload 事件监听
  const { hasAnyChanges } = useCompressState();

  React.useEffect(() => {
    const handleBeforeUnload = (e: BeforeUnloadEvent) => {
      if (hasAnyChanges) {
        e.preventDefault();
        // 现代浏览器会显示通用提示，不再支持自定义消息
        e.returnValue = "";
      }
    };

    window.addEventListener("beforeunload", handleBeforeUnload);
    return () => window.removeEventListener("beforeunload", handleBeforeUnload);
  }, [hasAnyChanges]);

  return (
    <div className="flex-1 min-h-0 overflow-hidden flex flex-col">
      {/* Story 10.6 AC #1: 主内容区域添加 padding-bottom 避免被统计栏遮挡 */}
      <div className="flex-1 min-h-0 overflow-hidden">
        <DualStreamLayout
          ref={layoutRef}
          // Story 10.2: 精简模式左侧显示原始消息列表
          narrativeContent={
            <OriginalMessageList messages={messages} />
          }
          // Story 10.3: 右侧显示压缩预览列表
          codeContent={
            <CompressPreviewList messages={messages} />
          }
          // Story 10.1 AC #5: 精简模式隐藏时间轴
          showTimeline={false}
        />
      </div>
      {/* Story 10.6: Token 统计栏 (AC #1: 固定在底部) */}
      <TokenStatistics messages={messages} />
    </div>
  );
}

export default CompressModeContent;
