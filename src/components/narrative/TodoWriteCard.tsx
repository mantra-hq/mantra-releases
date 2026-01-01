/**
 * TodoWriteCard - Todo 列表卡片组件
 * Story 2.15: TodoWrite 特化渲染
 *
 * 以 Todo List 形式展示 TodoWrite 工具调用
 */

import * as React from "react";
import * as Collapsible from "@radix-ui/react-collapsible";
import {
    ChevronDown,
    ChevronUp,
    ListTodo,
    CheckCircle2,
    Circle,
    Loader2,
} from "lucide-react";
import { cn } from "@/lib/utils";

/** Todo 项状态 */
type TodoStatus = "pending" | "in_progress" | "completed";

/** Todo 项数据 */
interface TodoItem {
    content: string;
    status: TodoStatus;
    activeForm?: string;
}

/** TodoWriteCard 属性 */
export interface TodoWriteCardProps {
    /** 唯一 ID */
    toolUseId: string;
    /** 工具输入参数 (包含 todos 数组) */
    toolInput?: Record<string, unknown>;
    /** 是否高亮 */
    isHighlighted?: boolean;
    /** 悬停回调 */
    onHover?: (toolUseId: string | null) => void;
    /** 自定义 className */
    className?: string;
}

/** 解析 todos 数组 */
function parseTodos(input?: Record<string, unknown>): TodoItem[] {
    if (!input || !Array.isArray(input.todos)) {
        return [];
    }

    return input.todos.map((item: unknown) => {
        if (typeof item === "object" && item !== null) {
            const obj = item as Record<string, unknown>;
            return {
                content: typeof obj.content === "string" ? obj.content : "",
                status: (obj.status as TodoStatus) || "pending",
                activeForm: typeof obj.activeForm === "string" ? obj.activeForm : undefined,
            };
        }
        return { content: "", status: "pending" as TodoStatus };
    }).filter((item) => item.content);
}

/** 获取状态图标 */
function StatusIcon({ status }: { status: TodoStatus }) {
    switch (status) {
        case "completed":
            return <CheckCircle2 className="h-4 w-4 text-green-500 shrink-0" />;
        case "in_progress":
            return <Loader2 className="h-4 w-4 text-blue-500 shrink-0 animate-spin" />;
        default:
            return <Circle className="h-4 w-4 text-muted-foreground shrink-0" />;
    }
}

/** 获取统计信息 */
function getStats(todos: TodoItem[]) {
    const completed = todos.filter((t) => t.status === "completed").length;
    const inProgress = todos.filter((t) => t.status === "in_progress").length;
    const total = todos.length;
    return { completed, inProgress, total };
}

/**
 * TodoWriteCard 组件
 *
 * 以 Todo List 形式展示 TodoWrite 工具调用：
 * - 显示任务列表带状态图标
 * - 统计完成/进行中/总数
 * - 可展开查看详情
 */
export function TodoWriteCard({
    toolUseId,
    toolInput,
    isHighlighted = false,
    onHover,
    className,
}: TodoWriteCardProps) {
    const [isExpanded, setIsExpanded] = React.useState(false);
    const todos = parseTodos(toolInput);
    const { completed, inProgress, total } = getStats(todos);

    const handleMouseEnter = React.useCallback(() => {
        onHover?.(toolUseId);
    }, [onHover, toolUseId]);

    const handleMouseLeave = React.useCallback(() => {
        onHover?.(null);
    }, [onHover]);

    if (todos.length === 0) {
        return null;
    }

    return (
        <div
            data-testid="todo-write-card"
            data-tool-use-id={toolUseId}
            className={cn(
                "rounded-lg border my-1.5 overflow-hidden transition-all duration-150",
                "border-border bg-muted/30",
                isHighlighted && "ring-2 ring-primary/50",
                className
            )}
            onMouseEnter={handleMouseEnter}
            onMouseLeave={handleMouseLeave}
        >
            <Collapsible.Root open={isExpanded} onOpenChange={setIsExpanded}>
                {/* 头部单行 */}
                <div className="flex items-center gap-2 px-3 py-2">
                    <ListTodo className="h-4 w-4 shrink-0 text-muted-foreground" />
                    <span className="font-medium text-sm text-foreground">TodoWrite</span>
                    <span className="text-muted-foreground">·</span>
                    <span className="text-sm text-muted-foreground">
                        {completed}/{total} 完成
                        {inProgress > 0 && `, ${inProgress} 进行中`}
                    </span>

                    <div className="flex-1" />

                    {/* 进度指示 */}
                    <div className="flex items-center gap-1">
                        {total > 0 && (
                            <div className="w-16 h-1.5 bg-muted rounded-full overflow-hidden">
                                <div
                                    className="h-full bg-green-500 transition-all duration-300"
                                    style={{ width: `${(completed / total) * 100}%` }}
                                />
                            </div>
                        )}
                    </div>

                    {/* 展开/折叠 */}
                    <Collapsible.Trigger
                        className={cn(
                            "p-1 rounded hover:bg-muted",
                            "text-muted-foreground hover:text-foreground",
                            "transition-colors"
                        )}
                        title={isExpanded ? "收起" : "展开"}
                    >
                        {isExpanded ? (
                            <ChevronUp className="h-3.5 w-3.5" />
                        ) : (
                            <ChevronDown className="h-3.5 w-3.5" />
                        )}
                    </Collapsible.Trigger>
                </div>

                {/* 展开的 Todo 列表 */}
                <Collapsible.Content
                    className={cn(
                        "overflow-hidden",
                        "data-[state=open]:animate-collapsible-down",
                        "data-[state=closed]:animate-collapsible-up"
                    )}
                >
                    <div className="border-t border-border bg-background px-3 py-2 space-y-1">
                        {todos.map((todo, index) => (
                            <div
                                key={index}
                                className={cn(
                                    "flex items-start gap-2 py-1 px-2 rounded",
                                    todo.status === "in_progress" && "bg-blue-500/10",
                                    todo.status === "completed" && "opacity-60"
                                )}
                            >
                                <StatusIcon status={todo.status} />
                                <span
                                    className={cn(
                                        "text-sm flex-1",
                                        todo.status === "completed" && "line-through text-muted-foreground"
                                    )}
                                >
                                    {todo.content}
                                </span>
                            </div>
                        ))}
                    </div>
                </Collapsible.Content>
            </Collapsible.Root>
        </div>
    );
}

export default TodoWriteCard;
