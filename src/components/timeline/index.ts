/**
 * Timeline 组件导出
 * Story 2.6: TimberLine 时间轴控制器
 */

export { TimberLine } from "./TimberLine";
export { TickMark } from "./TickMark";
export { TimeTooltip } from "./TimeTooltip";

// Re-export types
export type {
    TimelineEvent,
    TimelineEventType,
    TimberLineProps,
    TickMarkProps,
    TimeTooltipProps,
} from "@/types/timeline";

export {
    timeToPosition,
    positionToTime,
    findNearestEvent,
} from "@/types/timeline";
