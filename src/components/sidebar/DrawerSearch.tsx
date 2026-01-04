/**
 * DrawerSearch Component - 抽屉搜索框
 * Story 2.18: Task 5
 * Story 2.26: 国际化支持
 *
 * 搜索输入框，支持实时过滤项目和会话
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { Search, X } from "lucide-react";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

/**
 * DrawerSearch Props
 */
export interface DrawerSearchProps {
  /** 搜索值 */
  value: string;
  /** 值变更回调 */
  onChange: (value: string) => void;
  /** 占位符文本 */
  placeholder?: string;
  /** 额外的 className */
  className?: string;
}

/**
 * DrawerSearch 组件
 * 带图标的搜索输入框
 */
export function DrawerSearch({
  value,
  onChange,
  placeholder,
  className,
}: DrawerSearchProps) {
  const { t } = useTranslation();
  const inputRef = React.useRef<HTMLInputElement>(null);

  // 清空搜索
  const handleClear = React.useCallback(() => {
    onChange("");
    inputRef.current?.focus();
  }, [onChange]);

  return (
    <div className={cn("relative", className)}>
      <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
      <Input
        ref={inputRef}
        type="text"
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder || t("common.search")}
        className="pl-9 pr-8 h-9"
        data-testid="drawer-search-input"
      />
      {value && (
        <button
          type="button"
          onClick={handleClear}
          className="absolute right-2 top-1/2 -translate-y-1/2 h-5 w-5 flex items-center justify-center rounded-sm hover:bg-muted"
          aria-label={t("common.clearSearch")}
          data-testid="drawer-search-clear"
        >
          <X className="h-3 w-3 text-muted-foreground" />
        </button>
      )}
    </div>
  );
}

/**
 * HighlightText Component - 高亮文本
 * 用于搜索结果中高亮匹配的文字
 */
export interface HighlightTextProps {
  /** 原始文本 */
  text: string;
  /** 搜索关键词 */
  keyword?: string;
  /** 额外的 className */
  className?: string;
}

export function HighlightText({
  text,
  keyword,
  className,
}: HighlightTextProps) {
  if (!keyword || !keyword.trim()) {
    return <span className={className}>{text}</span>;
  }

  // 转义正则表达式特殊字符
  const escapedKeyword = keyword.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const regex = new RegExp(`(${escapedKeyword})`, "gi");
  const parts = text.split(regex);

  return (
    <span className={className}>
      {parts.map((part, i) =>
        regex.test(part) ? (
          <mark
            key={i}
            className="bg-primary/20 text-primary rounded-sm px-0.5"
          >
            {part}
          </mark>
        ) : (
          <React.Fragment key={i}>{part}</React.Fragment>
        )
      )}
    </span>
  );
}
