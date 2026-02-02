/**
 * MCP 服务表单 Sheet 组件
 * Story 12.2: 简单表单 Dialog 改造为 Sheet - Task 4
 * Story 12.4: 迁移使用 ActionSheet 统一封装组件
 *
 * 添加/编辑 MCP 服务的表单：
 * - 传输类型 (stdio / http)
 * - stdio 模式: 启动命令、命令参数
 * - http 模式: URL、请求头
 * - 环境变量引用 (JSON 对象)
 * - 表单校验
 */

import { useState, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { Button } from "@/components/ui/button";
import {
  ActionSheet,
  ActionSheetContent,
  ActionSheetDescription,
  ActionSheetFooter,
  ActionSheetHeader,
  ActionSheetTitle,
} from "@/components/ui/action-sheet";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Loader2 } from "lucide-react";
import { feedback } from "@/lib/feedback";
import type { McpService } from "./McpServiceList";

/** 传输类型 */
type TransportType = "stdio" | "http";

/**
 * 创建 MCP 服务请求
 */
interface CreateMcpServiceRequest {
  name: string;
  transport_type: TransportType;
  command: string;
  args: string[] | null;
  env: Record<string, string> | null;
  url: string | null;
  headers: Record<string, string> | null;
  source: "manual";
  source_file: null;
}

/**
 * 更新 MCP 服务请求
 */
interface UpdateMcpServiceRequest {
  name?: string;
  transport_type?: TransportType;
  command?: string;
  args?: string[] | null;
  env?: Record<string, string> | null;
  url?: string | null;
  headers?: Record<string, string> | null;
  enabled?: boolean;
}

interface McpServiceSheetProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  editService: McpService | null;
  onSuccess: () => void;
}

