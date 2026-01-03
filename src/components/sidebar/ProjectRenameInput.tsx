/**
 * ProjectRenameInput Component - 项目重命名输入框
 * Story 2.19: Task 5
 *
 * 行内编辑组件，支持 Enter 保存、Esc 取消
 */

import * as React from "react";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";

/**
 * ProjectRenameInput Props
 */
export interface ProjectRenameInputProps {
  /** 初始名称 */
  initialName: string;
  /** 保存回调 */
  onSave: (newName: string) => void;
  /** 取消回调 */
  onCancel: () => void;
  /** 额外的 className */
  className?: string;
}

/**
 * ProjectRenameInput 组件
 * 行内编辑项目名称
 */
export function ProjectRenameInput({
  initialName,
  onSave,
  onCancel,
  className,
}: ProjectRenameInputProps) {
  const [name, setName] = React.useState(initialName);
  const inputRef = React.useRef<HTMLInputElement>(null);

  // 挂载时聚焦并选中全部文本
  React.useEffect(() => {
    if (inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, []);

  // 提交名称
  const handleSubmit = React.useCallback(() => {
    const trimmedName = name.trim();

    // 空名称或未更改时取消
    if (!trimmedName || trimmedName === initialName) {
      onCancel();
      return;
    }

    onSave(trimmedName);
  }, [name, initialName, onSave, onCancel]);

  // 键盘事件处理 (AC11)
  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      e.preventDefault();
      handleSubmit();
    } else if (e.key === "Escape") {
      e.preventDefault();
      onCancel();
    }
  };

  // 失焦时保存
  const handleBlur = () => {
    handleSubmit();
  };

  return (
    <Input
      ref={inputRef}
      type="text"
      value={name}
      onChange={(e) => setName(e.target.value)}
      onKeyDown={handleKeyDown}
      onBlur={handleBlur}
      className={cn(
        "h-6 py-0 px-1 text-sm",
        "bg-background border-primary",
        className
      )}
      data-testid="project-rename-input"
    />
  );
}
