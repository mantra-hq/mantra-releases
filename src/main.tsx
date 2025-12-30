/**
 * Main Entry Point - 应用入口
 * Story 2.8: Task 1
 *
 * 配置路由和全局 Providers
 */

import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { Dashboard, Player } from "./routes";
import { ThemeProvider } from "./lib/theme-provider";
import { TooltipProvider } from "./components/ui/tooltip";
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

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="system">
      <TooltipProvider delayDuration={300}>
        <BrowserRouter>
          <Routes>
            {/* Dashboard: 项目列表页 */}
            <Route path="/" element={<Dashboard />} />
            {/* Player: 会话回放页 */}
            <Route path="/session/:sessionId" element={<Player />} />
            {/* 默认重定向到 Dashboard */}
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </BrowserRouter>
      </TooltipProvider>
    </ThemeProvider>
  </React.StrictMode>
);
