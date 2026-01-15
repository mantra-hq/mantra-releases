/**
 * ContentTypeFilter - 内容类型筛选器
 * Story 2.33: Task 3.2
 *
 * AC1: 类型选项：全部 | 代码 | 对话
 * Tab 样式切换
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { Code, MessageSquare, Layers } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ContentType } from "@/stores/useSearchStore";

export interface ContentTypeFilterProps {
    /** 当前选中的内容类型 */
    value: ContentType;
    /** 内容类型变化回调 */
    onChange: (value: ContentType) => void;
}

interface ContentTypeOption {
    value: ContentType;
    labelKey: string;
    icon: React.ReactNode;
}

const options: ContentTypeOption[] = [
    { value: "all", labelKey: "search.filters.all", icon: <Layers className="w-3.5 h-3.5" /> },
    { value: "code", labelKey: "search.filters.code", icon: <Code className="w-3.5 h-3.5" /> },
    {
        value: "conversation",
        labelKey: "search.filters.conversation",
        icon: <MessageSquare className="w-3.5 h-3.5" />,
    },
];

/**
 * 内容类型筛选器组件
 */
export function ContentTypeFilter({ value, onChange }: ContentTypeFilterProps) {
    const { t } = useTranslation();

    return (
        <div className="flex items-center gap-1 p-0.5 bg-muted rounded-md">
            {options.map((option) => (
                <button
                    key={option.value}
                    type="button"
                    onClick={() => onChange(option.value)}
                    className={cn(
                        "flex items-center gap-1.5 px-2.5 py-1 rounded text-xs font-medium transition-colors",
                        "focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-1",
                        value === option.value
                            ? "bg-background text-foreground shadow-sm"
                            : "text-muted-foreground hover:text-foreground"
                    )}
                    aria-pressed={value === option.value}
                >
                    {option.icon}
                    <span>{t(option.labelKey)}</span>
                </button>
            ))}
        </div>
    );
}

export default ContentTypeFilter;
