/**
 * Tool Tester 组件
 * Story 11.11: Task 3 - ToolTester (AC: 3)
 *
 * 工具测试交互面板：
 * - 基于 JSON Schema 动态生成表单
 * - 参数验证
 * - JSON 模式手动编辑
 * - 执行结果展示
 */

import { useState, useCallback, useMemo, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Switch } from "@/components/ui/switch";
import { Badge } from "@/components/ui/badge";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Separator } from "@/components/ui/separator";
import {
  Play,
  Code2,
  FormInput,
  Loader2,
  Check,
  AlertCircle,
  Copy,
  Wrench,
  FileText,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { feedback } from "@/lib/feedback";
import type { McpTool } from "@/types/mcp";
import type { McpResource } from "./InspectorDrawer";
import type { RpcLogEntry } from "./RpcLogViewer";

/**
 * JSON Schema 属性类型
 */
interface JsonSchemaProperty {
  type?: string | string[];
  description?: string;
  default?: unknown;
  enum?: unknown[];
  items?: JsonSchemaProperty;
  properties?: Record<string, JsonSchemaProperty>;
  required?: string[];
}

/**
 * JSON Schema 类型
 */
interface JsonSchema {
  type?: string;
  properties?: Record<string, JsonSchemaProperty>;
  required?: string[];
}

/**
 * ToolTester 属性
 */
export interface ToolTesterProps {
  selectedTool: McpTool | null;
  selectedResource: McpResource | null;
  onExecute: (tool: McpTool, args: Record<string, unknown>) => Promise<unknown>;
  logs: RpcLogEntry[];
}

/**
 * Tool Tester 组件
 */
export function ToolTester({
  selectedTool,
  selectedResource,
  onExecute,
  logs,
}: ToolTesterProps) {
  const { t } = useTranslation();
  const [mode, setMode] = useState<"form" | "json">("form");
  const [formValues, setFormValues] = useState<Record<string, unknown>>({});
  const [jsonInput, setJsonInput] = useState("{}");
  const [isExecuting, setIsExecuting] = useState(false);
  const [result, setResult] = useState<unknown>(null);
  const [error, setError] = useState<string | null>(null);

  // 解析 JSON Schema
  const schema = useMemo((): JsonSchema | null => {
    if (!selectedTool?.inputSchema) return null;
    return selectedTool.inputSchema as JsonSchema;
  }, [selectedTool]);

  // 获取属性列表
  const properties = useMemo(() => {
    if (!schema?.properties) return [];
    return Object.entries(schema.properties).map(([name, prop]) => ({
      name,
      ...prop,
      isRequired: schema.required?.includes(name) || false,
    }));
  }, [schema]);

  // 重置表单
  const resetForm = useCallback(() => {
    const defaults: Record<string, unknown> = {};
    properties.forEach((prop) => {
      if (prop.default !== undefined) {
        defaults[prop.name] = prop.default;
      }
    });
    setFormValues(defaults);
    setJsonInput(JSON.stringify(defaults, null, 2));
    setResult(null);
    setError(null);
  }, [properties]);

  // 当选择的工具变化时重置表单
  useEffect(() => {
    resetForm();
  }, [selectedTool, resetForm]);

  // 更新表单值
  const handleFormChange = useCallback((name: string, value: unknown) => {
    setFormValues((prev) => {
      const next = { ...prev, [name]: value };
      setJsonInput(JSON.stringify(next, null, 2));
      return next;
    });
  }, []);

  // 更新 JSON 输入
  const handleJsonChange = useCallback((value: string) => {
    setJsonInput(value);
    try {
      const parsed = JSON.parse(value);
      setFormValues(parsed);
      setError(null);
    } catch {
      // JSON 解析错误时不更新表单值
    }
  }, []);

  // 验证 JSON
  const validateJson = useCallback((): boolean => {
    try {
      JSON.parse(jsonInput);
      return true;
    } catch {
      return false;
    }
  }, [jsonInput]);

  // 执行工具调用
  const handleExecute = useCallback(async () => {
    if (!selectedTool) return;

    // 验证 JSON
    if (!validateJson()) {
      setError(t("hub.inspector.invalidJson"));
      return;
    }

    setIsExecuting(true);
    setError(null);

    try {
      const args = mode === "json" ? JSON.parse(jsonInput) : formValues;
      const response = await onExecute(selectedTool, args);
      setResult(response);
    } catch (err) {
      setError((err as Error).message);
    } finally {
      setIsExecuting(false);
    }
  }, [selectedTool, mode, jsonInput, formValues, onExecute, validateJson, t]);

  // 复制结果
  const handleCopyResult = useCallback(async () => {
    if (!result) return;
    try {
      await navigator.clipboard.writeText(JSON.stringify(result, null, 2));
      feedback.copied(t("hub.inspector.result"));
    } catch {
      feedback.error(t("common.copy"));
    }
  }, [result, t]);

  // 获取最新的执行结果
  const latestResult = useMemo(() => {
    if (result) return result;
    if (!selectedTool || logs.length === 0) return null;

    const toolLog = logs.find(
      (log) =>
        log.method === "tools/call" &&
        (log.request as { params?: { name?: string } })?.params?.name === selectedTool.name
    );
    return toolLog?.response || null;
  }, [result, selectedTool, logs]);

  // 渲染表单字段
  const renderFormField = useCallback(
    (prop: {
      name: string;
      type?: string | string[];
      description?: string;
      isRequired: boolean;
      enum?: unknown[];
    }) => {
      const value = formValues[prop.name];
      const types = Array.isArray(prop.type) ? prop.type : [prop.type || "string"];
      const primaryType = types[0];

      // Boolean 类型
      if (primaryType === "boolean") {
        return (
          <div key={prop.name} className="flex items-center justify-between py-2">
            <div className="space-y-0.5">
              <Label className="flex items-center gap-2">
                {prop.name}
                {prop.isRequired && (
                  <Badge variant="destructive" className="text-[10px] px-1 py-0">
                    *
                  </Badge>
                )}
              </Label>
              {prop.description && (
                <p className="text-xs text-muted-foreground">{prop.description}</p>
              )}
            </div>
            <Switch
              checked={Boolean(value)}
              onCheckedChange={(checked) => handleFormChange(prop.name, checked)}
            />
          </div>
        );
      }

      // Enum 类型
      if (prop.enum && prop.enum.length > 0) {
        return (
          <div key={prop.name} className="space-y-2">
            <Label className="flex items-center gap-2">
              {prop.name}
              {prop.isRequired && (
                <Badge variant="destructive" className="text-[10px] px-1 py-0">
                  *
                </Badge>
              )}
            </Label>
            {prop.description && (
              <p className="text-xs text-muted-foreground">{prop.description}</p>
            )}
            <div className="flex flex-wrap gap-1">
              {prop.enum.map((option) => (
                <Button
                  key={String(option)}
                  variant={value === option ? "default" : "outline"}
                  size="sm"
                  className="h-7 text-xs"
                  onClick={() => handleFormChange(prop.name, option)}
                >
                  {String(option)}
                </Button>
              ))}
            </div>
          </div>
        );
      }

      // Number 类型
      if (primaryType === "number" || primaryType === "integer") {
        return (
          <div key={prop.name} className="space-y-2">
            <Label className="flex items-center gap-2">
              {prop.name}
              {prop.isRequired && (
                <Badge variant="destructive" className="text-[10px] px-1 py-0">
                  *
                </Badge>
              )}
            </Label>
            {prop.description && (
              <p className="text-xs text-muted-foreground">{prop.description}</p>
            )}
            <Input
              type="number"
              value={value !== undefined ? String(value) : ""}
              onChange={(e) =>
                handleFormChange(
                  prop.name,
                  e.target.value ? Number(e.target.value) : undefined
                )
              }
              placeholder={prop.description}
              className="h-8 text-sm font-mono"
            />
          </div>
        );
      }

      // Array/Object 类型 - 使用 JSON 编辑
      if (primaryType === "array" || primaryType === "object") {
        return (
          <div key={prop.name} className="space-y-2">
            <Label className="flex items-center gap-2">
              {prop.name}
              {prop.isRequired && (
                <Badge variant="destructive" className="text-[10px] px-1 py-0">
                  *
                </Badge>
              )}
              <Badge variant="outline" className="text-[10px] px-1 py-0">
                {primaryType}
              </Badge>
            </Label>
            {prop.description && (
              <p className="text-xs text-muted-foreground">{prop.description}</p>
            )}
            <Textarea
              value={
                value !== undefined ? JSON.stringify(value, null, 2) : ""
              }
              onChange={(e) => {
                try {
                  const parsed = JSON.parse(e.target.value);
                  handleFormChange(prop.name, parsed);
                } catch {
                  // 保持原始字符串，让用户继续编辑
                }
              }}
              placeholder={`Enter ${primaryType} as JSON`}
              className="min-h-[80px] font-mono text-xs"
            />
          </div>
        );
      }

      // String 类型 (默认)
      return (
        <div key={prop.name} className="space-y-2">
          <Label className="flex items-center gap-2">
            {prop.name}
            {prop.isRequired && (
              <Badge variant="destructive" className="text-[10px] px-1 py-0">
                *
              </Badge>
            )}
          </Label>
          {prop.description && (
            <p className="text-xs text-muted-foreground">{prop.description}</p>
          )}
          <Input
            value={value !== undefined ? String(value) : ""}
            onChange={(e) => handleFormChange(prop.name, e.target.value || undefined)}
            placeholder={prop.description}
            className="h-8 text-sm font-mono"
          />
        </div>
      );
    },
    [formValues, handleFormChange]
  );

  // 空状态
  if (!selectedTool && !selectedResource) {
    return (
      <div
        className="flex flex-col items-center justify-center h-full text-muted-foreground"
        data-testid="tool-tester-empty"
      >
        <Wrench className="h-12 w-12 mb-4 opacity-20" />
        <p className="text-sm">{t("hub.inspector.selectToolHint")}</p>
      </div>
    );
  }

  // 资源视图
  if (selectedResource) {
    return (
      <div className="flex flex-col h-full" data-testid="resource-viewer">
        <div className="p-4 border-b">
          <div className="flex items-center gap-3">
            <FileText className="h-5 w-5 text-emerald-500" />
            <div>
              <h3 className="font-medium">{selectedResource.name}</h3>
              <p className="text-xs text-muted-foreground font-mono">
                {selectedResource.uri}
              </p>
            </div>
          </div>
          {selectedResource.description && (
            <p className="text-sm text-muted-foreground mt-2">
              {selectedResource.description}
            </p>
          )}
        </div>
        <ScrollArea className="flex-1 p-4">
          {latestResult ? (
            <pre className="text-xs font-mono bg-muted p-3 rounded-md overflow-auto">
              {JSON.stringify(latestResult, null, 2)}
            </pre>
          ) : (
            <p className="text-sm text-muted-foreground text-center py-8">
              {t("hub.inspector.resourceContentWillAppear")}
            </p>
          )}
        </ScrollArea>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full" data-testid="tool-tester">
      {/* 工具信息头 */}
      <div className="p-4 border-b">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Wrench className="h-5 w-5 text-blue-500" />
            <div>
              <h3 className="font-medium font-mono">{selectedTool?.name}</h3>
              {selectedTool?.description && (
                <p className="text-xs text-muted-foreground mt-0.5">
                  {selectedTool.description}
                </p>
              )}
            </div>
          </div>
          <Button
            onClick={handleExecute}
            disabled={isExecuting || !selectedTool}
            className="gap-2"
            data-testid="tool-execute-button"
          >
            {isExecuting ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Play className="h-4 w-4" />
            )}
            {t("hub.inspector.run")}
          </Button>
        </div>
      </div>

      {/* 参数输入区域 */}
      <div className="flex-1 min-h-0 flex flex-col">
        <Tabs
          value={mode}
          onValueChange={(v) => setMode(v as "form" | "json")}
          className="flex-1 flex flex-col"
        >
          <div className="px-4 pt-3">
            <TabsList className="grid w-full grid-cols-2 max-w-[240px]">
              <TabsTrigger value="form" className="gap-2 text-xs">
                <FormInput className="h-3.5 w-3.5" />
                {t("hub.inspector.formMode")}
              </TabsTrigger>
              <TabsTrigger value="json" className="gap-2 text-xs">
                <Code2 className="h-3.5 w-3.5" />
                {t("hub.inspector.jsonMode")}
              </TabsTrigger>
            </TabsList>
          </div>

          <TabsContent value="form" className="flex-1 min-h-0 mt-0">
            <ScrollArea className="h-full">
              <div className="p-4 space-y-4">
                {properties.length === 0 ? (
                  <p className="text-sm text-muted-foreground text-center py-4">
                    {t("hub.inspector.noParameters")}
                  </p>
                ) : (
                  properties.map(renderFormField)
                )}
              </div>
            </ScrollArea>
          </TabsContent>

          <TabsContent value="json" className="flex-1 min-h-0 mt-0">
            <div className="h-full p-4">
              <Textarea
                value={jsonInput}
                onChange={(e) => handleJsonChange(e.target.value)}
                className={cn(
                  "h-full font-mono text-xs resize-none",
                  !validateJson() && jsonInput !== "{}" && "border-destructive"
                )}
                placeholder='{"key": "value"}'
                data-testid="json-input"
              />
            </div>
          </TabsContent>
        </Tabs>
      </div>

      {/* 结果区域 */}
      {(latestResult || error) && (
        <>
          <Separator />
          <div className="p-4">
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2">
                {error ? (
                  <AlertCircle className="h-4 w-4 text-destructive" />
                ) : (
                  <Check className="h-4 w-4 text-emerald-500" />
                )}
                <span className="text-sm font-medium">
                  {error ? t("common.error") : t("hub.inspector.result")}
                </span>
              </div>
              {latestResult && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={handleCopyResult}
                  className="h-7 gap-1"
                >
                  <Copy className="h-3.5 w-3.5" />
                  {t("common.copy")}
                </Button>
              )}
            </div>
            <ScrollArea className="max-h-[200px]">
              <pre
                className={cn(
                  "text-xs font-mono p-3 rounded-md overflow-auto",
                  error ? "bg-destructive/10 text-destructive" : "bg-muted"
                )}
              >
                {error || JSON.stringify(latestResult, null, 2)}
              </pre>
            </ScrollArea>
          </div>
        </>
      )}
    </div>
  );
}

export default ToolTester;
