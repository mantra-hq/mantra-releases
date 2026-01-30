/**
 * Gateway 状态卡片组件
 * Story 11.6: Task 2 - Gateway 状态卡片 (AC: #1, #2)
 *
 * 显示 Gateway Server 状态：
 * - 运行状态（运行中/已停止）
 * - 端口号
 * - 连接数
 * - 启动/停止按钮
 * - 连接 URL 和 Token 复制
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  Radio,
  Play,
  Square,
  RotateCcw,
  Copy,
  Check,
  RefreshCw,
  Link2,
  Key,
  Loader2,
} from "lucide-react";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { feedback } from "@/lib/feedback";

/**
 * Gateway 状态响应类型
 */
interface GatewayStatus {
  running: boolean;
  port: number | null;
  auth_token: string;
  active_connections: number;
  total_connections: number;
  total_requests: number;
}

export function GatewayStatusCard() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<GatewayStatus | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [isStarting, setIsStarting] = useState(false);
  const [isStopping, setIsStopping] = useState(false);
  const [isRegenerating, setIsRegenerating] = useState(false);
  const [copied, setCopied] = useState<"url" | "token" | null>(null);

  // 加载 Gateway 状态
  const loadStatus = useCallback(async () => {
    try {
      const result = await invoke<GatewayStatus>("get_gateway_status");
      setStatus(result);
    } catch (error) {
      console.error("[GatewayStatusCard] Failed to load status:", error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 启动 Gateway
  const handleStart = useCallback(async () => {
    setIsStarting(true);
    try {
      const result = await invoke<GatewayStatus>("start_gateway");
      setStatus(result);
      feedback.success(t("hub.gateway.startSuccess"));
    } catch (error) {
      console.error("[GatewayStatusCard] Failed to start:", error);
      feedback.error(t("hub.gateway.start"), (error as Error).message);
    } finally {
      setIsStarting(false);
    }
  }, [t]);

  // 停止 Gateway
  const handleStop = useCallback(async () => {
    setIsStopping(true);
    try {
      const result = await invoke<GatewayStatus>("stop_gateway");
      setStatus(result);
      feedback.success(t("hub.gateway.stopSuccess"));
    } catch (error) {
      console.error("[GatewayStatusCard] Failed to stop:", error);
      feedback.error(t("hub.gateway.stop"), (error as Error).message);
    } finally {
      setIsStopping(false);
    }
  }, [t]);

  // 重新生成 Token
  const handleRegenerateToken = useCallback(async () => {
    setIsRegenerating(true);
    try {
      const newToken = await invoke<string>("regenerate_gateway_token");
      setStatus((prev) =>
        prev ? { ...prev, auth_token: newToken } : prev
      );
      feedback.success(t("hub.gateway.tokenRegenerateSuccess"));
    } catch (error) {
      console.error("[GatewayStatusCard] Failed to regenerate token:", error);
      feedback.error(t("hub.gateway.regenerateToken"), (error as Error).message);
    } finally {
      setIsRegenerating(false);
    }
  }, [t]);

  // 复制到剪贴板
  const handleCopy = useCallback(async (type: "url" | "token") => {
    if (!status) return;

    const text =
      type === "url"
        ? `http://127.0.0.1:${status.port}/sse?token=${status.auth_token}`
        : status.auth_token;

    try {
      await navigator.clipboard.writeText(text);
      setCopied(type);
      feedback.copied(t(type === "url" ? "hub.gateway.connectionUrl" : "hub.gateway.token"));
      setTimeout(() => setCopied(null), 2000);
    } catch (error) {
      console.error("[GatewayStatusCard] Failed to copy:", error);
      feedback.error(t("common.copy"), (error as Error).message);
    }
  }, [status, t]);

  // 初始加载 + 定时刷新
  useEffect(() => {
    loadStatus();
    const interval = setInterval(loadStatus, 5000);
    return () => clearInterval(interval);
  }, [loadStatus]);

  // 掩码显示 Token
  const maskToken = (token: string) => {
    if (token.length <= 8) return "****";
    return `${token.slice(0, 4)}****${token.slice(-4)}`;
  };

  return (
    <Card data-testid="gateway-status-card">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className={`p-2 rounded-md ${status?.running ? "bg-emerald-500/10" : "bg-zinc-500/10"}`}>
              <Radio className={`h-5 w-5 ${status?.running ? "text-emerald-500" : "text-zinc-500"}`} />
            </div>
            <div>
              <CardTitle className="text-base">{t("hub.gateway.title")}</CardTitle>
              <CardDescription>{t("hub.gateway.description")}</CardDescription>
            </div>
          </div>
          <div className="flex items-center gap-2">
            {/* 状态 Badge */}
            <Badge
              variant={status?.running ? "default" : "secondary"}
              className={status?.running ? "bg-emerald-500/10 text-emerald-500 border-emerald-500/20" : ""}
              data-testid="gateway-status-badge"
            >
              {status?.running ? t("hub.gateway.running") : t("hub.gateway.stopped")}
            </Badge>
            {/* 刷新按钮 */}
            <Button
              variant="ghost"
              size="icon"
              onClick={loadStatus}
              disabled={isLoading}
              className="h-8 w-8"
              title={t("common.refresh")}
            >
              <RefreshCw className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* 统计信息 */}
        <div className="grid grid-cols-3 gap-4 text-sm">
          <div className="space-y-1">
            <p className="text-muted-foreground">{t("hub.gateway.port")}</p>
            <p className="font-mono font-medium" data-testid="gateway-port">
              {status?.port ?? "-"}
            </p>
          </div>
          <div className="space-y-1">
            <p className="text-muted-foreground">{t("hub.gateway.connections")}</p>
            <p className="font-mono font-medium" data-testid="gateway-connections">
              {status?.active_connections ?? 0}
            </p>
          </div>
          <div className="space-y-1">
            <p className="text-muted-foreground">{t("hub.gateway.requests")}</p>
            <p className="font-mono font-medium" data-testid="gateway-requests">
              {status?.total_requests ?? 0}
            </p>
          </div>
        </div>

        <Separator />

        {/* 连接信息 (仅运行时显示) */}
        {status?.running && (
          <div className="space-y-3">
            {/* 连接 URL */}
            <div className="space-y-1.5">
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Link2 className="h-3.5 w-3.5" />
                <span>{t("hub.gateway.connectionUrl")}</span>
              </div>
              <div className="flex items-center gap-2">
                <code className="flex-1 text-xs bg-muted px-2 py-1.5 rounded-md font-mono truncate">
                  http://127.0.0.1:{status.port}/sse?token={maskToken(status.auth_token)}
                </code>
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleCopy("url")}
                        className="shrink-0"
                        data-testid="copy-url-button"
                      >
                        {copied === "url" ? (
                          <Check className="h-4 w-4 text-emerald-500" />
                        ) : (
                          <Copy className="h-4 w-4" />
                        )}
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent side="top">
                      <p>{t("hub.gateway.copyUrl")}</p>
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              </div>
            </div>

            {/* Token */}
            <div className="space-y-1.5">
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Key className="h-3.5 w-3.5" />
                <span>{t("hub.gateway.token")}</span>
              </div>
              <div className="flex items-center gap-2">
                <code className="flex-1 text-xs bg-muted px-2 py-1.5 rounded-md font-mono">
                  {maskToken(status.auth_token)}
                </code>
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleCopy("token")}
                        className="shrink-0"
                        data-testid="copy-token-button"
                      >
                        {copied === "token" ? (
                          <Check className="h-4 w-4 text-emerald-500" />
                        ) : (
                          <Copy className="h-4 w-4" />
                        )}
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent side="top">
                      <p>{t("hub.gateway.copyToken")}</p>
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
                <TooltipProvider>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={handleRegenerateToken}
                        disabled={isRegenerating}
                        className="shrink-0"
                        data-testid="regenerate-token-button"
                      >
                        {isRegenerating ? (
                          <Loader2 className="h-4 w-4 animate-spin" />
                        ) : (
                          <RotateCcw className="h-4 w-4" />
                        )}
                      </Button>
                    </TooltipTrigger>
                    <TooltipContent side="top">
                      <p>{t("hub.gateway.regenerateToken")}</p>
                    </TooltipContent>
                  </Tooltip>
                </TooltipProvider>
              </div>
            </div>
          </div>
        )}

        <Separator />

        {/* 操作按钮 */}
        <div className="flex justify-end gap-2">
          {status?.running ? (
            <Button
              variant="destructive"
              onClick={handleStop}
              disabled={isStopping}
              data-testid="gateway-stop-button"
            >
              {isStopping ? (
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              ) : (
                <Square className="h-4 w-4 mr-2" />
              )}
              {t("hub.gateway.stop")}
            </Button>
          ) : (
            <Button
              onClick={handleStart}
              disabled={isStarting}
              data-testid="gateway-start-button"
            >
              {isStarting ? (
                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
              ) : (
                <Play className="h-4 w-4 mr-2" />
              )}
              {t("hub.gateway.start")}
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

export default GatewayStatusCard;
