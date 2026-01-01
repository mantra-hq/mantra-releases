/**
 * TerminalRenderer - 终端类输出渲染器
 * Story 2.15: Task 6.2
 *
 * 使用 xterm.js 渲染命令输出
 * AC: #8, #9
 */

import { TerminalPanel } from "@/components/terminal";
import { cn } from "@/lib/utils";

export interface TerminalRendererProps {
    /** 输出内容 */
    content: string;
    /** 是否错误输出 */
    isError?: boolean;
    /** 自定义 className */
    className?: string;
}

/**
 * TerminalRenderer 组件
 *
 * 用于渲染命令类输出：
 * - run_command
 * - bash
 * - 错误堆栈
 */
export function TerminalRenderer({
    content,
    isError = false,
    className,
}: TerminalRendererProps) {
    return (
        <div
            data-testid="terminal-renderer"
            className={cn(
                isError && "border border-destructive rounded-md",
                className
            )}
        >
            <TerminalPanel
                content={content}
                isDark={true}
                minHeight={80}
                maxHeight={300}
            />
        </div>
    );
}

export default TerminalRenderer;
