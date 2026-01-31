/**
 * Tool Policy 编辑器组件
 * Story 11.10: Project-Level Tool Management - Task 4.1, 4.2, 4.3, 4.4
 *
 * 用于编辑项目级 MCP 工具策略：
 * - Mode 选择器 (Allow All / Deny All / Custom)
 * - 工具列表展示 (名称、描述、Toggle)
 * - "Refresh Tools" 按钮强制刷新工具列表
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { RefreshCw, ChevronDown, Shield, ShieldOff, Settings, Loader2 } from "lucide-react";
import { feedback } from "@/lib/feedback";
import type { ToolPolicy, ToolPolicyMode, McpTool, ToolDiscoveryResult } from "@/types/mcp";
import { DEFAULT_TOOL_POLICY, isToolAllowed } from "@/types/mcp";

interface ToolPolicyEditorProps {
  projectId: string;
  serviceId: string;
  serviceName?: string;
  /** 初始策略（可选，如果未提供会从后端加载） */
  initialPolicy?: ToolPolicy;
  /** 策略变更回调 */
  onPolicyChange?: (policy: ToolPolicy) => void;
  /** 保存成功回调 */
  onSaved?: () => void;
  /** 是否可展开（用于卡片集成） */
  collapsible?: boolean;
  /** 默认展开状态 */
  defaultOpen?: boolean;
}

/**
 * 获取 Mode 显示标签
 */
function getModeLabel(mode: ToolPolicyMode, t: (key: string) => string): string {
  switch (mode) {
    case 'allow_all':
      return t('hub.toolPolicy.modeAllowAll');
    case 'deny_all':
      return t('hub.toolPolicy.modeDenyAll');
    case 'custom':
      return t('hub.toolPolicy.modeCustom');
    default:
      return mode;
  }
}

/**
 * 获取 Mode 图标
 */
function getModeIcon(mode: ToolPolicyMode) {
  switch (mode) {
    case 'allow_all':
      return <Shield className="h-4 w-4 text-green-500" />;
    case 'deny_all':
      return <ShieldOff className="h-4 w-4 text-red-500" />;
    case 'custom':
      return <Settings className="h-4 w-4 text-yellow-500" />;
    default:
      return null;
  }
}

