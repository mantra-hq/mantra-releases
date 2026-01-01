/**
 * 渲染器映射表
 * Story 2.15: Task 6.6
 *
 * 根据工具名称选择对应的渲染器
 */

import * as React from "react";
import { TerminalRenderer } from "./TerminalRenderer";
import { FileRenderer } from "./FileRenderer";
import { SearchResultRenderer } from "./SearchResultRenderer";
import { GenericRenderer } from "./GenericRenderer";

/** 渲染器 Props */
export interface RendererProps {
    content: string;
    toolName: string;
    toolInput?: Record<string, unknown>;
    isError?: boolean;
    onResultClick?: (file: string, line: number) => void;
}

/** 渲染器组件类型 */
type RendererComponent = React.FC<RendererProps>;

/** 工具类型分类 */
const TERMINAL_TOOLS = [
    "run_command",
    "bash",
    "execute_command",
    "send_command_input",
];

const FILE_TOOLS = [
    "read_file",
    "view_file",
    "write_to_file",
    "replace_file_content",
    "multi_replace_file_content",
];

const SEARCH_TOOLS = [
    "grep_search",
    "find_by_name",
    "codebase_search",
];

/** 检查工具是否属于某类 */
function isToolOfType(toolName: string, tools: string[]): boolean {
    const lowerName = toolName.toLowerCase();
    return tools.some((t) => lowerName.includes(t.toLowerCase()));
}

/** 提取文件路径 */
function extractFilePath(input?: Record<string, unknown>): string | undefined {
    if (!input) return undefined;
    const pathKeys = ["path", "file_path", "filePath", "file", "AbsolutePath", "TargetFile"];
    for (const key of pathKeys) {
        if (typeof input[key] === "string") {
            return input[key] as string;
        }
    }
    return undefined;
}

/**
 * 获取工具对应的渲染器
 */
export function getToolRenderer(toolName: string): RendererComponent {
    if (isToolOfType(toolName, TERMINAL_TOOLS)) {
        return ({ content, isError }) => (
            <TerminalRenderer content={content} isError={isError} />
        );
    }

    if (isToolOfType(toolName, FILE_TOOLS)) {
        return ({ content, toolInput }) => (
            <FileRenderer content={content} filePath={extractFilePath(toolInput)} />
        );
    }

    if (isToolOfType(toolName, SEARCH_TOOLS)) {
        return ({ content, onResultClick }) => (
            <SearchResultRenderer content={content} onResultClick={onResultClick} />
        );
    }

    // 兜底
    return ({ content, isError }) => (
        <GenericRenderer content={content} isError={isError} />
    );
}

/**
 * ToolOutputRenderer - 工具输出渲染器
 *
 * 根据工具类型自动选择合适的渲染器
 */
export function ToolOutputRenderer(props: RendererProps) {
    const Renderer = React.useMemo(
        () => getToolRenderer(props.toolName),
        [props.toolName]
    );

    return <Renderer {...props} />;
}

export default ToolOutputRenderer;
