/**
 * DualStreamLayout - 双流分栏布局组件
 * Story 2.2: AC #1, #2, #3, #6
 * Story 2.6: 集成 TimberLine 时间轴控制器
 *
 * 左侧: NarrativePanel (对话流) - 默认 40%
 * 右侧: CodePanel (代码快照) - 默认 60%
 * 底部: TimberLine (时间轴控制器)
 *
 * Note: Uses react-resizable-panels v4.x API
 */

import * as React from "react";
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from "@/components/ui/resizable";
import { useLayoutPersist } from "@/hooks/use-layout-persist";
import { useResponsiveLayout, type LayoutMode } from "@/hooks/use-responsive-layout";
import { NarrativePanel, type NarrativePanelRef } from "./NarrativePanel";
import { CodePanel } from "./CodePanel";
import { cn } from "@/lib/utils";
import type { NarrativeMessage } from "@/types/message";
import type { TimelineEvent } from "@/types/timeline";
import { TimberLine } from "@/components/timeline";
import { useTimeTravelStore } from "@/stores/useTimeTravelStore";

// 响应式 Tab 组件 (Tablet/Mobile 模式)
import { MessageSquare, Code2 } from "lucide-react";

// Panel IDs for v4 API
const NARRATIVE_PANEL_ID = "narrative";
const CODE_PANEL_ID = "code";

export interface DualStreamLayoutProps {
  /** 左侧面板内容 (对话流) */
  narrativeContent?: React.ReactNode;
  /** 右侧面板内容 (代码快照) */
  codeContent?: React.ReactNode;
  /** 消息列表 (传递给 NarrativePanel) */
  messages?: NarrativeMessage[];
  /** 当前选中的消息 ID */
  selectedMessageId?: string;
  /** 消息选中回调 */
  onMessageSelect?: (messageId: string, message: NarrativeMessage) => void;
  /** 默认布局比例 [左, 右] */
  defaultLayout?: [number, number];
  /** 最小宽度限制 [左, 右] (百分比) */
  minSizes?: [number, number];
  /** 布局变化回调 */
  onLayoutChange?: (sizes: number[]) => void;
  /** 强制指定布局模式 (用于测试) */
  forceMode?: LayoutMode;
  /** 自定义 className */
  className?: string;

  // TimberLine 时间轴 Props (Story 2.6)
  /** 是否显示时间轴 */
  showTimeline?: boolean;
  /** 会话开始时间 (Unix ms) */
  timelineStartTime?: number;
  /** 会话结束时间 (Unix ms) */
  timelineEndTime?: number;
  /** 当前播放位置 (Unix ms) */
  timelineCurrentTime?: number;
  /** 时间轴事件列表 */
  timelineEvents?: TimelineEvent[];
  /** 时间轴位置变化回调 */
  onTimelineSeek?: (timestamp: number) => void;
  /** 时间轴悬停回调 */
  onTimelineHover?: (timestamp: number | null) => void;
}

/**
 * DualStreamLayout Ref 暴露的方法
 */
export interface DualStreamLayoutRef {
  /** 滚动到指定消息 */
  scrollToMessage: (messageId: string) => void;
  /** 滚动到顶部 */
  scrollToTop: () => void;
  /** 滚动到底部 */
  scrollToBottom: () => void;
}

const DEFAULT_LAYOUT: [number, number] = [40, 60];
const DEFAULT_MIN_SIZES: [number, number] = [20, 20];
const STORAGE_KEY = "mantra-dual-stream-layout";

type ActiveView = "narrative" | "code";

// 将 layout 数组转换为 v4 API 需要的 Layout 对象
function toLayoutObject(layout: [number, number]): { [id: string]: number } {
  return {
    [NARRATIVE_PANEL_ID]: layout[0],
    [CODE_PANEL_ID]: layout[1],
  };
}

// 将 v4 Layout 对象转换为 layout 数组
function fromLayoutObject(layout: { [id: string]: number }): [number, number] {
  return [
    layout[NARRATIVE_PANEL_ID] ?? DEFAULT_LAYOUT[0],
    layout[CODE_PANEL_ID] ?? DEFAULT_LAYOUT[1],
  ];
}

