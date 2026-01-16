/**
 * TimeRangeSelector Component - 时间范围选择器
 * Story 2.34: Task 7.6
 *
 * 选择统计数据的时间范围（7天/30天/全部）
 */

import { useTranslation } from "react-i18next";
import { Calendar } from "lucide-react";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type { TimeRange } from "@/types/analytics";

/**
 * TimeRangeSelector Props
 */
export interface TimeRangeSelectorProps {
  /** 当前选中的时间范围 */
  value: TimeRange;
  /** 值变更回调 */
  onChange: (value: TimeRange) => void;
  /** 自定义 className */
  className?: string;
}

/**
 * TimeRangeSelector 组件
 *
 * 下拉选择器用于切换统计时间范围
 */
export function TimeRangeSelector({
  value,
  onChange,
  className,
}: TimeRangeSelectorProps) {
  const { t } = useTranslation();

  return (
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger
        className={className}
        data-testid="time-range-selector"
      >
        <Calendar className="h-4 w-4 mr-2 text-muted-foreground" />
        <SelectValue placeholder={t("analytics.timeRange.label")} />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value="days7" data-testid="time-range-days7">
          {t("analytics.timeRange.days7")}
        </SelectItem>
        <SelectItem value="days30" data-testid="time-range-days30">
          {t("analytics.timeRange.days30")}
        </SelectItem>
        <SelectItem value="all" data-testid="time-range-all">
          {t("analytics.timeRange.all")}
        </SelectItem>
      </SelectContent>
    </Select>
  );
}
