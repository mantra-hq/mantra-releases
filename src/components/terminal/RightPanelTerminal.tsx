/**
 * RightPanelTerminal - 右侧面板终端组件
 * Story 2.15: 终端类工具输出渲染
 *
 * 使用 xterm.js 渲染终端输出，支持 ANSI 颜色
 */

import * as React from "react";
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { WebLinksAddon } from "@xterm/addon-web-links";
import "@xterm/xterm/css/xterm.css";
import { cn } from "@/lib/utils";
import { Terminal as TerminalIcon, X } from "lucide-react";

export interface RightPanelTerminalProps {
    /** 命令 (可选显示) */
    command?: string;
    /** 输出内容 */
    output: string;
    /** 是否错误输出 */
    isError?: boolean;
    /** 退出码 */
    exitCode?: number;
    /** 关闭回调 */
    onClose?: () => void;
    /** 自定义 className */
    className?: string;
}

/**
 * RightPanelTerminal 组件
 *
 * 在右侧面板显示终端类工具的输出
 */
export function RightPanelTerminal({
    command,
    output,
    isError = false,
    exitCode,
    onClose,
    className,
}: RightPanelTerminalProps) {
    const terminalRef = React.useRef<HTMLDivElement>(null);
    const xtermRef = React.useRef<Terminal | null>(null);
    const fitAddonRef = React.useRef<FitAddon | null>(null);

    // 初始化 xterm
    React.useEffect(() => {
        if (!terminalRef.current) return;
        // 避免重复初始化
        if (xtermRef.current) return;

        const terminal = new Terminal({
            theme: {
                background: "#09090b", // zinc-950
                foreground: "#fafafa", // zinc-50
                cursor: "#3b82f6", // blue-500
                cursorAccent: "#09090b",
                selectionBackground: "#3b82f680",
                black: "#09090b",
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
            },
            fontFamily: "JetBrains Mono, Fira Code, monospace",
            fontSize: 13,
            letterSpacing: 0,
            lineHeight: 1.0,
            customGlyphs: true,
            scrollback: 1000,
            cursorBlink: false,
            disableStdin: true,
            convertEol: true,
        });

        const fitAddon = new FitAddon();
        const webLinksAddon = new WebLinksAddon();

        terminal.loadAddon(fitAddon);
        terminal.loadAddon(webLinksAddon);

        terminal.open(terminalRef.current);

        // 延迟 fit 确保布局稳定
        const fitTimer = setTimeout(() => {
            try {
                fitAddon.fit();
            } catch (e) {
                console.warn("[RightPanelTerminal] fitAddon.fit() failed:", e);
            }
        }, 50);

        xtermRef.current = terminal;
        fitAddonRef.current = fitAddon;

        // 监听窗口大小变化
        const handleResize = () => {
            try {
                fitAddon.fit();
            } catch (e) {
                // 忽略 fit 错误
            }
        };
        window.addEventListener("resize", handleResize);

        return () => {
            clearTimeout(fitTimer);
            window.removeEventListener("resize", handleResize);
            terminal.dispose();
            xtermRef.current = null;
            fitAddonRef.current = null;
        };
    }, []);

    // 写入内容
    React.useEffect(() => {
        const terminal = xtermRef.current;
        const fitAddon = fitAddonRef.current;
        if (!terminal || !fitAddon) return;

        terminal.clear();

        // 显示命令
        if (command) {
            terminal.writeln(`\x1b[32m$\x1b[0m ${command}`);
            terminal.writeln("");
        }

        // 显示输出
        if (output) {
            // 按行写入，保留 ANSI 颜色
            const lines = output.split("\n");
            for (const line of lines) {
                terminal.writeln(line);
            }
        }

        // 显示退出码
        if (exitCode !== undefined) {
            terminal.writeln("");
            if (exitCode === 0) {
                terminal.writeln(`\x1b[32m✓ Exit code: ${exitCode}\x1b[0m`);
            } else {
                terminal.writeln(`\x1b[31m✗ Exit code: ${exitCode}\x1b[0m`);
            }
        }

        // 延迟适配大小，避免布局问题
        requestAnimationFrame(() => {
            fitAddon.fit();
        });
    }, [command, output, exitCode]);

    return (
        <div
            data-testid="right-panel-terminal"
            className={cn(
                "flex flex-col h-full bg-[#09090b]",
                isError && "border-l-2 border-l-destructive",
                className
            )}
        >
            {/* 头部 */}
            <div className="flex items-center gap-2 px-4 py-2 border-b border-zinc-800 shrink-0">
                <TerminalIcon className="h-4 w-4 text-zinc-400" />
                <span className="text-sm text-zinc-300 truncate flex-1">
                    {command ? `$ ${command.slice(0, 50)}${command.length > 50 ? "..." : ""}` : "终端输出"}
                </span>
                {onClose && (
                    <button
                        type="button"
                        onClick={onClose}
                        className="p-1 rounded hover:bg-zinc-800 transition-colors"
                        aria-label="关闭终端"
                    >
                        <X className="h-4 w-4 text-zinc-400" />
                    </button>
                )}
            </div>

            {/* 终端区域 */}
            <div ref={terminalRef} className="flex-1 min-h-0 p-2" />
        </div>
    );
}

export default RightPanelTerminal;
