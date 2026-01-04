/**
 * Main Entry Point - 应用入口
 * Story 2.8: Task 1
 * Story 2.10: Task 2.4 (Global Search Integration)
 * Story 2.21: Task 5 (移除 Dashboard，首页即 Player)
 *
 * 配置路由和全局 Providers
 */

import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { Player } from "./routes";
import { Settings } from "./routes/Settings";
import { ThemeProvider } from "./lib/theme-provider";
import { TooltipProvider } from "./components/ui/tooltip";
import { Toaster } from "./components/ui/sonner";
import { GlobalSearch } from "./components/search";
import { useGlobalShortcut } from "./hooks";
// Story 2-26: i18n 配置 (在导入 index.css 之前初始化)
import "./i18n";
import "./index.css";

// Prevent flash of incorrect theme on initial load
const initTheme = () => {
  const stored = localStorage.getItem("mantra-theme");
  const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;

  if (
    stored === "dark" ||
    (stored === "system" && prefersDark) ||
    (!stored && prefersDark)
  ) {
    document.documentElement.classList.add("dark");
  } else if (stored === "light" || (stored === "system" && !prefersDark)) {
    document.documentElement.classList.remove("dark");
  }
};

// Run before React hydration
initTheme();

/**
 * GlobalShortcutProvider - 全局快捷键 Provider
 * 在应用根级别注册全局快捷键
 */
function GlobalShortcutProvider({ children }: { children: React.ReactNode }) {
  useGlobalShortcut();
  return <>{children}</>;
}

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="system">
      <TooltipProvider delayDuration={300}>
        <BrowserRouter>
          <GlobalShortcutProvider>
            <Routes>
              {/* Story 2.21: 首页即 Player (空状态) */}
              <Route path="/" element={<Player />} />
              {/* Player: 会话回放页 */}
              <Route path="/session/:sessionId" element={<Player />} />
              {/* 兼容旧 URL: /player/:sessionId → /session/:sessionId */}
              <Route path="/player/:sessionId" element={<Player />} />
              {/* Settings: 设置页面 (Story 3-3) */}
              <Route path="/settings" element={<Settings />} />
              {/* 默认重定向到首页 */}
              <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
            {/* 全局搜索 Modal (Story 2.10) */}
            <GlobalSearch />
            {/* 全局 Toast 通知 */}
            <Toaster />
          </GlobalShortcutProvider>
        </BrowserRouter>
      </TooltipProvider>
    </ThemeProvider>
  </React.StrictMode>
);
