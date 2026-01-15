/**
 * TimeRangeFilter - 时间范围筛选器
 * Story 2.33: Task 3.4
 *
 * AC3: 选项：全部时间 | 今天 | 本周 | 本月
 * 使用 shadcn/ui Select 风格
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { Calendar, ChevronDown, Check } from "lucide-react";
import { cn } from "@/lib/utils";
import type { TimePreset } from "@/stores/useSearchStore";

export interface TimeRangeFilterProps {
    /** 当前选中的时间预设 */
    value: TimePreset;
    /** 时间预设变化回调 */
    onChange: (value: TimePreset) => void;
}

interface TimePresetOption {
    value: TimePreset;
    labelKey: string;
}

const options: TimePresetOption[] = [
    { value: "all", labelKey: "search.filters.allTime" },
    { value: "today", labelKey: "search.filters.today" },
    { value: "week", labelKey: "search.filters.thisWeek" },
    { value: "month", labelKey: "search.filters.thisMonth" },
];

/**
 * 时间范围筛选器组件
 */
export function TimeRangeFilter({ value, onChange }: TimeRangeFilterProps) {
    const { t } = useTranslation();
    const [isOpen, setIsOpen] = React.useState(false);
    const containerRef = React.useRef<HTMLDivElement>(null);

    // 获取当前选中的显示名称
    const selectedOption = options.find((o) => o.value === value) || options[0];
    const displayName = t(selectedOption.labelKey);

    // 点击外部关闭下拉
    React.useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (
                containerRef.current &&
                !containerRef.current.contains(event.target as Node)
            ) {
                setIsOpen(false);
            }
        };

        if (isOpen) {
            document.addEventListener("mousedown", handleClickOutside);
            return () => {
                document.removeEventListener("mousedown", handleClickOutside);
            };
        }
    }, [isOpen]);

    // 键盘导航
    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === "Escape") {
            setIsOpen(false);
        } else if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            setIsOpen(!isOpen);
        }
    };

    return (
        <div ref={containerRef} className="relative">
            <button
                type="button"
                onClick={() => setIsOpen(!isOpen)}
                onKeyDown={handleKeyDown}
                className={cn(
                    "flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-xs",
                    "bg-muted/50 hover:bg-muted transition-colors",
                    "border border-transparent hover:border-border",
                    "focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-1"
                )}
                aria-expanded={isOpen}
                aria-haspopup="listbox"
            >
                <Calendar className="w-3.5 h-3.5 text-muted-foreground" />
                <span className="text-foreground">{displayName}</span>
                <ChevronDown
                    className={cn(
                        "w-3.5 h-3.5 text-muted-foreground transition-transform",
                        isOpen && "rotate-180"
                    )}
                />
            </button>

            {/* Dropdown Menu */}
            {isOpen && (
                <div
                    role="listbox"
                    className={cn(
                        "absolute top-full left-0 mt-1 z-50",
                        "min-w-[140px]",
                        "bg-popover border border-border rounded-md shadow-lg",
                        "overflow-hidden",
                        "animate-in fade-in-0 zoom-in-95"
                    )}
                >
                    {options.map((option) => (
                        <button
                            key={option.value}
                            type="button"
                            role="option"
                            aria-selected={value === option.value}
                            onClick={() => {
                                onChange(option.value);
                                setIsOpen(false);
                            }}
                            className={cn(
                                "flex items-center justify-between w-full px-3 py-2 text-sm",
                                "hover:bg-accent transition-colors",
                                value === option.value && "bg-accent"
                            )}
                        >
                            <span>{t(option.labelKey)}</span>
                            {value === option.value && (
                                <Check className="w-4 h-4 text-primary" />
                            )}
                        </button>
                    ))}
                </div>
            )}
        </div>
    );
}

export default TimeRangeFilter;
