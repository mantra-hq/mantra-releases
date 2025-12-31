/**
 * DiffLine 组件 - Story 3-2
 * 渲染单行差异
 */

import { cn } from '@/lib/utils';
import type { DiffLine } from './types';

interface DiffLineComponentProps {
    line: DiffLine;
}

export function DiffLineComponent({ line }: DiffLineComponentProps) {
    return (
        <div
            className={cn(
                'flex px-4 py-0.5 font-mono text-sm',
                line.type === 'added' && 'bg-diff-add',
                line.type === 'removed' && 'bg-diff-remove'
            )}
        >
            {/* 原始行号 */}
            <span className="w-12 text-muted-foreground text-right pr-4 select-none shrink-0">
                {line.lineNumber.original ?? ''}
            </span>
            {/* 脱敏后行号 */}
            <span className="w-12 text-muted-foreground text-right pr-4 select-none shrink-0">
                {line.lineNumber.sanitized ?? ''}
            </span>
            {/* 变更标记 */}
            <span className="w-4 text-center select-none shrink-0">
                {line.type === 'added' && (
                    <span className="text-green-600 dark:text-green-400">+</span>
                )}
                {line.type === 'removed' && (
                    <span className="text-red-600 dark:text-red-400">-</span>
                )}
            </span>
            {/* 内容 */}
            <span className="flex-1 whitespace-pre-wrap break-all">
                {line.content || '\u00A0'} {/* 空行显示不可见空格以保持高度 */}
            </span>
        </div>
    );
}