export function McpServiceSheet({
  open,
  onOpenChange,
  editService,
  onSuccess,
}: McpServiceSheetProps) {
  const { t } = useTranslation();
  const [isSubmitting, setIsSubmitting] = useState(false);

  // 表单状态
  const [name, setName] = useState("");
  const [transportType, setTransportType] = useState<TransportType>("stdio");
  // stdio 模式
  const [command, setCommand] = useState("");
  const [argsText, setArgsText] = useState("");
  // http 模式
  const [url, setUrl] = useState("");
  const [headersText, setHeadersText] = useState("");
  // 通用
  const [envText, setEnvText] = useState("");

  // 错误状态
  const [nameError, setNameError] = useState<string | null>(null);
  const [commandError, setCommandError] = useState<string | null>(null);
  const [argsError, setArgsError] = useState<string | null>(null);
  const [urlError, setUrlError] = useState<string | null>(null);
  const [headersError, setHeadersError] = useState<string | null>(null);
  const [envError, setEnvError] = useState<string | null>(null);

  // 初始化表单
  useEffect(() => {
    if (open) {
      if (editService) {
        setName(editService.name);
        setTransportType(editService.transport_type || "stdio");
        setCommand(editService.command || "");
        setArgsText(
          editService.args ? JSON.stringify(editService.args, null, 2) : ""
        );
        setUrl(editService.url || "");
        setHeadersText(
          editService.headers ? JSON.stringify(editService.headers, null, 2) : ""
        );
        setEnvText(
          editService.env ? JSON.stringify(editService.env, null, 2) : ""
        );
      } else {
        setName("");
        setTransportType("stdio");
        setCommand("");
        setArgsText("");
        setUrl("");
        setHeadersText("");
        setEnvText("");
      }
      // 清除错误
      setNameError(null);
      setCommandError(null);
      setArgsError(null);
      setUrlError(null);
      setHeadersError(null);
      setEnvError(null);
    }
  }, [open, editService]);

  // 验证表单
  const validate = useCallback((): boolean => {
    let isValid = true;

    // 验证名称
    if (!name.trim()) {
      setNameError(t("hub.services.form.nameRequired"));
      isValid = false;
    } else {
      setNameError(null);
    }

    if (transportType === "stdio") {
      // stdio 模式：验证命令
      if (!command.trim()) {
        setCommandError(t("hub.services.form.commandRequired"));
        isValid = false;
      } else {
        setCommandError(null);
      }

      // 验证参数 JSON
      if (argsText.trim()) {
        try {
          const parsed = JSON.parse(argsText);
          if (!Array.isArray(parsed)) {
            setArgsError(t("hub.services.form.argsMustBeArray"));
            isValid = false;
          } else {
            setArgsError(null);
          }
        } catch {
          setArgsError(t("hub.services.form.invalidJson"));
          isValid = false;
        }
      } else {
        setArgsError(null);
      }

      // 清除 http 错误
      setUrlError(null);
      setHeadersError(null);
    } else {
      // http 模式：验证 URL
      if (!url.trim()) {
        setUrlError(t("hub.services.form.urlRequired", "URL is required"));
        isValid = false;
      } else if (!url.startsWith("http://") && !url.startsWith("https://")) {
        setUrlError(t("hub.services.form.urlInvalid", "URL must start with http:// or https://"));
        isValid = false;
      } else {
        setUrlError(null);
      }

      // 验证请求头 JSON
      if (headersText.trim()) {
        try {
          const parsed = JSON.parse(headersText);
          if (typeof parsed !== "object" || Array.isArray(parsed)) {
            setHeadersError(t("hub.services.form.headersMustBeObject", "Headers must be a JSON object"));
            isValid = false;
          } else {
            setHeadersError(null);
          }
        } catch {
          setHeadersError(t("hub.services.form.invalidJson"));
          isValid = false;
        }
      } else {
        setHeadersError(null);
      }

      // 清除 stdio 错误
      setCommandError(null);
      setArgsError(null);
    }

    // 验证环境变量 JSON
    if (envText.trim()) {
      try {
        const parsed = JSON.parse(envText);
        if (typeof parsed !== "object" || Array.isArray(parsed)) {
          setEnvError(t("hub.services.form.envMustBeObject"));
          isValid = false;
        } else {
          setEnvError(null);
        }
      } catch {
        setEnvError(t("hub.services.form.invalidJson"));
        isValid = false;
      }
    } else {
      setEnvError(null);
    }

    return isValid;
  }, [name, transportType, command, argsText, url, headersText, envText, t]);

  // 提交表单
  const handleSubmit = useCallback(async () => {
    if (!validate()) return;

    setIsSubmitting(true);
    try {
      const args = argsText.trim() ? JSON.parse(argsText) : null;
      const headers = headersText.trim() ? JSON.parse(headersText) : null;
      const env = envText.trim() ? JSON.parse(envText) : null;

      if (editService) {
        // 更新服务
        const updates: UpdateMcpServiceRequest = {
          name: name.trim(),
          transport_type: transportType,
          command: transportType === "stdio" ? command.trim() : "",
          args: transportType === "stdio" ? args : null,
          url: transportType === "http" ? url.trim() : null,
          headers: transportType === "http" ? headers : null,
          env,
        };
        await invoke<McpService>("update_mcp_service", {
          id: editService.id,
          updates,
        });
        feedback.success(t("hub.services.updateSuccess"));
      } else {
        // 创建服务
        const request: CreateMcpServiceRequest = {
          name: name.trim(),
          transport_type: transportType,
          command: transportType === "stdio" ? command.trim() : "",
          args: transportType === "stdio" ? args : null,
          url: transportType === "http" ? url.trim() : null,
          headers: transportType === "http" ? headers : null,
          env,
          source: "manual",
          source_file: null,
        };
        await invoke<McpService>("create_mcp_service", { request });
        feedback.success(t("hub.services.createSuccess"));
      }

      onOpenChange(false);
      onSuccess();
    } catch (error) {
      console.error("[McpServiceSheet] Failed to save service:", error);
      feedback.error(
        editService ? t("hub.services.updateError") : t("hub.services.createError"),
        (error as Error).message
      );
    } finally {
      setIsSubmitting(false);
    }
  }, [
    validate,
    editService,
    name,
    transportType,
    command,
    argsText,
    url,
    headersText,
    envText,
    onOpenChange,
    onSuccess,
    t,
  ]);

  return (
    <ActionSheet open={open} onOpenChange={onOpenChange}>
      <ActionSheetContent size="lg" className="overflow-y-auto">
        <ActionSheetHeader>
          <ActionSheetTitle>
            {editService
              ? t("hub.services.form.editTitle")
              : t("hub.services.form.addTitle")}
          </ActionSheetTitle>
          <ActionSheetDescription>
            {editService
              ? t("hub.services.form.editDescription")
              : t("hub.services.form.addDescription")}
          </ActionSheetDescription>
        </ActionSheetHeader>

        <div className="space-y-4 py-4 px-4">
          {/* 服务名称 */}
          <div className="space-y-2">
            <Label htmlFor="service-name">{t("hub.services.form.nameLabel")}</Label>
            <Input
              id="service-name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder={t("hub.services.form.namePlaceholder")}
              data-testid="mcp-service-name-input"
            />
            {nameError && (
              <p className="text-xs text-destructive">{nameError}</p>
            )}
          </div>

          {/* 传输类型 */}
          <div className="space-y-2">
            <Label htmlFor="transport-type">
              {t("hub.services.form.transportTypeLabel", "Transport Type")}
            </Label>
            <Select
              value={transportType}
              onValueChange={(value: TransportType) => setTransportType(value)}
            >
              <SelectTrigger id="transport-type" data-testid="mcp-service-transport-select">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="stdio">
                  {t("hub.services.form.transportStdio", "stdio (Local Command)")}
                </SelectItem>
                <SelectItem value="http">
                  {t("hub.services.form.transportHttp", "http (Remote URL)")}
                </SelectItem>
              </SelectContent>
            </Select>
            <p className="text-xs text-muted-foreground">
              {transportType === "stdio"
                ? t("hub.services.form.transportStdioHint", "Run a local command to start the MCP server")
                : t("hub.services.form.transportHttpHint", "Connect to a remote MCP server via HTTP")}
            </p>
          </div>

          {transportType === "stdio" ? (
            <>
              {/* stdio 模式: 启动命令 */}
              <div className="space-y-2">
                <Label htmlFor="service-command">{t("hub.services.form.commandLabel")}</Label>
                <Input
                  id="service-command"
                  value={command}
                  onChange={(e) => setCommand(e.target.value)}
                  placeholder={t("hub.services.form.commandPlaceholder")}
                  data-testid="mcp-service-command-input"
                />
                {commandError && (
                  <p className="text-xs text-destructive">{commandError}</p>
                )}
              </div>

              {/* stdio 模式: 命令参数 */}
              <div className="space-y-2">
                <Label htmlFor="service-args">{t("hub.services.form.argsLabel")}</Label>
                <Textarea
                  id="service-args"
                  value={argsText}
                  onChange={(e) => setArgsText(e.target.value)}
                  placeholder={t("hub.services.form.argsPlaceholder")}
                  className="font-mono text-sm min-h-[80px]"
                  data-testid="mcp-service-args-input"
                />
                <p className="text-xs text-muted-foreground">
                  {t("hub.services.form.argsHint")}
                </p>
                {argsError && (
                  <p className="text-xs text-destructive">{argsError}</p>
                )}
              </div>
            </>
          ) : (
            <>
              {/* http 模式: URL */}
              <div className="space-y-2">
                <Label htmlFor="service-url">
                  {t("hub.services.form.urlLabel", "MCP Endpoint URL")}
                </Label>
                <Input
                  id="service-url"
                  value={url}
                  onChange={(e) => setUrl(e.target.value)}
                  placeholder="https://mcp.example.com/mcp"
                  data-testid="mcp-service-url-input"
                />
                {urlError && (
                  <p className="text-xs text-destructive">{urlError}</p>
                )}
              </div>

              {/* http 模式: 请求头 */}
              <div className="space-y-2">
                <Label htmlFor="service-headers">
                  {t("hub.services.form.headersLabel", "HTTP Headers (Optional)")}
                </Label>
                <Textarea
                  id="service-headers"
                  value={headersText}
                  onChange={(e) => setHeadersText(e.target.value)}
                  placeholder='{"Authorization": "Bearer your-token"}'
                  className="font-mono text-sm min-h-[80px]"
                  data-testid="mcp-service-headers-input"
                />
                <p className="text-xs text-muted-foreground">
                  {t("hub.services.form.headersHint", "JSON object for custom HTTP headers (e.g., authentication)")}
                </p>
                {headersError && (
                  <p className="text-xs text-destructive">{headersError}</p>
                )}
              </div>
            </>
          )}

          {/* 环境变量 */}
          <div className="space-y-2">
            <Label htmlFor="service-env">{t("hub.services.form.envLabel")}</Label>
            <Textarea
              id="service-env"
              value={envText}
              onChange={(e) => setEnvText(e.target.value)}
              placeholder={t("hub.services.form.envPlaceholder")}
              className="font-mono text-sm min-h-[80px]"
              data-testid="mcp-service-env-input"
            />
            <p className="text-xs text-muted-foreground">
              {t("hub.services.form.envHint")}
            </p>
            {envError && (
              <p className="text-xs text-destructive">{envError}</p>
            )}
          </div>
        </div>

        <ActionSheetFooter>
          <Button
            variant="outline"
            onClick={() => onOpenChange(false)}
            disabled={isSubmitting}
          >
            {t("common.cancel")}
          </Button>
          <Button
            onClick={handleSubmit}
            disabled={isSubmitting}
            data-testid="mcp-service-submit-button"
          >
            {isSubmitting && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {editService ? t("common.save") : t("common.create")}
          </Button>
        </ActionSheetFooter>
      </ActionSheetContent>
    </ActionSheet>
  );
}

export default McpServiceSheet;
