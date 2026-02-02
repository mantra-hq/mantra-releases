/**
 * InsertMessageSheet - 消息插入 Sheet 组件
 * Story 10.5: Task 2
 * Story 12.1: Task 4 - Dialog → Sheet 改造
 * Story 12.4: 迁移使用 ActionSheet 统一封装组件
 *
 * AC2: 弹出 Sheet，显示角色选择和内容输入
 * AC3: 确认后调用回调
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { User, Bot } from "lucide-react";
import {
  ActionSheet,
  ActionSheetContent,
  ActionSheetHeader,
  ActionSheetTitle,
  ActionSheetDescription,
  ActionSheetFooter,
} from "@/components/ui/action-sheet";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { Label } from "@/components/ui/label";
import type { NarrativeMessage } from "@/types/message";
import { estimateTokenCount } from "@/lib/token-counter";

/**
 * InsertMessageSheet 组件 Props
 * Story 12.1: 重命名 Props 接口
 */
export interface InsertMessageSheetProps {
  /** Sheet 是否打开 */
  open: boolean;
  /** Sheet 状态变化回调 */
  onOpenChange: (open: boolean) => void;
  /** 确认插入回调 */
  onConfirm: (message: NarrativeMessage) => void;
  /** 插入位置描述 (用于显示) */
  insertPosition?: string;
  /** 初始消息 (用于编辑模式) */
  initialMessage?: NarrativeMessage | null;
}

/**
 * 创建插入的消息
 */
function createInsertedMessage(
  role: "user" | "assistant",
  content: string,
  afterIndex: number
): NarrativeMessage {
  return {
    id: `inserted-${afterIndex}-${Date.now()}`,
    role,
    content: [{ type: "text", content }],
    timestamp: new Date().toISOString(),
  };
}

/**
 * InsertMessageSheet - 插入消息 Sheet
 * Story 12.1: Dialog → Sheet 改造
 *
 * AC2: 角色选择 + 内容输入 + Token 统计
 * AC3: 确认插入回调
 */
