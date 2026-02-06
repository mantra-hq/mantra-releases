/**
 * Tool Policy 编辑器组件
 * Story 11.10 → Story 11.18: 简化的 Tool Policy 模型
 *
 * 用于编辑 MCP 工具策略：
 * - 项目级模式：当 projectId 有值时，编辑项目级策略覆盖
 * - 全局模式：当 projectId 为空时，编辑服务级默认策略
 * - Checkbox 列表选择工具
 * - 全选/全不选 按钮
 * - 手动保存按钮
 *
 * Story 11.18 简化：
 * - allowedTools = null → 继承全局
 * - allowedTools = [] → 全选
 * - allowedTools = [...] → 部分选
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
import { RefreshCw, ChevronDown, Shield, Settings, Loader2, Save, CheckSquare, Square, Info, ArrowLeft } from "lucide-react";
import { feedback } from "@/lib/feedback";
import type { ToolPolicy, McpTool, ToolDiscoveryResult } from "@/types/mcp";
import {
  isInheritPolicy,
  isAllowAllPolicy,
} from "@/types/mcp";

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
 * Story 11.18: 策略状态类型
 * - inherit: 继承全局
 * - allow_all: 全选
 * - custom: 部分选
 */
type PolicyStatus = 'inherit' | 'allow_all' | 'custom';

/**
 * 从 ToolPolicy 获取状态
 */
function getPolicyStatus(policy: ToolPolicy): PolicyStatus {
  if (isInheritPolicy(policy)) return 'inherit';
  if (isAllowAllPolicy(policy)) return 'allow_all';
  return 'custom';
}

/**
 * Story 11.18: 上下文提示组件
 * 区分全局模式和项目模式的策略编辑范围
 */
function PolicyContextHint({
  isGlobalMode,
  projectName,
  globalDefaultStatus,
  isInherited,
  onStartCustomize,
  onResetToInherit,
  t,
}: {
  isGlobalMode: boolean;
  projectName?: string;
  globalDefaultStatus?: PolicyStatus;
  isInherited?: boolean;
  onStartCustomize?: () => void;
  onResetToInherit?: () => void;
  t: ReturnType<typeof useTranslation>['t'];
}) {
  if (isGlobalMode) {
    return (
      <div className="flex items-center gap-2 p-3 bg-blue-500/10 border border-blue-500/20 rounded-lg">
        <Info className="h-4 w-4 text-blue-500 shrink-0" />
        <span className="text-sm text-blue-200">
          {t("hub.toolPolicy.globalHint", "Editing the service's default policy. This will affect all projects without custom configuration.")}
        </span>
      </div>
    );
  }

  // 项目模式：显示继承状态
  const statusLabel = globalDefaultStatus === 'allow_all'
    ? t('hub.toolPolicy.modeAllowAll')
    : t('hub.toolPolicy.modeCustom');

  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2 p-3 bg-amber-500/10 border border-amber-500/20 rounded-lg">
        <Info className="h-4 w-4 text-amber-500 shrink-0" />
        <span className="text-sm text-amber-200">
          {t("hub.toolPolicy.projectHint", "Customizing policy for project {{project}}. This will override the global default.", { project: projectName || "this project" })}
        </span>
      </div>
      {/* Story 11.18: 继承状态指示 + 操作按钮 */}
      {isInherited && globalDefaultStatus && (
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/50 rounded-lg">
          <Info className="h-4 w-4 text-muted-foreground shrink-0" />
          <span className="text-xs text-muted-foreground flex-1">
            {t("hub.toolPolicy.inheritingFrom", "Inheriting from global default: {{mode}}", { mode: statusLabel })}
          </span>
          {onStartCustomize && (
            <Button
              variant="outline"
              size="sm"
              className="h-6 text-xs"
              onClick={onStartCustomize}
              data-testid="start-customize-button"
            >
              {t("hub.toolPolicy.customPolicy", "Use Custom Policy")}
            </Button>
          )}
        </div>
      )}
      {/* Story 11.18: 已自定义时显示恢复继承按钮 */}
      {!isInherited && onResetToInherit && (
        <div className="flex items-center gap-2 px-3 py-2 bg-muted/50 rounded-lg">
          <Info className="h-4 w-4 text-muted-foreground shrink-0" />
          <span className="text-xs text-muted-foreground flex-1">
            {t("hub.toolPolicy.customActive", "Using custom policy for this project")}
          </span>
          <Button
            variant="outline"
            size="sm"
            className="h-6 text-xs"
            onClick={onResetToInherit}
            data-testid="reset-inherit-button"
          >
            <ArrowLeft className="h-3 w-3 mr-1" />
            {t("hub.toolPolicy.resetInherit", "Reset to Inherit")}
          </Button>
        </div>
      )}
    </div>
  );
}

