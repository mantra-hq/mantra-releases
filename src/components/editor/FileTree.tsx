/**
 * FileTree - 文件树组件
 * Story 2.13: Task 4 - AC #8, #9, #10, #11, #12, #13, #14
 *
 * 功能:
 * - 显示目录树结构
 * - 单击预览 / 双击打开文件
 * - 目录折叠/展开
 * - 当前文件高亮
 * - 虚拟化渲染 (支持大型仓库)
 */

import * as React from "react";
import { ChevronRight, ChevronDown, Folder, FolderOpen } from "lucide-react";
import { cn } from "@/lib/utils";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useEditorStore } from "@/stores/useEditorStore";
import { getFileIcon } from "@/lib/file-icons";

/** 树节点数据结构 (与 Rust 后端对齐) */
export interface TreeNode {
    /** 文件/目录名 */
    name: string;
    /** 完整路径 */
    path: string;
    /** 节点类型 */
    type: "file" | "directory";
    /** 子节点 (仅目录) */
    children?: TreeNode[];
}

export interface FileTreeProps {
    /** 树数据 */
    tree: TreeNode[];
    /** 当前激活的文件路径 */
    activeFilePath?: string;
    /** 文件单击回调 (预览) */
    onFileClick?: (path: string) => void;
    /** 文件双击回调 (打开) */
    onFileDoubleClick?: (path: string) => void;
    /** 自定义类名 */
    className?: string;
}

/** 扁平化树节点 (用于虚拟化) */
interface FlatNode {
    node: TreeNode;
    depth: number;
    isExpanded: boolean;
}

/**
 * 文件树组件
 */
export function FileTree({
    tree,
    activeFilePath,
    onFileClick,
    onFileDoubleClick,
    className,
}: FileTreeProps) {
    const { expandedFolders, toggleFolder } = useEditorStore();
    const parentRef = React.useRef<HTMLDivElement>(null);

    // 扁平化树结构
    const flatNodes = React.useMemo(() => {
        const result: FlatNode[] = [];

        const flatten = (nodes: TreeNode[], depth: number) => {
            for (const node of nodes) {
                const isExpanded = expandedFolders.has(node.path);
                result.push({ node, depth, isExpanded });

                if (node.type === "directory" && isExpanded && node.children) {
                    flatten(node.children, depth + 1);
                }
            }
        };

        flatten(tree, 0);
        return result;
    }, [tree, expandedFolders]);

    // 是否启用虚拟化
    // AC #14 要求 >1000 文件时启用，但实测 500+ 时性能下降明显
    // 使用更保守的阈值以确保流畅体验
    const useVirtualization = flatNodes.length > 500;

    // 虚拟化 (AC #14)
    const virtualizer = useVirtualizer({
        count: useVirtualization ? flatNodes.length : 0,
        getScrollElement: () => parentRef.current,
        estimateSize: () => 24, // 行高
        overscan: 10,
    });

    const handleNodeClick = (node: TreeNode) => {
        if (node.type === "directory") {
            toggleFolder(node.path);
        } else {
            onFileClick?.(node.path);
        }
    };

    const handleNodeDoubleClick = (node: TreeNode) => {
        if (node.type === "file") {
            onFileDoubleClick?.(node.path);
        }
    };

    // 渲染单个节点
    const renderNode = (flatNode: FlatNode, _index: number, style?: React.CSSProperties) => {
        const { node, depth, isExpanded } = flatNode;
        const isActive = node.path === activeFilePath;
        const Icon =
            node.type === "directory"
                ? isExpanded
                    ? FolderOpen
                    : Folder
                : getFileIcon(node.path);

        return (
            <div
                key={node.path}
                data-node
                data-active={isActive}
                role="treeitem"
                aria-selected={isActive}
                aria-expanded={node.type === "directory" ? isExpanded : undefined}
                style={style}
                onClick={() => handleNodeClick(node)}
                onDoubleClick={() => handleNodeDoubleClick(node)}
                className={cn(
                    "flex items-center gap-1 px-2 py-0.5 cursor-pointer",
                    "hover:bg-muted/50 transition-colors",
                    isActive && "bg-accent text-accent-foreground"
                )}
            >
                {/* 缩进 */}
                <div style={{ width: depth * 16 }} className="flex-shrink-0" />

                {/* 展开/折叠图标 */}
                {node.type === "directory" ? (
                    <span className="w-4 h-4 flex items-center justify-center flex-shrink-0">
                        {isExpanded ? (
                            <ChevronDown className="h-3 w-3" />
                        ) : (
                            <ChevronRight className="h-3 w-3" />
                        )}
                    </span>
                ) : (
                    <span className="w-4 flex-shrink-0" />
                )}

                {/* 文件/文件夹图标 */}
                <Icon
                    className={cn(
                        "h-4 w-4 flex-shrink-0",
                        node.type === "directory" && "text-amber-500"
                    )}
                />

                {/* 文件名 */}
                <span className="truncate text-sm">{node.name}</span>
            </div>
        );
    };

    return (
        <div
            ref={parentRef}
            role="tree"
            className={cn("h-full overflow-auto", className)}
        >
            {useVirtualization ? (
                // 虚拟化渲染 (大型仓库)
                <div
                    style={{
                        height: `${virtualizer.getTotalSize()}px`,
                        width: "100%",
                        position: "relative",
                    }}
                >
                    {virtualizer.getVirtualItems().map((virtualRow) => 
                        renderNode(flatNodes[virtualRow.index], virtualRow.index, {
                            position: "absolute",
                            top: 0,
                            left: 0,
                            width: "100%",
                            height: `${virtualRow.size}px`,
                            transform: `translateY(${virtualRow.start}px)`,
                        })
                    )}
                </div>
            ) : (
                // 普通渲染 (小型仓库，更好的测试兼容性)
                <div>
                    {flatNodes.map((flatNode, index) => renderNode(flatNode, index))}
                </div>
            )}
        </div>
    );
}

export default FileTree;

