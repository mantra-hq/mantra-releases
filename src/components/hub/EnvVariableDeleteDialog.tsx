/**
 * 环境变量删除确认对话框
 * Story 11.4: 环境变量管理 - Task 4.5
 *
 * 显示受影响的 MCP 服务列表，确认后删除
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { AlertTriangle, Loader2, Server } from "lucide-react";
import {
  deleteEnvVariable,
  getAffectedMcpServices,
  type EnvVariable,
  type McpService,
} from "@/lib/env-variable-ipc";
import { feedback } from "@/lib/feedback";

interface EnvVariableDeleteDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  variable: EnvVariable | null;
  onSuccess: () => void;
}

export function EnvVariableDeleteDialog({
  open,
  onOpenChange,
  variable,
  onSuccess,
}: EnvVariableDeleteDialogProps) {
  const { t } = useTranslation();
  const [affectedServices, setAffectedServices] = useState<McpService[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);

  // 加载受影响的服务
  useEffect(() => {
    if (open && variable) {
      setIsLoading(true);
      getAffectedMcpServices(variable.name)
        .then((services) => {
          setAffectedServices(services);
        })
        .catch((error) => {
          console.error("[EnvVariableDeleteDialog] Failed to load affected services:", error);
          setAffectedServices([]);
        })
        .finally(() => {
          setIsLoading(false);
        });
    } else {
      setAffectedServices([]);
    }
  }, [open, variable]);

  const handleDelete = useCallback(async () => {
    if (!variable) return;

    setIsDeleting(true);
    try {
      await deleteEnvVariable(variable.name);
      feedback.saved(t("hub.envVariables.deleteSuccess"));
      onSuccess();
      onOpenChange(false);
    } catch (error) {
      console.error("[EnvVariableDeleteDialog] Failed to delete:", error);
      feedback.error(t("hub.envVariables.deleteError"));
    } finally {
      setIsDeleting(false);
    }
  }, [variable, onSuccess, onOpenChange, t]);

  if (!variable) return null;

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-destructive" />
            {t("hub.envVariables.deleteTitle")}
          </AlertDialogTitle>
          <AlertDialogDescription asChild>
            <div className="space-y-3">
              <p>
                {t("hub.envVariables.deleteConfirm", { name: variable.name })}
              </p>

              {/* 受影响的服务列表 */}
              {isLoading ? (
                <div className="flex items-center justify-center py-4">
                  <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
                </div>
              ) : affectedServices.length > 0 ? (
                <div className="rounded-md border bg-muted/50 p-3">
                  <div className="flex items-center gap-2 text-sm font-medium text-foreground mb-2">
                    <Server className="h-4 w-4" />
                    {t("hub.envVariables.affectedServices", {
                      count: affectedServices.length,
                    })}
                  </div>
                  <ul className="space-y-1">
                    {affectedServices.map((service) => (
                      <li
                        key={service.id}
                        className="text-sm text-muted-foreground flex items-center gap-2"
                      >
                        <span className="w-2 h-2 rounded-full bg-muted-foreground/50" />
                        <span className="font-mono">{service.name}</span>
                        {!service.enabled && (
                          <span className="text-xs text-muted-foreground">
                            ({t("hub.envVariables.disabled")})
                          </span>
                        )}
                      </li>
                    ))}
                  </ul>
                  <p className="text-xs text-muted-foreground mt-2">
                    {t("hub.envVariables.affectedServicesHint")}
                  </p>
                </div>
              ) : (
                <p className="text-sm text-muted-foreground">
                  {t("hub.envVariables.noAffectedServices")}
                </p>
              )}
            </div>
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={isDeleting}>
            {t("common.cancel")}
          </AlertDialogCancel>
          <AlertDialogAction
            onClick={handleDelete}
            disabled={isDeleting || isLoading}
            className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            data-testid="env-variable-confirm-delete"
          >
            {isDeleting && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
            {t("common.delete")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

export default EnvVariableDeleteDialog;
