/**
 * Timeline Types - 时间轴类型定义
 * Story 2.6: TimberLine 时间轴控制器
 */

/**
 * 时间轴事件类型
 */
export type TimelineEventType = 'user-message' | 'ai-response' | 'git-commit';

/**
 * 时间轴事件
 */
export interface TimelineEvent {
  /** 时间戳 (Unix ms) */
  timestamp: number;
  /** 事件类型 */
  type: TimelineEventType;
  /** 关联的消息索引 (可选) */
  messageIndex?: number;
  /** 关联的 Commit Hash (可选) */
  commitHash?: string;
  /** 事件标签 (可选, 用于 Tooltip) */
  label?: string;
}

/**
 * TimberLine 组件 Props
 */
export interface TimberLineProps {
  /** 会话开始时间 (Unix ms) */
  startTime: number;
  /** 会话结束时间 (Unix ms) */
  endTime: number;
  /** 当前播放位置 (Unix ms) */
  currentTime: number;
  /** 时间轴事件列表 */
  events: TimelineEvent[];
  /** 位置变化回调 */
  onSeek: (timestamp: number) => void;
  /** 悬停事件回调 (可选) */
  onHover?: (timestamp: number | null) => void;
  /** 自定义 className */
  className?: string;
  /** 是否禁用交互 */
  disabled?: boolean;
}

/**
 * TickMark 组件 Props
 */
export interface TickMarkProps {
  /** 事件数据 */
  event: TimelineEvent;
  /** 位置百分比 (0-100) */
  position: number;
  /** 是否当前活跃 */
  isActive?: boolean;
  /** 点击回调 */
  onClick?: (event: TimelineEvent) => void;
  /** 悬停回调 */
  onHover?: (event: TimelineEvent | null) => void;
}

/**
 * TimeTooltip 组件 Props
 */
export interface TimeTooltipProps {
  /** 时间戳 (Unix ms) */
  timestamp: number;
  /** 是否显示 */
  visible: boolean;
  /** 附加标签 */
  label?: string;
  /** 位置样式 */
  style?: React.CSSProperties;
}

// ============================================
// 工具函数
// ============================================

/**
 * 将时间戳转换为百分比位置
 */
export function timeToPosition(
  timestamp: number,
  startTime: number,
  endTime: number
): number {
  if (endTime <= startTime) return 0;
  const position = ((timestamp - startTime) / (endTime - startTime)) * 100;
  return Math.max(0, Math.min(100, position));
}

/**
 * 将百分比位置转换为时间戳
 */
export function positionToTime(
  position: number,
  startTime: number,
  endTime: number
): number {
  const clampedPosition = Math.max(0, Math.min(100, position));
  return startTime + (clampedPosition / 100) * (endTime - startTime);
}

/**
 * 从 NarrativeMessage 数组生成 TimelineEvent 数组
 */
export function messagesToTimelineEvents(
  messages: Array<{ id: string; role: 'user' | 'assistant'; timestamp: string }>
): TimelineEvent[] {
  return messages.map((msg, index) => ({
    timestamp: new Date(msg.timestamp).getTime(),
    type: msg.role === 'user' ? 'user-message' : 'ai-response' as TimelineEventType,
    messageIndex: index,
    // label 由组件根据 type 进行国际化翻译
  }));
}

/**
 * 计算时间轴的起止时间
 */
export function getTimelineRange(
  events: TimelineEvent[]
): { startTime: number; endTime: number } {
  if (events.length === 0) {
    const now = Date.now();
    return { startTime: now, endTime: now };
  }

  const timestamps = events.map((e) => e.timestamp);
  const startTime = Math.min(...timestamps);
  const endTime = Math.max(...timestamps);

  // 如果只有一个事件，添加一些范围
  if (startTime === endTime) {
    return { startTime: startTime - 1000, endTime: endTime + 1000 };
  }

  return { startTime, endTime };
}

/**
 * 查找给定时间最近的事件
 */
export function findNearestEvent(
  events: TimelineEvent[],
  timestamp: number,
  direction: 'prev' | 'next' | 'nearest'
): TimelineEvent | null {
  if (events.length === 0) return null;

  const sorted = [...events].sort((a, b) => a.timestamp - b.timestamp);

  if (direction === 'prev') {
    for (let i = sorted.length - 1; i >= 0; i--) {
      if (sorted[i].timestamp < timestamp) return sorted[i];
    }
    return null;
  }

  if (direction === 'next') {
    for (const event of sorted) {
      if (event.timestamp > timestamp) return event;
    }
    return null;
  }

  // direction === 'nearest'
  let nearest = sorted[0];
  let minDiff = Math.abs(sorted[0].timestamp - timestamp);

  for (const event of sorted) {
    const diff = Math.abs(event.timestamp - timestamp);
    if (diff < minDiff) {
      minDiff = diff;
      nearest = event;
    }
  }

  return nearest;
}
