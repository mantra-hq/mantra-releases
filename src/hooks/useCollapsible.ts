/**
 * useCollapsible - 可折叠内容 hook
 * Story 2.15: Task 7
 *
 * 管理折叠状态、浮动栏显示和键盘快捷键
 * AC: #11, #12, #13
 */

import * as React from "react";

export interface UseCollapsibleOptions {
    /** 默认是否展开 */
    defaultExpanded?: boolean;
    /** 折叠时回调 */
    onCollapse?: () => void;
}

export interface UseCollapsibleResult {
    /** 是否展开 */
    isExpanded: boolean;
    /** 切换展开状态 */
    toggle: () => void;
    /** 展开 */
    expand: () => void;
    /** 折叠 */
    collapse: () => void;
    /** 是否显示浮动栏 */
    showFloatingBar: boolean;
    /** 折叠按钮 ref (用于 IntersectionObserver) */
    collapseButtonRef: React.RefObject<HTMLButtonElement | null>;
    /** 内容容器 ref (用于滚动) */
    contentRef: React.RefObject<HTMLDivElement | null>;
    /** 滚动到顶部 */
    scrollToTop: () => void;
}

/**
 * useCollapsible hook
 *
 * 管理可折叠组件的状态：
 * - 展开/折叠状态
 * - Escape 键折叠
 * - 浮动栏显示（当折叠按钮不可见时）
 */
export function useCollapsible(
    options: UseCollapsibleOptions = {}
): UseCollapsibleResult {
    const { defaultExpanded = false, onCollapse } = options;

    const [isExpanded, setIsExpanded] = React.useState(defaultExpanded);
    const [showFloatingBar, setShowFloatingBar] = React.useState(false);
    const collapseButtonRef = React.useRef<HTMLButtonElement>(null);
    const contentRef = React.useRef<HTMLDivElement>(null);

    // 展开
    const expand = React.useCallback(() => {
        setIsExpanded(true);
    }, []);

    // 折叠
    const collapse = React.useCallback(() => {
        setIsExpanded(false);
        setShowFloatingBar(false);
        onCollapse?.();
    }, [onCollapse]);

    // 切换
    const toggle = React.useCallback(() => {
        if (isExpanded) {
            collapse();
        } else {
            expand();
        }
    }, [isExpanded, collapse, expand]);

    // 滚动到顶部
    const scrollToTop = React.useCallback(() => {
        contentRef.current?.scrollTo({ top: 0, behavior: "smooth" });
    }, []);

    // Escape 键折叠
    React.useEffect(() => {
        if (!isExpanded) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === "Escape") {
                collapse();
            }
        };

        document.addEventListener("keydown", handleKeyDown);
        return () => document.removeEventListener("keydown", handleKeyDown);
    }, [isExpanded, collapse]);

    // IntersectionObserver 监测折叠按钮可见性
    React.useEffect(() => {
        if (!isExpanded || !collapseButtonRef.current) {
            setShowFloatingBar(false);
            return;
        }

        const observer = new IntersectionObserver(
            ([entry]) => {
                // 当折叠按钮不可见且内容已展开时，显示浮动栏
                setShowFloatingBar(!entry.isIntersecting && isExpanded);
            },
            { threshold: 0 }
        );

        observer.observe(collapseButtonRef.current);

        return () => observer.disconnect();
    }, [isExpanded]);

    return {
        isExpanded,
        toggle,
        expand,
        collapse,
        showFloatingBar,
        collapseButtonRef,
        contentRef,
        scrollToTop,
    };
}

export default useCollapsible;