export function ToolPolicyEditor({
  projectId,
  serviceId,
  serviceName = "",
  initialPolicy,
  onPolicyChange,
  onSaved,
  collapsible = false,
  defaultOpen = false,
}: ToolPolicyEditorProps) {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(defaultOpen);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  // 策略状态
  const [policy, setPolicy] = useState<ToolPolicy>(initialPolicy ?? DEFAULT_TOOL_POLICY);

  // 工具列表状态
  const [tools, setTools] = useState<McpTool[]>([]);
  const [fromCache, setFromCache] = useState(false);
  const [cachedAt, setCachedAt] = useState<string | undefined>();

  // 加载策略和工具列表
  const loadData = useCallback(async () => {
    setIsLoading(true);
    try {
      // 加载工具策略
      if (!initialPolicy) {
        const loadedPolicy = await invoke<ToolPolicy>('get_project_tool_policy', {
          projectId,
          serviceId,
        });
        setPolicy(loadedPolicy);
      }

      // 加载工具列表
      const result = await invoke<ToolDiscoveryResult>('fetch_service_tools', {
        serviceId,
      });
      setTools(result.tools);
      setFromCache(result.fromCache);
      setCachedAt(result.cachedAt);
    } catch (error) {
      console.error('[ToolPolicyEditor] Failed to load data:', error);
      feedback.error(t('hub.toolPolicy.loadError'), (error as Error).message);
    } finally {
      setIsLoading(false);
    }
  }, [projectId, serviceId, initialPolicy, t]);

  // 初始加载
  useEffect(() => {
    if (!collapsible || isOpen) {
      loadData();
    }
  }, [collapsible, isOpen, loadData]);

  // 刷新工具列表
  const handleRefreshTools = useCallback(async () => {
    setIsRefreshing(true);
    try {
      // 重新获取工具列表（会清除缓存并从 MCP 服务获取）
      const result = await invoke<ToolDiscoveryResult>('fetch_service_tools', {
        serviceId,
        forceRefresh: true,
      });
      setTools(result.tools);
      setFromCache(result.fromCache);
      setCachedAt(result.cachedAt);

      feedback.success(t('hub.toolPolicy.refreshSuccess'));
    } catch (error) {
      console.error('[ToolPolicyEditor] Failed to refresh tools:', error);
      feedback.error(t('hub.toolPolicy.refreshError'), (error as Error).message);
    } finally {
      setIsRefreshing(false);
    }
  }, [serviceId, t]);

  // 处理 Mode 变更
  const handleModeChange = useCallback(async (newMode: ToolPolicyMode) => {
    const newPolicy: ToolPolicy = {
      ...policy,
      mode: newMode,
    };
    setPolicy(newPolicy);
    onPolicyChange?.(newPolicy);

    // 保存到后端
    setIsSaving(true);
    try {
      await invoke('update_project_tool_policy', {
        projectId,
        serviceId,
        policy: newPolicy,
      });
      feedback.success(t('hub.toolPolicy.saveSuccess'));
      onSaved?.();
    } catch (error) {
      console.error('[ToolPolicyEditor] Failed to save policy:', error);
      feedback.error(t('hub.toolPolicy.saveError'), (error as Error).message);
    } finally {
      setIsSaving(false);
    }
  }, [policy, projectId, serviceId, onPolicyChange, onSaved, t]);

  // 切换工具的允许/禁止状态
  const handleToggleTool = useCallback(async (toolName: string, allowed: boolean) => {
    let newPolicy: ToolPolicy;

    if (allowed) {
      // 从 deniedTools 中移除，添加到 allowedTools
      newPolicy = {
        ...policy,
        allowedTools: policy.allowedTools.includes(toolName)
          ? policy.allowedTools
          : [...policy.allowedTools, toolName],
        deniedTools: policy.deniedTools.filter(t => t !== toolName),
      };
    } else {
      // 从 allowedTools 中移除，添加到 deniedTools
      newPolicy = {
        ...policy,
        allowedTools: policy.allowedTools.filter(t => t !== toolName),
        deniedTools: policy.deniedTools.includes(toolName)
          ? policy.deniedTools
          : [...policy.deniedTools, toolName],
      };
    }

    setPolicy(newPolicy);
    onPolicyChange?.(newPolicy);

    // 保存到后端
    setIsSaving(true);
    try {
      await invoke('update_project_tool_policy', {
        projectId,
        serviceId,
        policy: newPolicy,
      });
      onSaved?.();
    } catch (error) {
      console.error('[ToolPolicyEditor] Failed to save policy:', error);
      feedback.error(t('hub.toolPolicy.saveError'), (error as Error).message);
    } finally {
      setIsSaving(false);
    }
  }, [policy, projectId, serviceId, onPolicyChange, onSaved, t]);

  // 渲染内容
  const renderContent = () => {
    if (isLoading) {
      return (
        <div className="space-y-3">
          <Skeleton className="h-10 w-full" />
          <Skeleton className="h-20 w-full" />
          <Skeleton className="h-20 w-full" />
        </div>
      );
    }

    return (
      <div className="space-y-4">
        {/* Mode 选择器 */}
        <div className="space-y-2">
          <Label className="flex items-center gap-2">
            {getModeIcon(policy.mode)}
            {t('hub.toolPolicy.mode')}
          </Label>
          <Select
            value={policy.mode}
            onValueChange={(value) => handleModeChange(value as ToolPolicyMode)}
            disabled={isSaving}
          >
            <SelectTrigger className="w-full" data-testid="tool-policy-mode-select">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="allow_all">
                <div className="flex items-center gap-2">
                  <Shield className="h-4 w-4 text-green-500" />
                  {getModeLabel('allow_all', t)}
                </div>
              </SelectItem>
              <SelectItem value="deny_all">
                <div className="flex items-center gap-2">
                  <ShieldOff className="h-4 w-4 text-red-500" />
                  {getModeLabel('deny_all', t)}
                </div>
              </SelectItem>
              <SelectItem value="custom">
                <div className="flex items-center gap-2">
                  <Settings className="h-4 w-4 text-yellow-500" />
                  {getModeLabel('custom', t)}
                </div>
              </SelectItem>
            </SelectContent>
          </Select>
          <p className="text-xs text-muted-foreground">
            {t(`hub.toolPolicy.modeDescription.${policy.mode}`)}
          </p>
        </div>

        {/* 工具列表 */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Label>{t('hub.toolPolicy.tools')}</Label>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleRefreshTools}
              disabled={isRefreshing}
              className="h-7 text-xs"
              data-testid="refresh-tools-button"
            >
              {isRefreshing ? (
                <Loader2 className="h-3 w-3 mr-1 animate-spin" />
              ) : (
                <RefreshCw className="h-3 w-3 mr-1" />
              )}
              {t('hub.toolPolicy.refreshTools')}
            </Button>
          </div>

          {fromCache && cachedAt && (
            <p className="text-xs text-muted-foreground">
              {t('hub.toolPolicy.cachedAt', { time: new Date(cachedAt).toLocaleString() })}
            </p>
          )}

          {tools.length === 0 ? (
            <div className="text-center py-6 text-muted-foreground">
              <p className="text-sm">{t('hub.toolPolicy.noTools')}</p>
              <p className="text-xs mt-1">{t('hub.toolPolicy.noToolsHint')}</p>
            </div>
          ) : (
            <ScrollArea className="h-[200px] border rounded-md p-2">
              <div className="space-y-2">
                {tools.map((tool) => {
                  const allowed = isToolAllowed(policy, tool.name);
                  return (
                    <div
                      key={tool.name}
                      className="flex items-center justify-between p-2 rounded-md hover:bg-accent/50"
                      data-testid={`tool-item-${tool.name}`}
                    >
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="text-sm font-medium truncate">{tool.name}</span>
                          {!allowed && (
                            <Badge variant="outline" className="text-xs text-red-500 border-red-500/50">
                              {t('hub.toolPolicy.blocked')}
                            </Badge>
                          )}
                        </div>
                        {tool.description && (
                          <p className="text-xs text-muted-foreground truncate">
                            {tool.description}
                          </p>
                        )}
                      </div>
                      <Switch
                        checked={allowed}
                        onCheckedChange={(checked) => handleToggleTool(tool.name, checked)}
                        disabled={isSaving || policy.mode === 'deny_all'}
                        data-testid={`tool-toggle-${tool.name}`}
                      />
                    </div>
                  );
                })}
              </div>
            </ScrollArea>
          )}
        </div>

        {/* 保存指示器 */}
        {isSaving && (
          <div className="flex items-center justify-center text-xs text-muted-foreground">
            <Loader2 className="h-3 w-3 mr-1 animate-spin" />
            {t('common.saving')}
          </div>
        )}
      </div>
    );
  };

  // 可展开模式
  if (collapsible) {
    return (
      <Collapsible open={isOpen} onOpenChange={setIsOpen}>
        <CollapsibleTrigger asChild>
          <Button
            variant="ghost"
            size="sm"
            className="w-full justify-between h-9"
            data-testid="tool-policy-trigger"
          >
            <span className="flex items-center gap-2">
              {getModeIcon(policy.mode)}
              <span className="text-sm">{t('hub.toolPolicy.title')}</span>
            </span>
            <ChevronDown className={`h-4 w-4 transition-transform ${isOpen ? 'rotate-180' : ''}`} />
          </Button>
        </CollapsibleTrigger>
        <CollapsibleContent className="pt-3">
          {renderContent()}
        </CollapsibleContent>
      </Collapsible>
    );
  }

  // 卡片模式
  return (
    <Card>
      <CardHeader className="pb-3">
        <CardTitle className="flex items-center gap-2 text-base">
          {getModeIcon(policy.mode)}
          {t('hub.toolPolicy.title')}
        </CardTitle>
        <CardDescription>
          {t('hub.toolPolicy.description', { service: serviceName })}
        </CardDescription>
      </CardHeader>
      <CardContent>{renderContent()}</CardContent>
    </Card>
  );
}

export default ToolPolicyEditor;
