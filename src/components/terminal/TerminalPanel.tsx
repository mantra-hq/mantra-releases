/**
 * TerminalPanel - xterm.js 终端渲染组件
 * Story 2.15: Task 2
 *
 * 封装 xterm.js 用于渲染终端输出，支持 ANSI 颜色和主题切换
 * AC: #8, #9
 */

import * as React from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import { cn } from "@/lib/utils";

// 导入 xterm 样式
import "@xterm/xterm/css/xterm.css";

export interface TerminalPanelProps {
    /** 终端内容文本 */
    content: string;
    /** 是否深色主题 */
    isDark?: boolean;
    /** 自定义 className */
    className?: string;
    /** 最小高度 (px) */
    minHeight?: number;
    /** 最大高度 (px) */
    maxHeight?: number;
}

/** 深色主题配置 */
const DARK_THEME = {
    background: "#09090b", // zinc-950
    foreground: "#fafafa", // zinc-50
    cursor: "#3b82f6", // blue-500
    cursorAccent: "#09090b",
    selectionBackground: "#3b82f680",
    black: "#18181b",
    red: "#ef4444",
    green: "#22c55e",
    yellow: "#eab308",
    blue: "#3b82f6",
    magenta: "#a855f7",
    cyan: "#06b6d4",
    white: "#fafafa",
    brightBlack: "#71717a",
    brightRed: "#f87171",
    brightGreen: "#4ade80",
    brightYellow: "#facc15",
    brightBlue: "#60a5fa",
    brightMagenta: "#c084fc",
    brightCyan: "#22d3ee",
    brightWhite: "#ffffff",
};

/** 浅色主题配置 */
const LIGHT_THEME = {
    background: "#ffffff",
    foreground: "#09090b",
    cursor: "#3b82f6",
    cursorAccent: "#ffffff",
    selectionBackground: "#3b82f640",
    black: "#09090b",
    red: "#dc2626",
    green: "#16a34a",
    yellow: "#ca8a04",
    blue: "#2563eb",
    magenta: "#9333ea",
    cyan: "#0891b2",
    white: "#fafafa",
    brightBlack: "#a1a1aa",
    brightRed: "#ef4444",
    brightGreen: "#22c55e",
    brightYellow: "#eab308",
    brightBlue: "#3b82f6",
    brightMagenta: "#a855f7",
    brightCyan: "#06b6d4",
    brightWhite: "#ffffff",
};

/**
 * TerminalPanel 组件
 *
 * 使用 xterm.js 渲染终端输出，支持:
 * - ANSI 颜色转义序列
 * - 自适应容器大小
 * - 深色/浅色主题
 * - 可点击链接
 */
export function TerminalPanel({
    content,
    isDark = true,
    className,
    minHeight = 100,
    maxHeight = 400,
}: TerminalPanelProps) {
    const containerRef = React.useRef<HTMLDivElement>(null);
    const terminalRef = React.useRef<Terminal | null>(null);
    const fitAddonRef = React.useRef<FitAddon | null>(null);

    // 初始化终端
    React.useEffect(() => {
        if (!containerRef.current) return;

        const terminal = new Terminal({
            theme: isDark ? DARK_THEME : LIGHT_THEME,
            fontFamily: "JetBrains Mono, Fira Code, Consolas, monospace",
            fontSize: 13,
            letterSpacing: 0,
            lineHeight: 1.0,
            customGlyphs: true,
            scrollback: 1000,
            cursorBlink: false,
            cursorStyle: "block",
            disableStdin: true, // 禁用输入
            convertEol: true, // 转换换行符
        });

        const fitAddon = new FitAddon();
        const webLinksAddon = new WebLinksAddon();

        terminal.loadAddon(fitAddon);
        terminal.loadAddon(webLinksAddon);

        terminal.open(containerRef.current);

        // 写入内容
        if (content) {
            terminal.write(content);
        }

        // 适配容器大小
        try {
            fitAddon.fit();
        } catch {
            // 容器可能尚未完全渲染
        }

        terminalRef.current = terminal;
        fitAddonRef.current = fitAddon;

        return () => {
            terminal.dispose();
            terminalRef.current = null;
            fitAddonRef.current = null;
        };
    }, [content, isDark]);

    // 监听容器大小变化
    React.useEffect(() => {
        if (!containerRef.current || !fitAddonRef.current) return;

        const resizeObserver = new ResizeObserver(() => {
            try {
                fitAddonRef.current?.fit();
            } catch {
                // 忽略 fit 错误
            }
        });

        resizeObserver.observe(containerRef.current);

        return () => {
            resizeObserver.disconnect();
        };
    }, []);

    // 主题变化时更新
    React.useEffect(() => {
        if (terminalRef.current) {
            terminalRef.current.options.theme = isDark ? DARK_THEME : LIGHT_THEME;
        }
    }, [isDark]);

    return (
        <div
            ref={containerRef}
            data-testid="terminal-panel"
            className={cn(
                "rounded-md overflow-hidden",
                "border border-border",
                isDark ? "bg-zinc-950" : "bg-white",
                className
            )}
            style={{
                minHeight: `${minHeight}px`,
                maxHeight: `${maxHeight}px`,
            }}
        />
    );
}

export default TerminalPanel;
