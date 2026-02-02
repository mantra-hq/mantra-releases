/**
 * Tool Policy 编辑器组件
 * Story 11.10: Project-Level Tool Management - Task 4.1, 4.2, 4.3, 4.4
 * Story 11.9 Phase 2: Task 10 - 支持全局模式（服务级默认策略）
 *
 * 用于编辑 MCP 工具策略：
 * - 项目级模式：当 projectId 有值时，编辑项目级策略覆盖
 * - 全局模式：当 projectId 为空时，编辑服务级默认策略
 * - Checkbox 列表选择工具
 * - 全选/全不选 按钮
 * - 手动保存按钮
 * - Mode 从选择状态自动推导
 */

import { useState, useEffect, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";
import { Badge } from "@/components/ui/badge";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Skeleton } from "@/components/ui/skeleton";
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
import { RefreshCw, ChevronDown, Shield, ShieldOff, Settings, Loader2, Save, CheckSquare, Square, Info, Globe, FolderOpen } from "lucide-react";
import { feedback } from "@/lib/feedback";
import type { ToolPolicy, ToolPolicyMode, McpTool, ToolDiscoveryResult } from "@/types/mcp";

interface ToolPolicyEditorProps {
  /** 项目 ID（可选，不提供则为全局模式） */
  projectId?: string;
  serviceId: string;
  serviceName?: string;
  /** 项目名称（用于上下文提示） */
  projectName?: string;
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
  /** 嵌入模式：只渲染内容，不渲染 Card 包装（用于 Sheet 内嵌） */
  embedded?: boolean;
}

/**
 * Story 12.5: 上下文提示组件
 * 区分全局模式和项目模式的策略编辑范围
 */
function PolicyContextHint({
  isGlobalMode,
  projectName,
  globalDefaultMode,
  isInherited,
  t,
}: {
  isGlobalMode: boolean;
  projectName?: string;
  globalDefaultMode?: ToolPolicyMode;
  isInherited?: boolean;
  t: (key: string, fallback?: string, opts?: Record<string, unknown>) => string;
}) {
  if (isGlobalMode) {
    return (
      <div className="flex items-center gap-2 p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg">
        <Globe className="h-4 w-4 text-blue-500 shrink-0" />
        <span className="text-sm text-blue-200">
          {t("hub.toolPolicy.globalHint", "Editing the service's default policy. This will affect all projects without custom configuration.")}
        </span>
      </div>
    );
  }

  // AC3: 项目模式下显示继承状态
  const modeLabel = globalDefaultMode === 'allow_all'
    ? t('hub.toolPolicy.modeAllowAll')
    : globalDefaultMode === 'deny_all'
    ? t('hub.toolPolicy.modeDenyAll')
    : t('hub.toolPolicy.modeCustom');

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 p-3 bg-amber-500/10 border border-amber-500/20 rounded-lg">
        <FolderOpen className="h-4 w-4 text-amber-500 shrink-0" />
        <span className="text-sm text-amber-200">
          {t("hub.toolPolicy.projectHint", "Customizing policy for project {{project}}. This will override the global default.", { project: projectName || "this project" })}
        </span>
      </div>
      {/* AC3: 继承状态指示 */}
      {isInherited && globalDefaultMode && (
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/50 rounded-lg">
          <Info className="h-4 w-4 text-muted-foreground shrink-0" />
          <span className="text-xs text-muted-foreground">
            {t("hub.toolPolicy.inheritingFrom", "Inheriting from global default: {{mode}}", { mode: modeLabel })}
          </span>
          <Badge variant="outline" className="text-[10px] px-1.5 py-0 h-5 ml-auto">
            {t("hub.toolPolicy.isInherited", "Inherited")}
          </Badge>
        </div>
      )}
    </div>
  );
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

/**
 * 从选中的工具列表推导 Mode
 */
function deriveMode(selectedTools: Set<string>, totalTools: number): ToolPolicyMode {
  if (selectedTools.size === 0) {
    return 'deny_all';
  }
  if (selectedTools.size === totalTools) {
    return 'allow_all';
  }
  return 'custom';
}

