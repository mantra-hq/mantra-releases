/**
 * MCP Inspector 抽屉组件
 * Story 11.11: Task 1 - InspectorDrawer (AC: 1, 2)
 *
 * 提供 MCP 服务调试功能：
 * - 三栏布局（工具树、交互面板、日志面板）
 * - 全屏模式切换
 * - JSON-RPC 请求/响应日志记录
 */

import { useState, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
  SheetDescription,
} from "@/components/ui/sheet";
import {
  ResizablePanelGroup,
  ResizablePanel,
  ResizableHandle,
} from "@/components/ui/resizable";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import {
  Maximize2,
  Minimize2,
  Bug,
  Loader2,
  AlertCircle,
} from "lucide-react";
import { invoke } from "@/lib/ipc-adapter";
import { feedback } from "@/lib/feedback";
import type { McpService } from "../McpServiceList";
import type { McpTool } from "@/types/mcp";
import { ToolExplorer } from "./ToolExplorer";
import { ToolTester } from "./ToolTester";
import { RpcLogViewer } from "./RpcLogViewer";
import type { RpcLogEntry } from "./RpcLogViewer";

/**
 * MCP Resource 定义
 */
export interface McpResource {
  uri: string;
  name: string;
  description?: string;
  mimeType?: string;
}

/**
 * Inspector 抽屉属性
 */
export interface InspectorDrawerProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  service: McpService | null;
  gatewayRunning: boolean;
}

/**
 * Gateway 状态类型
 */
interface GatewayStatus {
  running: boolean;
  port: number | null;
  auth_token: string;
}

/**
 * Inspector 抽屉组件
 */
