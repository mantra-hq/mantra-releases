/**
 * Tool/Resource Explorer 组件
 * Story 11.11: Task 2 - ToolExplorer (AC: 2, 4)
 *
 * 显示 MCP 服务的工具和资源树：
 * - 树状结构展示 tools 和 resources
 * - 搜索过滤功能
 * - 显示工具描述和参数概要
 */

import { useState, useMemo, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  Search,
  Wrench,
  FileText,
  ChevronRight,
  ChevronDown,
  X,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type { McpTool } from "@/types/mcp";
import type { McpResource } from "./InspectorDrawer";

/**
 * ToolExplorer 属性
 */
export interface ToolExplorerProps {
  tools: McpTool[];
  resources: McpResource[];
  selectedTool: McpTool | null;
  selectedResource: McpResource | null;
  onSelectTool: (tool: McpTool) => void;
  onSelectResource: (resource: McpResource) => void;
}

/**
 * Tool/Resource Explorer 组件
 */
export function ToolExplorer({
  tools,
  resources,
  selectedTool,
  selectedResource,
  onSelectTool,
  onSelectResource,
}: ToolExplorerProps) {
  const { t } = useTranslation();
  const [search, setSearch] = useState("");
  const [toolsExpanded, setToolsExpanded] = useState(true);
  const [resourcesExpanded, setResourcesExpanded] = useState(true);

  // 过滤工具和资源
  const filteredTools = useMemo(() => {
    if (!search.trim()) return tools;
    const query = search.toLowerCase();
    return tools.filter(
      (tool) =>
        tool.name.toLowerCase().includes(query) ||
        tool.description?.toLowerCase().includes(query)
    );
  }, [tools, search]);

  const filteredResources = useMemo(() => {
    if (!search.trim()) return resources;
    const query = search.toLowerCase();
    return resources.filter(
      (resource) =>
        resource.name.toLowerCase().includes(query) ||
        resource.uri.toLowerCase().includes(query) ||
        resource.description?.toLowerCase().includes(query)
    );
  }, [resources, search]);

  // 清除搜索
  const handleClearSearch = useCallback(() => {
    setSearch("");
  }, []);

  // 获取工具参数数量
  const getParamCount = (tool: McpTool): number => {
    if (!tool.inputSchema) return 0;
    const schema = tool.inputSchema as { properties?: Record<string, unknown> };
    return Object.keys(schema.properties || {}).length;
  };

  // 获取工具必填参数数量
  const getRequiredCount = (tool: McpTool): number => {
    if (!tool.inputSchema) return 0;
    const schema = tool.inputSchema as { required?: string[] };
    return schema.required?.length || 0;
  };

  return (
    <div className="flex flex-col h-full w-full overflow-hidden" data-testid="tool-explorer">
      {/* 搜索框 */}
      <div className="p-3 border-b shrink-0">
        <div className="relative">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground pointer-events-none" />
          <Input
            placeholder={t("hub.inspector.searchPlaceholder")}
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-8 pr-8 h-8 text-sm w-full"
            data-testid="tool-explorer-search"
          />
          {search && (
            <Button
              variant="ghost"
              size="icon"
              className="absolute right-0.5 top-1/2 -translate-y-1/2 h-7 w-7"
              onClick={handleClearSearch}
            >
              <X className="h-3.5 w-3.5" />
            </Button>
          )}
        </div>
      </div>

      {/* 列表区域 */}
      <ScrollArea className="flex-1 min-h-0 w-full">
        <div className="p-2 space-y-1">
          {/* Tools 分组 */}
          <Collapsible open={toolsExpanded} onOpenChange={setToolsExpanded}>
            <CollapsibleTrigger className="flex items-center gap-2 w-full p-2 rounded-md hover:bg-muted/50 text-sm font-medium">
              {toolsExpanded ? (
                <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
              ) : (
                <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
              )}
              <Wrench className="h-4 w-4 shrink-0 text-blue-500" />
              <span className="truncate">{t("hub.inspector.tools")}</span>
              <Badge variant="secondary" className="ml-auto shrink-0 text-xs">
                {filteredTools.length}
              </Badge>
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div className="ml-4 space-y-0.5">
                {filteredTools.length === 0 ? (
                  <p className="text-xs text-muted-foreground py-2 px-2">
                    {search
                      ? t("hub.inspector.noToolsFound")
                      : t("hub.inspector.noTools")}
                  </p>
                ) : (
                  filteredTools.map((tool) => (
                    <TooltipProvider key={tool.name}>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <button
                            className={cn(
                              "flex items-center gap-2 w-full p-2 rounded-md text-sm text-left transition-colors",
                              selectedTool?.name === tool.name
                                ? "bg-accent text-accent-foreground"
                                : "hover:bg-muted/50"
                            )}
                            onClick={() => onSelectTool(tool)}
                            data-testid={`tool-item-${tool.name}`}
                          >
                            <Wrench className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                            <span className="truncate font-mono text-xs flex-1">
                              {tool.name}
                            </span>
                            {getParamCount(tool) > 0 && (
                              <Badge
                                variant="outline"
                                className="ml-auto text-[10px] px-1 py-0 shrink-0"
                              >
                                {getRequiredCount(tool)}/{getParamCount(tool)}
                              </Badge>
                            )}
                          </button>
                        </TooltipTrigger>
                        <TooltipContent side="right" className="max-w-xs">
                          <p className="font-medium">{tool.name}</p>
                          {tool.description && (
                            <p className="text-xs text-muted-foreground mt-1">
                              {tool.description}
                            </p>
                          )}
                        </TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                  ))
                )}
              </div>
            </CollapsibleContent>
          </Collapsible>

          {/* Resources 分组 */}
          <Collapsible open={resourcesExpanded} onOpenChange={setResourcesExpanded}>
            <CollapsibleTrigger className="flex items-center gap-2 w-full p-2 rounded-md hover:bg-muted/50 text-sm font-medium">
              {resourcesExpanded ? (
                <ChevronDown className="h-4 w-4 shrink-0 text-muted-foreground" />
              ) : (
                <ChevronRight className="h-4 w-4 shrink-0 text-muted-foreground" />
              )}
              <FileText className="h-4 w-4 shrink-0 text-emerald-500" />
              <span className="truncate">{t("hub.inspector.resources")}</span>
              <Badge variant="secondary" className="ml-auto shrink-0 text-xs">
                {filteredResources.length}
              </Badge>
            </CollapsibleTrigger>
            <CollapsibleContent>
              <div className="ml-4 space-y-0.5">
                {filteredResources.length === 0 ? (
                  <p className="text-xs text-muted-foreground py-2 px-2">
                    {search
                      ? t("hub.inspector.noResourcesFound")
                      : t("hub.inspector.noResources")}
                  </p>
                ) : (
                  filteredResources.map((resource) => (
                    <TooltipProvider key={resource.uri}>
                      <Tooltip>
                        <TooltipTrigger asChild>
                          <button
                            className={cn(
                              "flex items-center gap-2 w-full p-2 rounded-md text-sm text-left transition-colors",
                              selectedResource?.uri === resource.uri
                                ? "bg-accent text-accent-foreground"
                                : "hover:bg-muted/50"
                            )}
                            onClick={() => onSelectResource(resource)}
                            data-testid={`resource-item-${resource.name}`}
                          >
                            <FileText className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                            <span className="truncate font-mono text-xs flex-1">
                              {resource.name}
                            </span>
                            {resource.mimeType && (
                              <Badge
                                variant="outline"
                                className="ml-auto text-[10px] px-1 py-0 shrink-0"
                              >
                                {resource.mimeType.split("/").pop()}
                              </Badge>
                            )}
                          </button>
                        </TooltipTrigger>
                        <TooltipContent side="right" className="max-w-xs">
                          <p className="font-medium">{resource.name}</p>
                          <p className="text-xs text-muted-foreground font-mono mt-1">
                            {resource.uri}
                          </p>
                          {resource.description && (
                            <p className="text-xs text-muted-foreground mt-1">
                              {resource.description}
                            </p>
                          )}
                        </TooltipContent>
                      </Tooltip>
                    </TooltipProvider>
                  ))
                )}
              </div>
            </CollapsibleContent>
          </Collapsible>
        </div>
      </ScrollArea>

      {/* 统计信息 */}
      <div className="p-2 border-t text-xs text-muted-foreground text-center shrink-0">
        {t("hub.inspector.stats", {
          tools: tools.length,
          resources: resources.length,
        })}
      </div>
    </div>
  );
}

export default ToolExplorer;