export function ToolPolicyEditor({
  projectId,
  serviceId,
  serviceName = "",
  projectName,
  initialPolicy,
  onPolicyChange,
  onSaved,
  collapsible = false,
  defaultOpen = false,
  embedded = false,
}: ToolPolicyEditorProps) {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(defaultOpen);
  const [isLoading, setIsLoading] = useState(true);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  // 原始策略（用于检测是否有变更）
  const [originalPolicy, setOriginalPolicy] = useState<ToolPolicy | null>(null);

  // 选中的工具集合
  const [selectedTools, setSelectedTools] = useState<Set<string>>(new Set());

  // 工具列表状态
  const [tools, setTools] = useState<McpTool[]>([]);
  const [fromCache, setFromCache] = useState(false);
  const [cachedAt, setCachedAt] = useState<string | undefined>();

  // 是否为全局模式（无 projectId）
  const isGlobalMode = !projectId;

  // Story 12.5 AC3: 全局默认策略（用于项目模式下显示继承状态）
  const [globalDefaultPolicy, setGlobalDefaultPolicy] = useState<ToolPolicy | null>(null);
  const [isInherited, setIsInherited] = useState(false);

  // 从策略初始化选中状态
  const initializeSelection = useCallback((policy: ToolPolicy, toolList: McpTool[]) => {
    const selected = new Set<string>();

    if (policy.mode === 'allow_all') {
      // allow_all: 全部选中，除了 deniedTools
      for (const tool of toolList) {
        if (!policy.deniedTools.includes(tool.name)) {
          selected.add(tool.name);
        }
      }
    } else if (policy.mode === 'deny_all') {
      // deny_all: 全部不选，除了 allowedTools
      for (const toolName of policy.allowedTools) {
        selected.add(toolName);
      }
    } else {
      // custom: 只选中 allowedTools
      for (const toolName of policy.allowedTools) {
        selected.add(toolName);
      }
    }

    setSelectedTools(selected);
  }, []);

  /**
   * 比较两个策略是否相同（用于检测是否继承自全局）
   */
  const arePoliciesEqual = useCallback((p1: ToolPolicy, p2: ToolPolicy): boolean => {
    if (p1.mode !== p2.mode) return false;
    if (p1.mode === 'custom') {
      const allowed1 = new Set(p1.allowedTools);
      const allowed2 = new Set(p2.allowedTools);
      if (allowed1.size !== allowed2.size) return false;
      for (const tool of allowed1) {
        if (!allowed2.has(tool)) return false;
      }
    }
    return true;
  }, []);

  // 加载策略和工具列表
  const loadData = useCallback(async () => {
    setIsLoading(true);
    try {
      // 加载工具列表
      const result = await invoke<ToolDiscoveryResult>('fetch_service_tools', {
        serviceId,
      });
      setTools(result.tools);
      setFromCache(result.fromCache);
      setCachedAt(result.cachedAt);

      // Story 12.5 AC3: 始终加载全局默认策略（用于项目模式下对比）
      const globalPolicy = await invoke<ToolPolicy>('get_service_default_policy', {
        serviceId,
      });
      setGlobalDefaultPolicy(globalPolicy);

      // 加载工具策略
      let loadedPolicy: ToolPolicy;
      if (initialPolicy) {
        loadedPolicy = initialPolicy;
      } else if (isGlobalMode) {
        // 全局模式：使用已加载的全局策略
        loadedPolicy = globalPolicy;
      } else {
        // 项目模式：加载项目级策略
        loadedPolicy = await invoke<ToolPolicy>('get_project_tool_policy', {
          projectId,
          serviceId,
        });
      }

      setOriginalPolicy(loadedPolicy);
      initializeSelection(loadedPolicy, result.tools);

      // Story 12.5 AC3: 检测是否继承自全局（仅项目模式）
      if (!isGlobalMode) {
        const inherited = arePoliciesEqual(loadedPolicy, globalPolicy);
        setIsInherited(inherited);
      }
    } catch (error) {
      console.error('[ToolPolicyEditor] Failed to load data:', error);
      feedback.error(t('hub.toolPolicy.loadError'), (error as Error).message);
    } finally {
      setIsLoading(false);
    }
  }, [isGlobalMode, projectId, serviceId, initialPolicy, t, initializeSelection, arePoliciesEqual]);

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

      // 如果有原始策略，重新初始化选中状态
      if (originalPolicy) {
        initializeSelection(originalPolicy, result.tools);
      }

      feedback.success(t('hub.toolPolicy.refreshSuccess'));
    } catch (error) {
      console.error('[ToolPolicyEditor] Failed to refresh tools:', error);
      feedback.error(t('hub.toolPolicy.refreshError'), (error as Error).message);
    } finally {
      setIsRefreshing(false);
    }
  }, [serviceId, originalPolicy, t, initializeSelection]);

  // 切换单个工具
  const handleToggleTool = useCallback((toolName: string, checked: boolean) => {
    setSelectedTools(prev => {
      const next = new Set(prev);
      if (checked) {
        next.add(toolName);
      } else {
        next.delete(toolName);
      }
      return next;
    });
  }, []);

  // 全选
  const handleSelectAll = useCallback(() => {
    setSelectedTools(new Set(tools.map(t => t.name)));
  }, [tools]);

  // 全不选
  const handleDeselectAll = useCallback(() => {
    setSelectedTools(new Set());
  }, []);

  // 计算当前 mode
  const currentMode = useMemo(() => {
    return deriveMode(selectedTools, tools.length);
  }, [selectedTools, tools.length]);

  // 构建当前策略
  const buildPolicy = useCallback((): ToolPolicy => {
    const mode = deriveMode(selectedTools, tools.length);

    if (mode === 'allow_all') {
      return { mode: 'allow_all', allowedTools: [], deniedTools: [] };
    }
    if (mode === 'deny_all') {
      return { mode: 'deny_all', allowedTools: [], deniedTools: [] };
    }
    // custom: 只有选中的工具在 allowedTools 中
    return {
      mode: 'custom',
      allowedTools: Array.from(selectedTools),
      deniedTools: [],
    };
  }, [selectedTools, tools.length]);

  // 检查是否有未保存的变更
  const hasChanges = useMemo(() => {
    if (!originalPolicy) return false;
    const currentPolicy = buildPolicy();
    return !arePoliciesEqual(currentPolicy, originalPolicy);
  }, [originalPolicy, buildPolicy, arePoliciesEqual]);

  // Story 12.5 AC3: 当前编辑状态是否与全局一致（用于显示继承状态）
  const currentIsInherited = useMemo(() => {
    if (isGlobalMode || !globalDefaultPolicy) return false;
    const currentPolicy = buildPolicy();
    return arePoliciesEqual(currentPolicy, globalDefaultPolicy);
  }, [isGlobalMode, globalDefaultPolicy, buildPolicy, arePoliciesEqual]);

  // 保存策略
  const handleSave = useCallback(async () => {
    const newPolicy = buildPolicy();

    setIsSaving(true);
    try {
      if (isGlobalMode) {
        // 全局模式：更新服务级默认策略
        await invoke('update_service_default_policy', {
          serviceId,
          policy: newPolicy,
        });
      } else {
        // 项目模式：更新项目级策略
        await invoke('update_project_tool_policy', {
          projectId,
          serviceId,
          policy: newPolicy,
        });
      }

      setOriginalPolicy(newPolicy);
      onPolicyChange?.(newPolicy);
      feedback.success(t('hub.toolPolicy.saveSuccess'));
      onSaved?.();
    } catch (error) {
      console.error('[ToolPolicyEditor] Failed to save policy:', error);
      feedback.error(t('hub.toolPolicy.saveError'), (error as Error).message);
    } finally {
      setIsSaving(false);
    }
  }, [buildPolicy, isGlobalMode, projectId, serviceId, onPolicyChange, onSaved, t]);

  // 渲染工具列表项
  const renderToolItem = (tool: McpTool) => {
    const isSelected = selectedTools.has(tool.name);
    return (
      <div
        key={tool.name}
        className="flex items-start gap-3 p-2 rounded-md hover:bg-accent/50"
        data-testid={`tool-item-${tool.name}`}
      >
        <Checkbox
          id={`tool-${tool.name}`}
          checked={isSelected}
          onCheckedChange={(checked) => handleToggleTool(tool.name, checked === true)}
          data-testid={`tool-checkbox-${tool.name}`}
        />
        <div className="flex-1 min-w-0">
          <label
            htmlFor={`tool-${tool.name}`}
            className="text-sm font-medium cursor-pointer truncate block"
          >
            {tool.name}
          </label>
          {tool.description && (
            <p className="text-xs text-muted-foreground line-clamp-2">
              {tool.description}
            </p>
          )}
        </div>
      </div>
    );
  };

  // 渲染工具列表头部（模式显示 + 操作按钮）
  const renderToolsHeader = () => (
    <>
      {/* 当前模式显示 */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          {getModeIcon(currentMode)}
          <span className="text-sm font-medium">
            {t(`hub.toolPolicy.mode${currentMode === 'allow_all' ? 'AllowAll' : currentMode === 'deny_all' ? 'DenyAll' : 'Custom'}`)}
          </span>
          {/* Story 12.5 AC3: 项目模式下显示自定义/继承状态徽章 */}
          {!isGlobalMode && (
            <Badge
              variant={currentIsInherited ? "secondary" : "default"}
              className="text-[10px] px-1.5 py-0 h-5"
            >
              {currentIsInherited
                ? t('hub.toolPolicy.isInherited', 'Inherited')
                : t('hub.toolPolicy.isCustom', 'Custom')}
            </Badge>
          )}
        </div>
        <Badge variant="outline" className="text-xs">
          {t('hub.toolPolicy.selectedCount', { selected: selectedTools.size, total: tools.length })}
        </Badge>
      </div>

      {/* 操作按钮 */}
      <div className="flex items-center justify-between">
        <Label>{t('hub.toolPolicy.tools')}</Label>
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="sm"
            onClick={handleSelectAll}
            className="h-7 text-xs"
            data-testid="select-all-button"
          >
            <CheckSquare className="h-3 w-3 mr-1" />
            {t('hub.toolPolicy.selectAll')}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleDeselectAll}
            className="h-7 text-xs"
            data-testid="deselect-all-button"
          >
            <Square className="h-3 w-3 mr-1" />
            {t('hub.toolPolicy.deselectAll')}
          </Button>
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
      </div>

      {fromCache && cachedAt && (
        <p className="text-xs text-muted-foreground">
          {t('hub.toolPolicy.cachedAt', { time: new Date(cachedAt).toLocaleString() })}
        </p>
      )}
    </>
  );

  // 渲染保存按钮
  const renderSaveButton = (className?: string) => (
    <div className={className}>
      <Button
        onClick={handleSave}
        disabled={isSaving || !hasChanges}
        data-testid="save-policy-button"
        className="w-full"
      >
        {isSaving ? (
          <Loader2 className="h-4 w-4 mr-2 animate-spin" />
        ) : (
          <Save className="h-4 w-4 mr-2" />
        )}
        {t('hub.toolPolicy.save')}
      </Button>
    </div>
  );

  // 嵌入模式内容（用于 Sheet，全高布局）
  const renderEmbeddedContent = () => {
    if (isLoading) {
      return (
        <div className="flex flex-col flex-1 min-h-0 px-4">
          <div className="space-y-3 py-4">
            <Skeleton className="h-10 w-full" />
            <Skeleton className="h-10 w-full" />
          </div>
          <div className="flex-1 py-2">
            <Skeleton className="h-full w-full" />
          </div>
          <div className="py-4">
            <Skeleton className="h-10 w-full" />
          </div>
        </div>
      );
    }

    return (
      <div className="flex flex-col flex-1 min-h-0">
        {/* Story 12.5: 上下文提示 + AC3 继承状态 */}
        <div className="px-4 pt-4">
          <PolicyContextHint
            isGlobalMode={isGlobalMode}
            projectName={projectName}
            globalDefaultMode={globalDefaultPolicy?.mode}
            isInherited={currentIsInherited}
            t={t}
          />
        </div>

        {/* 头部：模式 + 操作按钮 */}
        <div className="space-y-3 px-4 pb-4 border-b">
          {renderToolsHeader()}
        </div>

        {/* 工具列表：占据剩余空间 */}
        <div className="flex-1 overflow-hidden">
          {tools.length === 0 ? (
            <div className="flex items-center justify-center h-full text-muted-foreground">
              <div className="text-center">
                <p className="text-sm">{t('hub.toolPolicy.noTools')}</p>
                <p className="text-xs mt-1">{t('hub.toolPolicy.noToolsHint')}</p>
              </div>
            </div>
          ) : (
            <ScrollArea className="h-full">
              <div className="space-y-1 p-4">
                {tools.map(renderToolItem)}
              </div>
            </ScrollArea>
          )}
        </div>

        {/* 底部：保存按钮（类似 SheetFooter） */}
        {renderSaveButton("border-t p-4")}
      </div>
    );
  };

  // 渲染内容（用于 Card 和 Collapsible 模式）
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
        {/* 当前模式显示 */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {getModeIcon(currentMode)}
            <span className="text-sm font-medium">
              {t(`hub.toolPolicy.mode${currentMode === 'allow_all' ? 'AllowAll' : currentMode === 'deny_all' ? 'DenyAll' : 'Custom'}`)}
            </span>
          </div>
          <Badge variant="outline" className="text-xs">
            {t('hub.toolPolicy.selectedCount', { selected: selectedTools.size, total: tools.length })}
          </Badge>
        </div>

        {/* 工具列表 */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <Label>{t('hub.toolPolicy.tools')}</Label>
            <div className="flex items-center gap-2">
              <Button
                variant="ghost"
                size="sm"
                onClick={handleSelectAll}
                className="h-7 text-xs"
                data-testid="select-all-button"
              >
                <CheckSquare className="h-3 w-3 mr-1" />
                {t('hub.toolPolicy.selectAll')}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleDeselectAll}
                className="h-7 text-xs"
                data-testid="deselect-all-button"
              >
                <Square className="h-3 w-3 mr-1" />
                {t('hub.toolPolicy.deselectAll')}
              </Button>
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
              <div className="space-y-1">
                {tools.map((tool) => {
                  const isSelected = selectedTools.has(tool.name);
                  return (
                    <div
                      key={tool.name}
                      className="flex items-start gap-3 p-2 rounded-md hover:bg-accent/50"
                      data-testid={`tool-item-${tool.name}`}
                    >
                      <Checkbox
                        id={`tool-${tool.name}`}
                        checked={isSelected}
                        onCheckedChange={(checked) => handleToggleTool(tool.name, checked === true)}
                        data-testid={`tool-checkbox-${tool.name}`}
                      />
                      <div className="flex-1 min-w-0">
                        <label
                          htmlFor={`tool-${tool.name}`}
                          className="text-sm font-medium cursor-pointer truncate block"
                        >
                          {tool.name}
                        </label>
                        {tool.description && (
                          <p className="text-xs text-muted-foreground line-clamp-2">
                            {tool.description}
                          </p>
                        )}
                      </div>
                    </div>
                  );
                })}
              </div>
            </ScrollArea>
          )}
        </div>

        {/* 保存按钮 */}
        <div className="flex justify-end pt-2">
          <Button
            onClick={handleSave}
            disabled={isSaving || !hasChanges}
            data-testid="save-policy-button"
          >
            {isSaving ? (
              <Loader2 className="h-4 w-4 mr-2 animate-spin" />
            ) : (
              <Save className="h-4 w-4 mr-2" />
            )}
            {t('hub.toolPolicy.save')}
          </Button>
        </div>
      </div>
    );
  };

  // 嵌入模式：全高布局，适用于 Sheet
  if (embedded) {
    return renderEmbeddedContent();
  }

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
              {getModeIcon(currentMode)}
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
          {getModeIcon(currentMode)}
          {t('hub.toolPolicy.title')}
        </CardTitle>
        <CardDescription>
          {isGlobalMode
            ? t('hub.toolPolicy.descriptionGlobal', { service: serviceName })
            : t('hub.toolPolicy.description', { service: serviceName })}
        </CardDescription>
      </CardHeader>
      <CardContent>{renderContent()}</CardContent>
    </Card>
  );
}

export default ToolPolicyEditor;
