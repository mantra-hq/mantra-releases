/**
 * Resizable Components
 * Based on react-resizable-panels v4.x
 *
 * Note: In v4.x, components are named:
 * - Group (was PanelGroup)
 * - Panel (same)
 * - Separator (was PanelResizeHandle)
 */

import { GripVertical, GripHorizontal } from "lucide-react";
import {
  Group,
  Panel,
  Separator,
  type GroupProps,
  type PanelProps,
  type SeparatorProps,
} from "react-resizable-panels";

import { cn } from "@/lib/utils";

function ResizablePanelGroup({ className, ...props }: GroupProps) {
  return (
    <Group
      data-slot="resizable-panel-group"
      className={cn(
        "flex h-full w-full data-[panel-group-direction=vertical]:flex-col",
        className
      )}
      {...props}
    />
  );
}

function ResizablePanel({ ...props }: PanelProps) {
  return <Panel data-slot="resizable-panel" {...props} />;
}

/**
 * ResizableHandle - 可拖动的分隔栏
 * @param withHandle - 是否显示拖动手柄图标
 * @param orientation - 分隔栏方向，'horizontal' 用于左右分隔（垂直线），'vertical' 用于上下分隔（水平线）
 */
function ResizableHandle({
  withHandle,
  orientation = "horizontal",
  className,
  style,
  ...props
}: SeparatorProps & {
  withHandle?: boolean;
  orientation?: "horizontal" | "vertical";
}) {
  const isVertical = orientation === "vertical";
  
  return (
    <Separator
      data-slot="resizable-handle"
      className={cn(
        // Base styles - ensure it can receive pointer events
        "relative flex items-center justify-center transition-colors",
        "touch-none select-none",
        // Horizontal orientation (left-right separator = vertical line)
        !isVertical && "w-3 bg-border/40 hover:bg-primary/20 data-[resize-handle-state=drag]:bg-primary cursor-col-resize",
        // Vertical orientation (top-bottom separator = horizontal line)
        isVertical && "h-3 w-full bg-border/40 hover:bg-primary/20 data-[resize-handle-state=drag]:bg-primary cursor-row-resize",
        "focus-visible:ring-ring focus-visible:ring-1 focus-visible:ring-offset-1 focus-visible:outline-hidden",
        className
      )}
      style={{
        // Ensure pointer events are enabled and z-index is high enough
        pointerEvents: "auto",
        zIndex: 10,
        ...style,
      }}
      {...props}
    >
      {withHandle && (
        <div 
          className="bg-background flex items-center justify-center rounded-sm border shadow-sm pointer-events-none"
          style={{
            width: isVertical ? "2rem" : "1rem",
            height: isVertical ? "1rem" : "2rem",
            zIndex: 11,
          }}
        >
          {isVertical ? (
            <GripHorizontal className="size-3 text-muted-foreground" />
          ) : (
            <GripVertical className="size-3 text-muted-foreground" />
          )}
        </div>
      )}
    </Separator>
  );
}

export { ResizablePanelGroup, ResizablePanel, ResizableHandle };
