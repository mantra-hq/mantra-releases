/**
 * MCP 服务表单组件
 * Story 11.6: Task 4 - MCP 服务表单 (AC: #4, #5)
 *
 * 添加/编辑 MCP 服务的表单：
 * - 服务名称
 * - 启动命令
 * - 命令参数 (JSON 数组)
 * - 环境变量引用 (JSON 对象)
 * - 表单校验
 */

import { useState, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Loader2 } from "lucide-react";
import { feedback } from "@/lib/feedback";
import type { McpService } from "./McpServiceList";

/**
 * 创建 MCP 服务请求
 */
interface CreateMcpServiceRequest {
  name: string;
  command: string;
  args: string[] | null;
  env: Record<string, string> | null;
}

/**
 * 更新 MCP 服务请求
 */
interface UpdateMcpServiceRequest {
  name?: string;
  command?: string;
  args?: string[] | null;
  env?: Record<string, string> | null;
  enabled?: boolean;
}

interface McpServiceFormProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  editService: McpService | null;
  onSuccess: () => void;
}

export function McpServiceForm({
  open,
  onOpenChange,
  editService,
  onSuccess,
}: McpServiceFormProps) {
  const { t } = useTranslation();
  const [isSubmitting, setIsSubmitting] = useState(false);

  // 表单状态
  const [name, setName] = useState("");
  const [command, setCommand] = useState("");
  const [argsText, setArgsText] = useState("");
  const [envText, setEnvText] = useState("");

  // 错误状态
  const [nameError, setNameError] = useState<string | null>(null);
  const [commandError, setCommandError] = useState<string | null>(null);
  const [argsError, setArgsError] = useState<string | null>(null);
  const [envError, setEnvError] = useState<string | null>(null);

  // 初始化表单
  useEffect(() => {
    if (open) {
      if (editService) {
        setName(editService.name);
        setCommand(editService.command);
        setArgsText(
          editService.args ? JSON.stringify(editService.args, null, 2) : ""
        );
        setEnvText(
          editService.env ? JSON.stringify(editService.env, null, 2) : ""
        );
      } else {
        setName("");
        setCommand("");
        setArgsText("");
        setEnvText("");
      }
      // 清除错误
      setNameError(null);
      setCommandError(null);
      setArgsError(null);
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

    // 验证命令
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
  }, [name, command, argsText, envText, t]);

  // 提交表单
  const handleSubmit = useCallback(async () => {
    if (!validate()) return;

    setIsSubmitting(true);
    try {
      const args = argsText.trim() ? JSON.parse(argsText) : null;
      const env = envText.trim() ? JSON.parse(envText) : null;

      if (editService) {
        // 更新服务
        const updates: UpdateMcpServiceRequest = {
          name: name.trim(),
          command: command.trim(),
          args,
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
          command: command.trim(),
          args,
          env,
        };
        await invoke<McpService>("create_mcp_service", { request });
        feedback.success(t("hub.services.createSuccess"));
      }

      onOpenChange(false);
      onSuccess();
    } catch (error) {
      console.error("[McpServiceForm] Failed to save service:", error);
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
    command,
    argsText,
    envText,
    onOpenChange,
    onSuccess,
    t,
  ]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>
            {editService
              ? t("hub.services.form.editTitle")
              : t("hub.services.form.addTitle")}
          </DialogTitle>
          <DialogDescription>
            {editService
              ? t("hub.services.form.editDescription")
              : t("hub.services.form.addDescription")}
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
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

          {/* 启动命令 */}
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

          {/* 命令参数 */}
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

        <DialogFooter>
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
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export default McpServiceForm;
