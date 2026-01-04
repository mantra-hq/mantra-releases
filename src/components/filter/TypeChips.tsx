/**
 * TypeChips - 消息类型过滤 Chips 组件
 * Story 2.16: Task 2
 * Story 2.26: 国际化支持
 *
 * 支持多选的过滤类型 Chips，使用 OR 逻辑过滤消息
 * AC: #1, #2, #3
 */


import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { MESSAGE_TYPES, useMessageFilterStore } from "@/stores/useMessageFilterStore";

export interface TypeChipsProps {
    /** 自定义 className */
    className?: string;
}

/**
 * TypeChips 组件
 * 显示可多选的消息类型过滤芯片
 */
export function TypeChips({ className }: TypeChipsProps) {
    const { t } = useTranslation();
    const { selectedTypes, toggleType } = useMessageFilterStore();

    return (
        <div
            className={cn("flex flex-wrap gap-1.5", className)}
            role="group"
            aria-label={t("filter.messageTypeFilter")}
        >
            {MESSAGE_TYPES.map((type) => {
                const isSelected = selectedTypes.has(type.id);
                return (
                    <button
                        key={type.id}
                        type="button"
                        onClick={() => toggleType(type.id)}
                        aria-pressed={isSelected}
                        className={cn(
                            // 基础样式
                            "inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium cursor-pointer",
                            "transition-all duration-150 select-none",
                            // 未选中样式
                            !isSelected && [
                                "bg-muted/50 text-muted-foreground",
                                "hover:bg-muted hover:text-foreground",
                                "border border-transparent",
                            ],
                            // 选中样式
                            isSelected && [
                                "bg-primary/10 text-primary",
                                "border border-primary/30",
                                "hover:bg-primary/20",
                            ],
                            // 焦点样式
                            "focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1"
                        )}
                    >
                        <span className="text-sm" aria-hidden="true">
                            {type.icon}
                        </span>
                        <span>{t(type.label)}</span>
                    </button>
                );
            })}
        </div>
    );
}

export default TypeChips;