/**
 * Story 11.18: 获取状态图标
 */
function getStatusIcon(status: PolicyStatus) {
  switch (status) {
    case 'allow_all':
      return <Shield className="h-4 w-4 text-green-500" />;
    case 'custom':
      return <Settings className="h-4 w-4 text-yellow-500" />;
    case 'inherit':
      return <Shield className="h-4 w-4 text-muted-foreground" />;
    default:
      return null;
  }
}

/**
 * Story 11.18: 从选中的工具列表推导状态
 */
function deriveStatus(selectedTools: Set<string>, totalTools: number): PolicyStatus {
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

  // Story 11.18: 从策略初始化选中状态
  const initializeSelection = useCallback((policy: ToolPolicy, toolList: McpTool[]) => {
    const selected = new Set<string>();

    // Story 11.18: 简化的逻辑
    // - allowedTools = null (继承) 或 [] (全选): 全部选中
    // - allowedTools = [...] (部分选): 只选中列表中的
    if (policy.allowedTools === null || policy.allowedTools.length === 0) {
      // 继承或全选: 全部选中
      for (const tool of toolList) {
        selected.add(tool.name);
      }
    } else {
      // 部分选: 只选中 allowedTools 中的
      for (const toolName of policy.allowedTools) {
        selected.add(toolName);
      }
    }

    setSelectedTools(selected);
  }, []);

  /**
   * Story 11.18: 比较两个策略是否相同
   */
  const arePoliciesEqual = useCallback((p1: ToolPolicy, p2: ToolPolicy): boolean => {
    const status1 = getPolicyStatus(p1);
    const status2 = getPolicyStatus(p2);

    if (status1 !== status2) return false;

    if (status1 === 'custom') {
      const allowed1 = new Set(p1.allowedTools || []);
      const allowed2 = new Set(p2.allowedTools || []);
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
      // Story 12.6: 当项目策略是继承模式时，使用全局策略初始化选中状态
      // 这样显示的是真正生效的工具选择，与 check_project_mcp_status 的逻辑一致
      const effectivePolicy = (loadedPolicy.allowedTools === null && globalPolicy)
        ? globalPolicy
        : loadedPolicy;
      initializeSelection(effectivePolicy, result.tools);
      // Story 12.5 AC3: 继承状态现在通过 currentIsInherited useMemo 自动计算
    } catch (error) {
      console.error('[ToolPolicyEditor] Failed to load data:', error);
      feedback.error(t('hub.toolPolicy.loadError'), (error as Error).message);
    } finally {
      setIsLoading(false);
    }
  }, [isGlobalMode, projectId, serviceId, initialPolicy, t, initializeSelection]);

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
        // Story 12.6: 当项目策略是继承模式时，使用全局策略初始化选中状态
        const effectivePolicy = (originalPolicy.allowedTools === null && globalDefaultPolicy)
          ? globalDefaultPolicy
          : originalPolicy;
        initializeSelection(effectivePolicy, result.tools);
      }

      feedback.success(t('hub.toolPolicy.refreshSuccess'));
    } catch (error) {
      console.error('[ToolPolicyEditor] Failed to refresh tools:', error);
      feedback.error(t('hub.toolPolicy.refreshError'), (error as Error).message);
    } finally {
      setIsRefreshing(false);
    }
  }, [serviceId, originalPolicy, globalDefaultPolicy, t, initializeSelection]);

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

  // 反选
  const handleInvertSelection = useCallback(() => {
    setSelectedTools(prev => {
      const next = new Set<string>();
      for (const tool of tools) {
        if (!prev.has(tool.name)) {
          next.add(tool.name);
        }
      }
      return next;
    });
  }, [tools]);

  // 计算当前状态
  const currentStatus = useMemo(() => {
    return deriveStatus(selectedTools, tools.length);
  }, [selectedTools, tools.length]);

  // Story 11.18: 构建当前策略
  const buildPolicy = useCallback((): ToolPolicy => {
    const status = deriveStatus(selectedTools, tools.length);

    if (status === 'allow_all') {
      // 全选: allowedTools = []
      return { allowedTools: [] };
    }
    // 部分选: allowedTools = [...]
    return {
      allowedTools: Array.from(selectedTools),
    };
  }, [selectedTools, tools.length]);

  // 检查是否有未保存的变更
  const hasChanges = useMemo(() => {
    if (!originalPolicy) return false;
    const currentPolicy = buildPolicy();
    return !arePoliciesEqual(currentPolicy, originalPolicy);
  }, [originalPolicy, buildPolicy, arePoliciesEqual]);

  // Story 11.18: 当前编辑状态是否与全局一致（用于显示继承状态）
  const currentIsInherited = useMemo(() => {
    if (isGlobalMode || !globalDefaultPolicy) return false;
    const currentPolicy = buildPolicy();
    return arePoliciesEqual(currentPolicy, globalDefaultPolicy);
  }, [isGlobalMode, globalDefaultPolicy, buildPolicy, arePoliciesEqual]);

  // Story 11.18: 开始自定义策略（取消选中一个工具触发变更）
  const handleStartCustomize = useCallback(() => {
    // 触发一个小变更让用户开始自定义
    // 取消选中第一个工具
    if (tools.length === 0) return;

    const firstTool = tools[0].name;
    setSelectedTools(prev => {
      const next = new Set(prev);
      if (next.has(firstTool)) {
        next.delete(firstTool);
      } else {
        next.add(firstTool);
      }
      return next;
    });
  }, [tools]);

  // Story 11.18: 重置到继承模式（恢复到全局默认）
  const handleResetToInherit = useCallback(() => {
    if (!globalDefaultPolicy) return;
    // 恢复选中状态到全局默认
    initializeSelection(globalDefaultPolicy, tools);
  }, [globalDefaultPolicy, tools, initializeSelection]);

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
        className="flex items-start gap-3 p-2 rounded-md hover:bg-accent/50 cursor-pointer"
        onClick={() => handleToggleTool(tool.name, !isSelected)}
        data-testid={`tool-item-${tool.name}`}
      >
        <Checkbox
          id={`tool-${tool.name}`}
          checked={isSelected}
          onCheckedChange={(checked) => handleToggleTool(tool.name, checked === true)}
          onClick={(e: React.MouseEvent) => e.stopPropagation()}
          className="border-zinc-400 data-[state=unchecked]:bg-zinc-700/30"
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
          {getStatusIcon(currentStatus)}
          <span className="text-sm font-medium">
            {t(`hub.toolPolicy.mode${currentStatus === 'allow_all' ? 'AllowAll' : 'Custom'}`)}
          </span>
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
            onClick={selectedTools.size === tools.length ? handleDeselectAll : handleSelectAll}
            className="h-7 text-xs"
            data-testid="toggle-all-button"
          >
            {selectedTools.size === tools.length ? (
              <>
                <Square className="h-3 w-3 mr-1" />
                {t('hub.toolPolicy.clearSelection')}
              </>
            ) : (
              <>
                <CheckSquare className="h-3 w-3 mr-1" />
                {t('hub.toolPolicy.selectAll')}
              </>
            )}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={handleInvertSelection}
            disabled={tools.length === 0}
            className="h-7 text-xs"
            data-testid="invert-selection-button"
          >
            {t('hub.toolPolicy.invertSelection')}
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
            globalDefaultStatus={globalDefaultPolicy ? getPolicyStatus(globalDefaultPolicy) : undefined}
            isInherited={currentIsInherited}
            onStartCustomize={handleStartCustomize}
            onResetToInherit={handleResetToInherit}
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
            {getStatusIcon(currentStatus)}
            <span className="text-sm font-medium">
              {t(`hub.toolPolicy.mode${currentStatus === 'allow_all' ? 'AllowAll' : 'Custom'}`)}
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
                onClick={selectedTools.size === tools.length ? handleDeselectAll : handleSelectAll}
                className="h-7 text-xs"
                data-testid="toggle-all-button"
              >
                {selectedTools.size === tools.length ? (
                  <>
                    <Square className="h-3 w-3 mr-1" />
                    {t('hub.toolPolicy.clearSelection')}
                  </>
                ) : (
                  <>
                    <CheckSquare className="h-3 w-3 mr-1" />
                    {t('hub.toolPolicy.selectAll')}
                  </>
                )}
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleInvertSelection}
                disabled={tools.length === 0}
                className="h-7 text-xs"
                data-testid="invert-selection-button"
              >
                {t('hub.toolPolicy.invertSelection')}
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
                      className="flex items-start gap-3 p-2 rounded-md hover:bg-accent/50 cursor-pointer"
                      onClick={() => handleToggleTool(tool.name, !isSelected)}
                      data-testid={`tool-item-${tool.name}`}
                    >
                      <Checkbox
                        id={`tool-${tool.name}`}
                        checked={isSelected}
                        onCheckedChange={(checked) => handleToggleTool(tool.name, checked === true)}
                        onClick={(e: React.MouseEvent) => e.stopPropagation()}
                        className="border-zinc-400 data-[state=unchecked]:bg-zinc-700/30"
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
              {getStatusIcon(currentStatus)}
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
          {getStatusIcon(currentStatus)}
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