export const DualStreamLayout = React.forwardRef<
  DualStreamLayoutRef,
  DualStreamLayoutProps
>(
  (
    {
      narrativeContent,
      codeContent,
      messages,
      selectedMessageId,
      onMessageSelect,
      defaultLayout = DEFAULT_LAYOUT,
      minSizes = DEFAULT_MIN_SIZES,
      onLayoutChange,
      forceMode,
      className,
      // TimberLine props
      showTimeline = false,
      timelineStartTime,
      timelineEndTime,
      timelineCurrentTime,
      timelineEvents = [],
      onTimelineSeek,
      onTimelineHover,
    },
    ref
  ) => {
    // NarrativePanel ref
    const narrativePanelRef = React.useRef<NarrativePanelRef>(null);

    // 从 store 订阅代码状态
    const currentCode = useTimeTravelStore((state) => state.currentCode);
    const currentFilePath = useTimeTravelStore((state) => state.currentFilePath);
    // const previousCode = useTimeTravelStore((state) => state.previousCode);

    // 响应式布局检测
    const detectedMode = useResponsiveLayout();
    const layoutMode = forceMode ?? detectedMode;

    // 布局持久化 (仅 Desktop 模式)
    const { layout, setLayout } = useLayoutPersist({
      storageKey: STORAGE_KEY,
      defaultLayout,
    });

    // Tablet/Mobile 模式的活动视图状态
    const [activeView, setActiveView] = React.useState<ActiveView>("narrative");

    // 暴露给父组件的方法
    React.useImperativeHandle(
      ref,
      () => ({
        scrollToMessage: (messageId: string) => {
          narrativePanelRef.current?.scrollToMessage(messageId);
        },
        scrollToTop: () => {
          narrativePanelRef.current?.scrollToTop();
        },
        scrollToBottom: () => {
          narrativePanelRef.current?.scrollToBottom();
        },
      }),
      []
    );

    // 处理布局变化 (v4 API)
    const handleLayoutChange = React.useCallback(
      (newLayout: { [panelId: string]: number }) => {
        const layoutArray = fromLayoutObject(newLayout);
        setLayout(layoutArray);
        onLayoutChange?.(layoutArray);
      },
      [setLayout, onLayoutChange]
    );

    // 渲染内容
    const renderNarrativeContent = narrativeContent ?? (
      <NarrativePanel
        ref={narrativePanelRef}
        messages={messages}
        selectedMessageId={selectedMessageId}
        onMessageSelect={onMessageSelect}
      />
    );
    const renderCodeContent = codeContent ?? (
      <CodePanel
        code={currentCode ?? ""}
        filePath={currentFilePath ?? ""}
        // previousCode={previousCode ?? undefined} // 用于 diff 显示
      />
    );

    // 渲染 TimberLine 时间轴
    const renderTimeline = showTimeline &&
      timelineStartTime !== undefined &&
      timelineEndTime !== undefined &&
      timelineCurrentTime !== undefined &&
      onTimelineSeek && (
        <TimberLine
          startTime={timelineStartTime}
          endTime={timelineEndTime}
          currentTime={timelineCurrentTime}
          events={timelineEvents}
          onSeek={onTimelineSeek}
          onHover={onTimelineHover}
        />
      );

    // Desktop 模式: 双流分栏 + 底部时间轴
    if (layoutMode === "desktop") {
      return (
        <div className={cn("h-full w-full flex flex-col", className)}>
          <ResizablePanelGroup
            orientation="horizontal"
            defaultLayout={toLayoutObject(layout)}
            onLayoutChange={handleLayoutChange}
            className="flex-1"
          >
            {/* 左侧面板: 对话流 */}
            <ResizablePanel
              id={NARRATIVE_PANEL_ID}
              minSize={minSizes[0]}
              className="narrative-panel bg-background"
            >
              {renderNarrativeContent}
            </ResizablePanel>

            {/* 拖拽把手 */}
            <ResizableHandle
              withHandle
              className={cn(
                "w-1 transition-colors duration-150",
                "bg-transparent hover:bg-muted",
                "data-[resize-handle-state=drag]:bg-primary",
                "data-[resize-handle-active]:bg-primary/80"
              )}
            />

            {/* 右侧面板: 代码快照 */}
            <ResizablePanel
              id={CODE_PANEL_ID}
              minSize={minSizes[1]}
              className="code-panel bg-card"
            >
              {renderCodeContent}
            </ResizablePanel>
          </ResizablePanelGroup>

          {/* 底部时间轴 */}
          {renderTimeline}
        </div>
      );
    }

    // Tablet 模式: Tab 切换 + 底部时间轴
    if (layoutMode === "tablet") {
      return (
        <div className={cn("h-full w-full flex flex-col", className)}>
          {/* Tab 切换栏 */}
          <div className="flex border-b border-border bg-background">
            <TabButton
              active={activeView === "narrative"}
              onClick={() => setActiveView("narrative")}
              icon={<MessageSquare className="size-4" />}
              label="对话流"
            />
            <TabButton
              active={activeView === "code"}
              onClick={() => setActiveView("code")}
              icon={<Code2 className="size-4" />}
              label="代码快照"
            />
          </div>

          {/* 内容区域 */}
          <div className="flex-1 overflow-hidden">
            <div
              className={cn(
                "h-full transition-opacity duration-200",
                activeView === "narrative" ? "block" : "hidden"
              )}
            >
              {renderNarrativeContent}
            </div>
            <div
              className={cn(
                "h-full transition-opacity duration-200",
                activeView === "code" ? "block" : "hidden"
              )}
            >
              {renderCodeContent}
            </div>
          </div>

          {/* 底部时间轴 */}
          {renderTimeline}
        </div>
      );
    }

    // Mobile 模式: 单视图 + 底部 Tab Bar + 时间轴
    return (
      <div className={cn("h-full w-full flex flex-col", className)}>
        {/* 内容区域 */}
        <div className="flex-1 overflow-hidden">
          <div
            className={cn(
              "h-full transition-opacity duration-200",
              activeView === "narrative" ? "block" : "hidden"
            )}
          >
            {renderNarrativeContent}
          </div>
          <div
            className={cn(
              "h-full transition-opacity duration-200",
              activeView === "code" ? "block" : "hidden"
            )}
          >
            {renderCodeContent}
          </div>
        </div>

        {/* 时间轴 (在 Tab Bar 上方) */}
        {renderTimeline}

        {/* 底部 Tab Bar */}
        <div className="flex border-t border-border bg-background">
          <TabButton
            active={activeView === "narrative"}
            onClick={() => setActiveView("narrative")}
            icon={<MessageSquare className="size-5" />}
            label="对话"
            variant="bottom"
          />
          <TabButton
            active={activeView === "code"}
            onClick={() => setActiveView("code")}
            icon={<Code2 className="size-5" />}
            label="代码"
            variant="bottom"
          />
        </div>
      </div>
    );
  }
);

// Tab 按钮组件
interface TabButtonProps {
  active: boolean;
  onClick: () => void;
  icon: React.ReactNode;
  label: string;
  variant?: "top" | "bottom";
}

function TabButton({
  active,
  onClick,
  icon,
  label,
  variant = "top",
}: TabButtonProps) {
  const isBottom = variant === "bottom";

  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        "flex-1 flex items-center justify-center gap-2 transition-colors",
        "focus:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset",
        isBottom
          ? cn(
            "flex-col py-2",
            active
              ? "text-primary"
              : "text-muted-foreground hover:text-foreground"
          )
          : cn(
            "px-4 py-3",
            active
              ? "text-primary border-b-2 border-primary -mb-px"
              : "text-muted-foreground hover:text-foreground"
          )
      )}
      aria-pressed={active}
    >
      {icon}
      <span className={cn(isBottom ? "text-xs" : "text-sm font-medium")}>
        {label}
      </span>
    </button>
  );
}

DualStreamLayout.displayName = "DualStreamLayout";

export default DualStreamLayout;
