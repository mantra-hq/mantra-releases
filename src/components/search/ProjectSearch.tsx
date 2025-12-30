/**
 * ProjectSearch Component - 项目搜索组件
 * Story 2.8: Task 6
 *
 * 带搜索图标的输入框，支持防抖搜索
 */

import * as React from "react";
import { Search } from "lucide-react";
import { Input } from "@/components/ui/input";
import { useDebouncedValue } from "@/hooks/useDebouncedValue";
import { cn } from "@/lib/utils";

/**
 * ProjectSearch Props
 */
export interface ProjectSearchProps {
  /** 搜索回调 */
  onSearch: (query: string) => void;
  /** 占位符文本 */
  placeholder?: string;
  /** 自定义类名 */
  className?: string;
}

/**
 * ProjectSearch 组件
 * 带搜索图标和防抖功能的搜索输入框
 */
export function ProjectSearch({
  onSearch,
  placeholder = "搜索项目...",
  className,
}: ProjectSearchProps) {
  // 输入值
  const [query, setQuery] = React.useState("");
  
  // 防抖后的值 (300ms)
  const debouncedQuery = useDebouncedValue(query, 300);

  // 跟踪是否为首次挂载，避免初始空搜索
  const isFirstRender = React.useRef(true);

  // 防抖值变化时触发搜索 (跳过首次挂载的空字符串)
  React.useEffect(() => {
    if (isFirstRender.current) {
      isFirstRender.current = false;
      return;
    }
    onSearch(debouncedQuery);
  }, [debouncedQuery, onSearch]);

  // 输入变化处理
  const handleChange = React.useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      setQuery(event.target.value);
    },
    []
  );

  return (
    <div
      data-testid="project-search"
      className={cn("relative", className)}
    >
      {/* 搜索图标 */}
      <Search
        aria-hidden="true"
        className={cn(
          "absolute left-3 top-1/2 -translate-y-1/2",
          "w-4 h-4 text-muted-foreground pointer-events-none"
        )}
      />
      
      {/* 输入框 */}
      <Input
        type="text"
        value={query}
        onChange={handleChange}
        placeholder={placeholder}
        className="pl-9"
      />
    </div>
  );
}

