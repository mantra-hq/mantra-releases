/**
 * Hub Page - Mantra Hub 页面
 * Story 11.6: Task 1 - Hub 路由和页面布局 (AC: #1)
 * Story 11.15: Task 7.4 - 集成接管状态卡片
 *
 * 提供 MCP Gateway 管理界面：
 * - Gateway 状态卡片
 * - 接管状态卡片（显示活跃接管和恢复按钮）
 * - MCP 服务列表
 * - 环境变量管理入口
 * - 项目关联配置
 */

import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { useCallback, useRef } from "react";
import { ArrowLeft, Radio, Key, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { GatewayStatusCard } from "@/components/hub/GatewayStatusCard";
import { TakeoverStatusCard } from "@/components/hub/TakeoverStatusCard";
import { McpServiceList, type McpServiceListRef } from "@/components/hub/McpServiceList";

export function Hub() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const serviceListRef = useRef<McpServiceListRef>(null);

  // Story 11.15: 恢复接管后刷新服务列表
  const handleTakeoverRestore = useCallback(() => {
    serviceListRef.current?.refresh();
  }, []);

  return (
    <div className="min-h-screen bg-background">
      {/* Header */}
      <header className="sticky top-0 z-50 border-b bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="container flex h-14 items-center px-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => navigate(-1)}
            aria-label={t("common.back")}
            data-testid="hub-back-button"
          >
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div className="flex items-center gap-2 ml-2">
            <Radio className="h-5 w-5 text-emerald-500" />
            <h1 className="text-lg font-semibold">{t("hub.title")}</h1>
          </div>
        </div>
      </header>

      {/* Content */}
      <main className="container px-4 py-6 max-w-5xl mx-auto">
        <div className="space-y-6">
          {/* Gateway 状态卡片 (AC: #1, #2) */}
          <section data-testid="hub-gateway-section">
            <GatewayStatusCard />
          </section>

          {/* 接管状态卡片 (Story 11.15: AC: #4, #5) */}
          <section data-testid="hub-takeover-section">
            <TakeoverStatusCard onRestore={handleTakeoverRestore} />
          </section>

          {/* MCP 服务列表 (AC: #1, #3, #4, #5) */}
          <section data-testid="hub-services-section">
            <McpServiceList ref={serviceListRef} />
          </section>

          {/* 环境变量管理入口 */}
          <section data-testid="hub-env-section">
            <button
              onClick={() => navigate('/settings')}
              className="w-full flex items-center justify-between rounded-lg border bg-card hover:bg-accent/50 p-4 transition-colors cursor-pointer"
              data-testid="hub-env-settings-link"
              aria-label={t("hub.envVariables.goToSettings")}
            >
              <div className="flex items-center gap-3">
                <div className="p-2 rounded-md bg-amber-500/10">
                  <Key className="h-5 w-5 text-amber-500" />
                </div>
                <div className="text-left">
                  <p className="text-sm font-medium">{t("hub.envVariables.title")}</p>
                  <p className="text-xs text-muted-foreground">{t("hub.envVariables.goToSettings")}</p>
                </div>
              </div>
              <ChevronRight className="h-5 w-5 text-muted-foreground" />
            </button>
          </section>
        </div>
      </main>
    </div>
  );
}

export default Hub;
