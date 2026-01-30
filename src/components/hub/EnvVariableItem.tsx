/**
 * 环境变量项组件
 * Story 11.4: 环境变量管理 - Task 4.2
 *
 * 显示单个环境变量，支持显示/隐藏值、编辑和删除操作
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Eye, EyeOff, Pencil, Trash2, Loader2 } from "lucide-react";
import { getEnvVariableDecrypted, type EnvVariable } from "@/lib/env-variable-ipc";

interface EnvVariableItemProps {
  variable: EnvVariable;
  onEdit: (variable: EnvVariable) => void;
  onDelete: (variable: EnvVariable) => void;
}

export function EnvVariableItem({
  variable,
  onEdit,
  onDelete,
}: EnvVariableItemProps) {
  const { t } = useTranslation();
  const [showValue, setShowValue] = useState(false);
  const [decryptedValue, setDecryptedValue] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  // 5秒后自动隐藏
  useEffect(() => {
    if (showValue) {
      const timer = setTimeout(() => {
        setShowValue(false);
        setDecryptedValue(null);
      }, 5000);
      return () => clearTimeout(timer);
    }
  }, [showValue]);

  const handleShowValue = useCallback(async () => {
    if (showValue) {
      // 如果已经显示，则隐藏
      setShowValue(false);
      setDecryptedValue(null);
      return;
    }

    setIsLoading(true);
    try {
      const value = await getEnvVariableDecrypted(variable.name);
      setDecryptedValue(value);
      setShowValue(true);
    } catch (error) {
      console.error("[EnvVariableItem] Failed to decrypt value:", error);
    } finally {
      setIsLoading(false);
    }
  }, [showValue, variable.name]);

  return (
    <div
      className="flex items-center justify-between p-3 border rounded-lg bg-card hover:bg-accent/50 transition-colors"
      data-testid={`env-variable-item-${variable.name}`}
    >
      <div className="flex-1 min-w-0">
        <div className="font-mono text-sm font-medium text-foreground">
          {variable.name}
        </div>
        <div className="text-sm text-muted-foreground font-mono truncate">
          {showValue && decryptedValue !== null ? decryptedValue : variable.masked_value}
        </div>
        {variable.description && (
          <div className="text-xs text-muted-foreground mt-1 truncate">
            {variable.description}
          </div>
        )}
      </div>
      <div className="flex items-center gap-1 ml-2 flex-shrink-0">
        <Button
          variant="ghost"
          size="sm"
          onClick={handleShowValue}
          disabled={isLoading}
          title={showValue ? t("hub.envVariables.hideValue") : t("hub.envVariables.showValue")}
          data-testid={`env-variable-toggle-${variable.name}`}
        >
          {isLoading ? (
            <Loader2 className="h-4 w-4 animate-spin" />
          ) : showValue ? (
            <EyeOff className="h-4 w-4" />
          ) : (
            <Eye className="h-4 w-4" />
          )}
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onEdit(variable)}
          title={t("common.edit")}
          data-testid={`env-variable-edit-${variable.name}`}
        >
          <Pencil className="h-4 w-4" />
        </Button>
        <Button
          variant="ghost"
          size="sm"
          onClick={() => onDelete(variable)}
          title={t("common.delete")}
          className="text-destructive hover:text-destructive"
          data-testid={`env-variable-delete-${variable.name}`}
        >
          <Trash2 className="h-4 w-4" />
        </Button>
      </div>
    </div>
  );
}

export default EnvVariableItem;