export function InsertMessageSheet({
  open,
  onOpenChange,
  onConfirm,
  insertPosition,
  initialMessage,
}: InsertMessageSheetProps) {
  const { t } = useTranslation();

  // 角色状态 (默认 user)
  const [role, setRole] = React.useState<"user" | "assistant">("user");
  // 内容状态
  const [content, setContent] = React.useState("");
  // Token 计数状态
  const [tokenCount, setTokenCount] = React.useState(0);
  const [isCalculatingTokens, setIsCalculatingTokens] = React.useState(false);

  // 是否为编辑模式
  const isEditMode = !!initialMessage;

  // 解析 afterIndex 从 insertPosition
  // [Fix #2] 直接使用 parseInt 支持负数索引
  const afterIndex = React.useMemo(() => {
    if (!insertPosition) return 0;
    const parsed = parseInt(insertPosition, 10);
    return Number.isNaN(parsed) ? 0 : parsed;
  }, [insertPosition]);

  // 获取初始消息的文本内容
  const getInitialContent = React.useCallback((msg: NarrativeMessage | null | undefined): string => {
    if (!msg?.content) return "";
    return msg.content
      .filter((block) => block.type === "text")
      .map((block) => block.content)
      .join("\n");
  }, []);

  // Sheet 打开时设置初始状态 (支持新建和编辑)
  React.useEffect(() => {
    if (open) {
      if (initialMessage) {
        // 编辑模式：使用现有消息的内容
        setRole(initialMessage.role as "user" | "assistant");
        setContent(getInitialContent(initialMessage));
      } else {
        // 新建模式：重置为默认值
        setRole("user");
        setContent("");
      }
      setTokenCount(0);
    }
  }, [open, initialMessage, getInitialContent]);

  // 使用 debounce 计算 Token (150ms)
  React.useEffect(() => {
    if (!content) {
      setTokenCount(0);
      return;
    }

    setIsCalculatingTokens(true);
    const timer = setTimeout(() => {
      setTokenCount(estimateTokenCount(content));
      setIsCalculatingTokens(false);
    }, 150);

    return () => clearTimeout(timer);
  }, [content]);

  // 处理内容变化
  const handleContentChange = React.useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      setContent(e.target.value);
    },
    []
  );

  // 处理确认
  const handleConfirm = React.useCallback(() => {
    if (content.trim()) {
      const message = createInsertedMessage(role, content.trim(), afterIndex);
      onConfirm(message);
      onOpenChange(false);
    }
  }, [content, role, afterIndex, onConfirm, onOpenChange]);

  // 处理取消
  const handleCancel = React.useCallback(() => {
    onOpenChange(false);
  }, [onOpenChange]);

  // 处理键盘快捷键
  const handleKeyDown = React.useCallback(
    (e: React.KeyboardEvent) => {
      // Ctrl/Cmd + Enter 确认
      if ((e.ctrlKey || e.metaKey) && e.key === "Enter" && content.trim()) {
        e.preventDefault();
        handleConfirm();
      }
      // Escape 取消 - Sheet 已有内置处理
    },
    [content, handleConfirm]
  );

  // 确认按钮是否禁用
  const isConfirmDisabled = !content.trim();

  return (
    <ActionSheet open={open} onOpenChange={onOpenChange}>
      <ActionSheetContent
        size="lg"
        className="flex flex-col"
        data-testid="insert-message-sheet"
        onKeyDown={handleKeyDown}
      >
        <ActionSheetHeader>
          <ActionSheetTitle>
            {isEditMode
              ? t("compress.insertDialog.titleEdit")
              : t("compress.insertDialog.title")}
          </ActionSheetTitle>
          <ActionSheetDescription>
            {isEditMode
              ? t("compress.insertDialog.descriptionEdit")
              : t("compress.insertDialog.description")}
          </ActionSheetDescription>
        </ActionSheetHeader>

        {/* 角色选择 */}
        <div className="flex-shrink-0 px-4">
          <Label className="text-sm text-muted-foreground mb-2 block">
            {t("compress.insertDialog.roleLabel")}
          </Label>
          <div className="flex gap-2" data-testid="role-toggle-group">
            <Button
              type="button"
              variant={role === "user" ? "default" : "outline"}
              size="sm"
              onClick={() => setRole("user")}
              className="gap-1.5"
              data-testid="role-user-button"
              data-state={role === "user" ? "on" : "off"}
            >
              <User className="size-4" />
              {t("compress.insertDialog.roleUser")}
            </Button>
            <Button
              type="button"
              variant={role === "assistant" ? "default" : "outline"}
              size="sm"
              onClick={() => setRole("assistant")}
              className="gap-1.5"
              data-testid="role-assistant-button"
              data-state={role === "assistant" ? "on" : "off"}
            >
              <Bot className="size-4" />
              {t("compress.insertDialog.roleAssistant")}
            </Button>
          </div>
        </div>

        {/* 内容输入 */}
        <div className="flex-1 min-h-0 flex flex-col px-4">
          <div className="flex justify-between items-center mb-1">
            <Label className="text-sm text-muted-foreground">
              {t("compress.insertDialog.contentLabel")}
            </Label>
            <span
              className="text-xs text-muted-foreground"
              data-testid="token-count-display"
            >
              <span className={isCalculatingTokens ? "opacity-50" : ""}>
                {tokenCount}
              </span>{" "}
              {t("compress.insertDialog.tokens")}
            </span>
          </div>
          <Textarea
            value={content}
            onChange={handleContentChange}
            className="flex-1 min-h-[20vh] resize-none font-mono text-sm"
            placeholder={t("compress.insertDialog.placeholder")}
            data-testid="content-input"
            autoFocus
          />
        </div>

        <ActionSheetFooter className="px-4">
          <Button
            variant="outline"
            onClick={handleCancel}
            data-testid="cancel-button"
          >
            {t("compress.insertDialog.cancel")}
          </Button>
          <Button
            onClick={handleConfirm}
            disabled={isConfirmDisabled}
            data-testid="confirm-button"
          >
            {isEditMode
              ? t("compress.insertDialog.confirmEdit")
              : t("compress.insertDialog.confirm")}
          </Button>
        </ActionSheetFooter>
      </ActionSheetContent>
    </ActionSheet>
  );
}

export default InsertMessageSheet;
