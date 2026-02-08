/**
 * Main Entry Point - 应用入口
 * Story 2.8: Task 1
 * Story 2.10: Task 2.4 (Global Search Integration)
 * Story 2.21: Task 5 (移除 Dashboard，首页即 Player)
 * Story 9.2: Task 2 (Playwright 测试环境注入)
 * Tech-Spec: 通知系统 Task 14
 * Story 3-8: Task 1.2 (添加 /privacy-records 路由)
 * Story 11.6: Task 1.2 (添加 /hub 路由)
 *
 * 配置路由和全局 Providers
 */

/* eslint-disable react-refresh/only-export-components */

import React from "react";
import ReactDOM from "react-dom/client";
import { BrowserRouter, Routes, Route, Navigate } from "react-router-dom";
import { Player, PrivacyRecords, Hub } from "./routes";
import { Settings } from "./routes/Settings";
import { GeneralSettings, DevelopmentSettings, PrivacySettings } from "./routes/settings";
import { ThemeProvider } from "./lib/theme-provider";
import { TooltipProvider } from "./components/ui/tooltip";
import { Toaster } from "./components/ui/sonner";
import { GlobalSearch } from "./components/search";
// TODO: 通知功能暂未开放
// import { NotificationBannerStack } from "./components/notifications";
import { UpdateNotificationBar } from "./components/notifications/UpdateNotificationBar";
import { useGlobalShortcut } from "./hooks";
import { useUpdateChecker } from "./hooks/useUpdateChecker";
// import { useNotificationInit } from "./hooks";
// Story 2-26: i18n 配置 (在导入 index.css 之前初始化)
import "./i18n";
// Monaco Editor 本地资源配置 (修复 AppImage 打包后编辑器无法加载问题)
import "./lib/monaco-setup";
import "./index.css";
// Story 9.2: IPC 适配器 (用于测试环境 Mock 注入)
import { setMockInvoke } from "./lib/ipc-adapter";

// Story 9.2: Playwright 测试环境检测与 Mock 注入
// 在 React 渲染前初始化，确保首次 IPC 调用使用 Mock
const initTestEnv = async (): Promise<void> => {
  const params = new URLSearchParams(window.location.search);
  if (params.has("playwright")) {
    // 设置全局测试标志
    window.__PLAYWRIGHT_TEST__ = true;
    console.log("[Mantra] Playwright 测试模式已启用");

    // 动态导入 Mock 处理器 (仅测试环境加载)
    try {
      const { mockInvoke } = await import("../e2e/fixtures/ipc-mock");
      setMockInvoke(mockInvoke);
      console.log("[Mantra] IPC Mock 已注入");
    } catch (err) {
      console.error("[Mantra] 加载 IPC Mock 失败:", err);
    }
  }
};

// Prevent flash of incorrect theme on initial load
// Default is dark theme when no stored preference
const initTheme = () => {
  const stored = localStorage.getItem("mantra-theme");
  const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;

  if (stored === "light" || (stored === "system" && !prefersDark)) {
    document.documentElement.classList.remove("dark");
  } else {
    // Default to dark: no stored value, stored === "dark", or system prefers dark
    document.documentElement.classList.add("dark");
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
  // TODO: 通知功能暂未开放
  // useNotificationInit();
  return <>{children}</>;
}

/**
 * UpdateCheckerProvider - 更新检查 Provider (Story 14.6)
 * 在根级别调用 useUpdateChecker，渲染 UpdateNotificationBar
 */
function UpdateCheckerProvider({ children }: { children: React.ReactNode }) {
  const {
    updateStatus,
    updateInfo,
    restartToUpdate,
    dismissUpdate,
  } = useUpdateChecker();

  return (
    <>
      <UpdateNotificationBar
        updateStatus={updateStatus}
        updateInfo={updateInfo}
        onRestart={restartToUpdate}
        onDismiss={dismissUpdate}
      />
      {children}
    </>
  );
}

/**
 * 应用启动函数
 * Story 9.2 Fix: 确保测试环境 Mock 注入完成后再渲染 React
 * 避免首次 IPC 调用在 Mock 注入前发生
 */
async function startApp() {
  // 等待测试环境初始化完成 (如果是测试模式)
  await initTestEnv();

  // 渲染 React 应用
  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ThemeProvider defaultTheme="dark">
      <TooltipProvider delayDuration={300}>
        <BrowserRouter>
          <GlobalShortcutProvider>
            {/* TODO: 通知功能暂未开放 */}
            {/* <NotificationBannerStack /> */}
            <UpdateCheckerProvider>
            <Routes>
              {/* Story 2.21: 首页即 Player (空状态) */}
              <Route path="/" element={<Player />} />
              {/* Player: 会话回放页 */}
              <Route path="/session/:sessionId" element={<Player />} />
              {/* 兼容旧 URL: /player/:sessionId → /session/:sessionId */}
              <Route path="/player/:sessionId" element={<Player />} />
              {/* Settings: 设置页面 - 嵌套路由 (Story 2-35) */}
              <Route path="/settings" element={<Settings />}>
                <Route index element={<Navigate to="general" replace />} />
                <Route path="general" element={<GeneralSettings />} />
                <Route path="development" element={<DevelopmentSettings />} />
                <Route path="privacy" element={<PrivacySettings />} />
              </Route>
              {/* PrivacyRecords: 隐私保护记录页面 (Story 3-8) */}
              <Route path="/privacy-records" element={<PrivacyRecords />} />
              {/* Hub: MCP Gateway 管理页面 (Story 11.6) */}
              <Route path="/hub" element={<Hub />} />
              {/* 默认重定向到首页 */}
              <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
            </UpdateCheckerProvider>
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
}

// 启动应用
startApp();
