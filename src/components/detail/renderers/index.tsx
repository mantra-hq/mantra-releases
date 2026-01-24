/**
 * 渲染器映射表
 * Story 2.15: Task 6.6
 * Story 8.12: Task 8 - 使用 standardTool.type 选择渲染器
 *
 * 根据 standardTool 类型选择对应的渲染器
 */

/* eslint-disable react-refresh/only-export-components */

import * as React from "react";
import { TerminalRenderer } from "./TerminalRenderer";
import { FileRenderer } from "./FileRenderer";
import { SearchResultRenderer } from "./SearchResultRenderer";
import { GenericRenderer } from "./GenericRenderer";
import type { StandardTool } from "@/types/message";
import { isTerminalTool, isFileTool, isSearchTool, getToolPath } from "@/lib/tool-utils";

/** 渲染器 Props */
export interface RendererProps {
    content: string;
    toolName: string;
    toolInput?: Record<string, unknown>;
    /** Story 8.12: 标准化工具 */
    standardTool?: StandardTool;
    isError?: boolean;
    onResultClick?: (file: string, line: number) => void;
}

/** 渲染器组件类型 */
type RendererComponent = React.FC<RendererProps>;

/**
 * 获取工具对应的渲染器
 * Story 8.12: 使用 standardTool.type 选择渲染器
 */
export function getToolRenderer(standardTool?: StandardTool): RendererComponent {
    // 终端类工具
    if (isTerminalTool(standardTool)) {
        return ({ content, isError }) => (
            <TerminalRenderer content={content} isError={isError} />
        );
    }

    // 文件类工具
    if (isFileTool(standardTool)) {
        return ({ content, standardTool: st }) => (
            <FileRenderer content={content} filePath={getToolPath(st)} />
        );
    }

    // 搜索类工具
    if (isSearchTool(standardTool)) {
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
 * Story 8.12: 使用 standardTool.type 选择渲染器
 *
 * 根据工具类型自动选择合适的渲染器
 */
export function ToolOutputRenderer(props: RendererProps) {
    const Renderer = React.useMemo(
        () => getToolRenderer(props.standardTool),
        [props.standardTool]
    );

    return <Renderer {...props} />;
}

export default ToolOutputRenderer;