export function InspectorDrawer({
  open,
  onOpenChange,
  service,
  gatewayRunning,
}: InspectorDrawerProps) {
  const { t } = useTranslation();

  // 状态管理
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [tools, setTools] = useState<McpTool[]>([]);
  const [resources, setResources] = useState<McpResource[]>([]);
  const [selectedTool, setSelectedTool] = useState<McpTool | null>(null);
  const [selectedResource, setSelectedResource] = useState<McpResource | null>(null);
  const [logs, setLogs] = useState<RpcLogEntry[]>([]);
  const [gatewayInfo, setGatewayInfo] = useState<GatewayStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  // 加载 Gateway 状态
  const loadGatewayInfo = useCallback(async () => {
    try {
      const status = await invoke<GatewayStatus>("get_gateway_status");
      setGatewayInfo(status);
      return status;
    } catch (err) {
      console.error("[InspectorDrawer] Failed to get gateway status:", err);
      return null;
    }
  }, []);

  // 加载服务的工具和资源
  const loadServiceCapabilities = useCallback(async () => {
    if (!service || !gatewayRunning) return;

    setIsLoading(true);
    setError(null);

    try {
      const status = await loadGatewayInfo();
      if (!status?.running || !status.port) {
        setError(t("hub.inspector.gatewayNotRunning"));
        return;
      }

      // 构建 Gateway URL
      const baseUrl = `http://127.0.0.1:${status.port}`;

      // 发送 tools/list 请求
      const toolsResponse = await sendJsonRpcRequest(
        baseUrl,
        status.auth_token,
        "tools/list",
        {},
        service.name
      );

      if (toolsResponse.result?.tools) {
        setTools(toolsResponse.result.tools);
      }

      // 发送 resources/list 请求
      const resourcesResponse = await sendJsonRpcRequest(
        baseUrl,
        status.auth_token,
        "resources/list",
        {},
        service.name
      );

      if (resourcesResponse.result?.resources) {
        setResources(resourcesResponse.result.resources);
      }
    } catch (err) {
      console.error("[InspectorDrawer] Failed to load capabilities:", err);
      setError((err as Error).message);
    } finally {
      setIsLoading(false);
    }
  }, [service, gatewayRunning, loadGatewayInfo, t]);

  // 发送 JSON-RPC 请求并记录日志
  const sendJsonRpcRequest = useCallback(
    async (
      baseUrl: string,
      token: string,
      method: string,
      params: Record<string, unknown>,
      serviceName: string
    ) => {
      const requestId = Date.now();
      const request = {
        jsonrpc: "2.0" as const,
        method,
        params: {
          ...params,
          _meta: { serviceName },
        },
        id: requestId,
      };

      const startTime = Date.now();

      // 记录请求日志
      const logEntry: RpcLogEntry = {
        id: crypto.randomUUID(),
        timestamp: new Date().toISOString(),
        method,
        request,
        response: null,
        error: null,
        duration: 0,
      };

      setLogs((prev) => [logEntry, ...prev]);

      try {
        const response = await fetch(`${baseUrl}/message`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            Authorization: `Bearer ${token}`,
          },
          body: JSON.stringify(request),
        });

        const duration = Date.now() - startTime;
        const data = await response.json();

        // 更新日志
        setLogs((prev) =>
          prev.map((log) =>
            log.id === logEntry.id
              ? {
                  ...log,
                  response: data,
                  error: data.error || null,
                  duration,
                }
              : log
          )
        );

        return data;
      } catch (err) {
        const duration = Date.now() - startTime;
        const errorMessage = (err as Error).message;

        // 更新日志
        setLogs((prev) =>
          prev.map((log) =>
            log.id === logEntry.id
              ? {
                  ...log,
                  error: { code: -1, message: errorMessage },
                  duration,
                }
              : log
          )
        );

        throw err;
      }
    },
    []
  );

  // 执行工具调用
  const handleToolCall = useCallback(
    async (tool: McpTool, args: Record<string, unknown>) => {
      if (!gatewayInfo?.running || !gatewayInfo.port || !service) {
        feedback.error(t("hub.inspector.gatewayNotRunning"));
        return null;
      }

      const baseUrl = `http://127.0.0.1:${gatewayInfo.port}`;

      try {
        const response = await sendJsonRpcRequest(
          baseUrl,
          gatewayInfo.auth_token,
          "tools/call",
          { name: tool.name, arguments: args },
          service.name
        );

        if (response.error) {
          feedback.error(t("hub.inspector.toolCallError"), response.error.message);
        }

        return response;
      } catch (err) {
        feedback.error(t("hub.inspector.toolCallError"), (err as Error).message);
        return null;
      }
    },
    [gatewayInfo, service, sendJsonRpcRequest, t]
  );

  // 读取资源
  const handleResourceRead = useCallback(
    async (resource: McpResource) => {
      if (!gatewayInfo?.running || !gatewayInfo.port || !service) {
        feedback.error(t("hub.inspector.gatewayNotRunning"));
        return null;
      }

      const baseUrl = `http://127.0.0.1:${gatewayInfo.port}`;

      try {
        const response = await sendJsonRpcRequest(
          baseUrl,
          gatewayInfo.auth_token,
          "resources/read",
          { uri: resource.uri },
          service.name
        );

        if (response.error) {
          feedback.error(t("hub.inspector.resourceReadError"), response.error.message);
        }

        return response;
      } catch (err) {
        feedback.error(t("hub.inspector.resourceReadError"), (err as Error).message);
        return null;
      }
    },
    [gatewayInfo, service, sendJsonRpcRequest, t]
  );

  // 清空日志
  const handleClearLogs = useCallback(() => {
    setLogs([]);
  }, []);

  // 打开时加载数据
  useEffect(() => {
    if (open && service && gatewayRunning) {
      loadServiceCapabilities();
    } else if (!open) {
      // 关闭时重置状态
      setTools([]);
      setResources([]);
      setSelectedTool(null);
      setSelectedResource(null);
      setError(null);
    }
  }, [open, service, gatewayRunning, loadServiceCapabilities]);

  // 切换全屏
  const toggleFullscreen = useCallback(() => {
    setIsFullscreen((prev) => !prev);
  }, []);

  // 选择工具
  const handleSelectTool = useCallback((tool: McpTool) => {
    setSelectedTool(tool);
    setSelectedResource(null);
  }, []);

  // 选择资源
  const handleSelectResource = useCallback(async (resource: McpResource) => {
    setSelectedResource(resource);
    setSelectedTool(null);
    // 自动读取资源内容
    await handleResourceRead(resource);
  }, [handleResourceRead]);

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent
        side="right"
        className={
          isFullscreen
            ? "w-full max-w-none sm:max-w-none"
            : "w-[90vw] max-w-[1200px] sm:max-w-[1200px]"
        }
      >
        <SheetHeader className="flex-none">
          <div className="flex items-center justify-between pr-8">
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-md bg-amber-500/10">
                <Bug className="h-5 w-5 text-amber-500" />
              </div>
              <div>
                <SheetTitle className="flex items-center gap-2">
                  {t("hub.inspector.title")}
                  {service && (
                    <Badge variant="outline" className="font-mono text-xs">
                      {service.name}
                    </Badge>
                  )}
                </SheetTitle>
                <SheetDescription>
                  {t("hub.inspector.description")}
                </SheetDescription>
              </div>
            </div>
            <Button
              variant="ghost"
              size="icon"
              onClick={toggleFullscreen}
              title={
                isFullscreen
                  ? t("hub.inspector.exitFullscreen")
                  : t("hub.inspector.enterFullscreen")
              }
            >
              {isFullscreen ? (
                <Minimize2 className="h-4 w-4" />
              ) : (
                <Maximize2 className="h-4 w-4" />
              )}
            </Button>
          </div>
        </SheetHeader>

        <Separator className="my-4" />

        {/* 主内容区 - 三栏布局 */}
        <div className="flex-1 min-h-0 h-[calc(100vh-140px)]">
          {!gatewayRunning ? (
            <div className="flex flex-col items-center justify-center h-full text-muted-foreground">
              <AlertCircle className="h-12 w-12 mb-4 opacity-50" />
              <p className="text-lg font-medium">{t("hub.inspector.gatewayNotRunning")}</p>
              <p className="text-sm mt-2">{t("hub.inspector.startGatewayHint")}</p>
            </div>
          ) : isLoading ? (
            <div className="flex flex-col items-center justify-center h-full">
              <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
              <p className="text-sm text-muted-foreground mt-4">
                {t("hub.inspector.loading")}
              </p>
            </div>
          ) : error ? (
            <div className="flex flex-col items-center justify-center h-full text-muted-foreground">
              <AlertCircle className="h-12 w-12 mb-4 text-destructive opacity-50" />
              <p className="text-lg font-medium text-destructive">{t("common.error")}</p>
              <p className="text-sm mt-2">{error}</p>
              <Button
                variant="outline"
                size="sm"
                className="mt-4"
                onClick={loadServiceCapabilities}
              >
                {t("common.retry")}
              </Button>
            </div>
          ) : (
            <ResizablePanelGroup orientation="horizontal" className="h-full rounded-lg border">
              {/* 左侧 - 工具/资源列表 */}
              <ResizablePanel defaultSize={25} minSize={15} maxSize={40}>
                <ToolExplorer
                  tools={tools}
                  resources={resources}
                  selectedTool={selectedTool}
                  selectedResource={selectedResource}
                  onSelectTool={handleSelectTool}
                  onSelectResource={handleSelectResource}
                />
              </ResizablePanel>

              <ResizableHandle className="bg-border" />

              {/* 右侧 - 交互面板和日志 */}
              <ResizablePanel defaultSize={75} minSize={50}>
                <ResizablePanelGroup orientation="vertical" className="h-full">
                  {/* 上部 - 交互面板 */}
                  <ResizablePanel defaultSize={60} minSize={30}>
                    <ToolTester
                      selectedTool={selectedTool}
                      selectedResource={selectedResource}
                      onExecute={handleToolCall}
                      logs={logs}
                    />
                  </ResizablePanel>

                  <ResizableHandle className="bg-border" />

                  {/* 下部 - 日志面板 */}
                  <ResizablePanel defaultSize={40} minSize={20}>
                    <RpcLogViewer logs={logs} onClear={handleClearLogs} />
                  </ResizablePanel>
                </ResizablePanelGroup>
              </ResizablePanel>
            </ResizablePanelGroup>
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}

export default InspectorDrawer;
