/**
 * Local API Server 配置组件
 * Story 3.11: Task 4.5 - AC #7
 *
 * 允许用户配置本地 API Server 端口
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Server, RefreshCw, Check, AlertCircle, Circle, Loader2 } from "lucide-react";
import {
  getLocalServerStatus,
  getLocalServerConfig,
  updateLocalServerPort,
  isValidPort,
  DEFAULT_PORT,
  type LocalServerStatus,
} from "@/lib/local-server-ipc";
import { feedback } from "@/lib/feedback";

export function LocalServerConfig() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<LocalServerStatus | null>(null);
  const [port, setPort] = useState<string>("");
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasChanges, setHasChanges] = useState(false);

  // 加载初始状态
  useEffect(() => {
    async function loadStatus() {
      try {
        const [statusResult, configResult] = await Promise.all([
          getLocalServerStatus(),
          getLocalServerConfig(),
        ]);
        setStatus(statusResult);
        setPort(configResult.local_api_port.toString());
        setError(null);
      } catch (err) {
        console.error("[LocalServerConfig] Failed to load status:", err);
        setError(t("settings.localServer.loadError"));
      } finally {
        setIsLoading(false);
      }
    }
    loadStatus();
  }, [t]);

  // 检测端口变化
  useEffect(() => {
    if (status) {
      setHasChanges(port !== status.port.toString());
    }
  }, [port, status]);

  // 端口输入处理
  const handlePortChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const value = e.target.value;
    // 只允许数字
    if (value === "" || /^\d+$/.test(value)) {
      setPort(value);
      setError(null);
    }
  }, []);

  // 保存端口配置
  const handleSave = useCallback(async () => {
    const portNum = parseInt(port, 10);
    
    if (!isValidPort(portNum)) {
      setError(t("settings.localServer.invalidPort"));
      return;
    }

    setIsSaving(true);
    setError(null);

    try {
      const newStatus = await updateLocalServerPort(portNum);
      setStatus(newStatus);
      setHasChanges(false);
      feedback.saved(t("settings.localServer.saveSuccess"));
    } catch (err) {
      console.error("[LocalServerConfig] Failed to update port:", err);
      const errorMsg = (err as { message?: string })?.message || t("settings.localServer.saveError");
      setError(errorMsg);
      feedback.error(t("settings.localServer.saveError"), errorMsg);
    } finally {
      setIsSaving(false);
    }
  }, [port, t]);

  // 重置为默认端口
  const handleReset = useCallback(() => {
    setPort(DEFAULT_PORT.toString());
    setError(null);
  }, []);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-4">
        <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-3">
        <div className="p-2 rounded-md bg-blue-500/10">
          <Server className="h-5 w-5 text-blue-500" />
        </div>
        <div>
          <h3 className="text-sm font-medium">{t("settings.localServer.title")}</h3>
          <p className="text-xs text-muted-foreground">
            {t("settings.localServer.description")}
          </p>
        </div>
      </div>

      {/* 状态指示器 */}
      <div className="flex items-center gap-2 text-sm">
        {status?.running ? (
          <>
            <Circle className="h-3 w-3 fill-emerald-500 text-emerald-500" />
            <span className="text-emerald-500">
              {t("settings.localServer.running", { port: status.port })}
            </span>
          </>
        ) : (
          <>
            <Circle className="h-3 w-3 fill-muted text-muted" />
            <span className="text-muted-foreground">
              {t("settings.localServer.stopped")}
            </span>
          </>
        )}
      </div>

      {/* 端口配置 */}
      <div className="space-y-2">
        <Label htmlFor="port-input" className="text-sm">
          {t("settings.localServer.portLabel")}
        </Label>
        <div className="flex items-center gap-2">
          <Input
            id="port-input"
            type="text"
            inputMode="numeric"
            value={port}
            onChange={handlePortChange}
            placeholder={DEFAULT_PORT.toString()}
            className="w-32"
            data-testid="local-server-port-input"
          />
          <Button
            variant="outline"
            size="sm"
            onClick={handleReset}
            disabled={port === DEFAULT_PORT.toString()}
            title={t("settings.localServer.resetToDefault")}
          >
            <RefreshCw className="h-4 w-4" />
          </Button>
          <Button
            size="sm"
            onClick={handleSave}
            disabled={!hasChanges || isSaving}
            data-testid="local-server-save-button"
          >
            {isSaving ? (
              <Loader2 className="h-4 w-4 animate-spin" />
            ) : (
              <Check className="h-4 w-4" />
            )}
            <span className="ml-1">{t("common.save")}</span>
          </Button>
        </div>
        <p className="text-xs text-muted-foreground">
          {t("settings.localServer.portHint")}
        </p>
      </div>

      {/* 错误提示 */}
      {error && (
        <div className="flex items-center gap-2 text-sm text-destructive">
          <AlertCircle className="h-4 w-4" />
          <span>{error}</span>
        </div>
      )}
    </div>
  );
}

export default LocalServerConfig;
