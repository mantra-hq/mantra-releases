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
  const [error, setError] = useState<string | null>(null);

  // Note: gatewayRunning prop is no longer used as we call MCP services directly via Tauri
  void gatewayRunning;

  // 加载服务的工具和资源（通过 Tauri 直接调用 MCP 服务）
  const loadServiceCapabilities = useCallback(async () => {
    if (!service) return;

    setIsLoading(true);
    setError(null);

    try {
      // 直接通过 Tauri 命令获取 MCP 服务能力
      const capabilities = await invoke<{
        tools: McpTool[];
        resources: McpResource[];
      }>("mcp_get_service_capabilities", { serviceId: service.id });

      setTools(capabilities.tools || []);
      setResources(capabilities.resources || []);

      // 记录日志
      const logEntry: RpcLogEntry = {
        id: crypto.randomUUID(),
        timestamp: new Date().toISOString(),
        method: "mcp_get_service_capabilities",
        request: { serviceId: service.id },
        response: capabilities,
        error: null,
        duration: 0,
      };
      setLogs((prev) => [logEntry, ...prev]);
    } catch (err) {
      console.error("[InspectorDrawer] Failed to load capabilities:", err);
      const errorMessage = (err as Error).message || String(err);
      setError(errorMessage);

      // 记录错误日志
      const logEntry: RpcLogEntry = {
        id: crypto.randomUUID(),
        timestamp: new Date().toISOString(),
        method: "mcp_get_service_capabilities",
        request: { serviceId: service.id },
        response: null,
        error: { code: -1, message: errorMessage },
        duration: 0,
      };
      setLogs((prev) => [logEntry, ...prev]);
    } finally {
      setIsLoading(false);
    }
  }, [service]);

  // 执行工具调用（通过 Tauri 直接调用）
  const handleToolCall = useCallback(
    async (tool: McpTool, args: Record<string, unknown>) => {
      if (!service) {
        feedback.error(t("hub.inspector.noServiceSelected"));
        return null;
      }

      const startTime = Date.now();
      const logEntry: RpcLogEntry = {
        id: crypto.randomUUID(),
        timestamp: new Date().toISOString(),
        method: "tools/call",
        request: { name: tool.name, arguments: args },
        response: null,
        error: null,
        duration: 0,
      };
      setLogs((prev) => [logEntry, ...prev]);

      try {
        const result = await invoke<unknown>("mcp_call_tool", {
          serviceId: service.id,
          toolName: tool.name,
          arguments: args,
        });

        const duration = Date.now() - startTime;
        setLogs((prev) =>
          prev.map((log) =>
            log.id === logEntry.id
              ? { ...log, response: { result }, duration }
              : log
          )
        );

        return { result };
      } catch (err) {
        const duration = Date.now() - startTime;
        const errorMessage = (err as Error).message || String(err);
        
        setLogs((prev) =>
          prev.map((log) =>
            log.id === logEntry.id
              ? { ...log, error: { code: -1, message: errorMessage }, duration }
              : log
          )
        );

        feedback.error(t("hub.inspector.toolCallError"), errorMessage);
        return null;
      }
    },
    [service, t]
  );

  // 读取资源（通过 Tauri 直接调用）
  const handleResourceRead = useCallback(
    async (resource: McpResource) => {
      if (!service) {
        feedback.error(t("hub.inspector.noServiceSelected"));
        return null;
      }

      const startTime = Date.now();
      const logEntry: RpcLogEntry = {
        id: crypto.randomUUID(),
        timestamp: new Date().toISOString(),
        method: "resources/read",
        request: { uri: resource.uri },
        response: null,
        error: null,
        duration: 0,
      };
      setLogs((prev) => [logEntry, ...prev]);

      try {
        const result = await invoke<unknown>("mcp_read_resource", {
          serviceId: service.id,
          uri: resource.uri,
        });

        const duration = Date.now() - startTime;
        setLogs((prev) =>
          prev.map((log) =>
            log.id === logEntry.id
              ? { ...log, response: { result }, duration }
              : log
          )
        );

        return { result };
      } catch (err) {
        const duration = Date.now() - startTime;
        const errorMessage = (err as Error).message || String(err);
        
        setLogs((prev) =>
          prev.map((log) =>
            log.id === logEntry.id
              ? { ...log, error: { code: -1, message: errorMessage }, duration }
              : log
          )
        );

        feedback.error(t("hub.inspector.resourceReadError"), errorMessage);
        return null;
      }
    },
    [service, t]
  );

  // 清空日志
  const handleClearLogs = useCallback(() => {
    setLogs([]);
  }, []);

  // 打开时加载数据（现在直接通过 Tauri 调用，不需要 Gateway）
  useEffect(() => {
    if (open && service) {
      loadServiceCapabilities();
    } else if (!open) {
      // 关闭时重置状态
      setTools([]);
      setResources([]);
      setSelectedTool(null);
      setSelectedResource(null);
      setError(null);
    }
  }, [open, service, loadServiceCapabilities]);

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
            ? "!w-full !max-w-none"
            : "!w-[90vw] !max-w-[1400px] p-6"
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
          {isLoading ? (
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
            <ResizablePanelGroup 
                id="inspector-horizontal" 
                orientation="horizontal" 
                disabled={false}
                className="h-full w-full"
              >
              {/* 左侧 - 工具/资源列表 */}
              <ResizablePanel 
                id="inspector-left"
                defaultSize={35} 
                minSize={15}
              >
                <div className="h-full w-full">
                  <ToolExplorer
                    tools={tools}
                    resources={resources}
                    selectedTool={selectedTool}
                    selectedResource={selectedResource}
                    onSelectTool={handleSelectTool}
                    onSelectResource={handleSelectResource}
                  />
                </div>
              </ResizablePanel>

              <ResizableHandle withHandle orientation="horizontal" />

              {/* 右侧 - 交互面板和日志 */}
              <ResizablePanel id="inspector-right" defaultSize={65} minSize={15}>
                <ResizablePanelGroup id="inspector-vertical" orientation="vertical" className="h-full">
                  {/* 上部 - 交互面板 */}
                  <ResizablePanel id="inspector-top" defaultSize={60} minSize={20}>
                    <ToolTester
                      selectedTool={selectedTool}
                      selectedResource={selectedResource}
                      onExecute={handleToolCall}
                      logs={logs}
                    />
                  </ResizablePanel>

                  <ResizableHandle withHandle orientation="vertical" />

                  {/* 下部 - 日志面板 */}
                  <ResizablePanel id="inspector-bottom" defaultSize={40} minSize={15}>
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
