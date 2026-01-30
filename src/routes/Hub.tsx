/**
 * Hub Page - Mantra Hub 页面
 * Story 11.6: Task 1 - Hub 路由和页面布局 (AC: #1)
 *
 * 提供 MCP Gateway 管理界面：
 * - Gateway 状态卡片
 * - MCP 服务列表
 * - 环境变量管理入口
 * - 项目关联配置
 */

import { useNavigate } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { ArrowLeft, Radio } from "lucide-react";
import { Button } from "@/components/ui/button";
import { GatewayStatusCard } from "@/components/hub/GatewayStatusCard";
import { McpServiceList } from "@/components/hub/McpServiceList";
import { EnvVariableManager } from "@/components/hub/EnvVariableManager";

export function Hub() {
  const { t } = useTranslation();
  const navigate = useNavigate();

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

          {/* MCP 服务列表 (AC: #1, #3, #4, #5) */}
          <section data-testid="hub-services-section">
            <McpServiceList />
          </section>

          {/* 环境变量管理 (AC: #6) */}
          <section data-testid="hub-env-section">
            <EnvVariableManager />
          </section>
        </div>
      </main>
    </div>
  );
}

export default Hub;
