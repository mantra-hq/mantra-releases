/**
 * MCP 服务删除确认对话框
 * Story 11.6: Task 4.6 - 删除服务功能（带确认对话框）
 *
 * 删除服务前的确认对话框
 */

import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@/lib/ipc-adapter";
import { Button } from "@/components/ui/button";
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
import { Loader2, AlertTriangle } from "lucide-react";
import { feedback } from "@/lib/feedback";
import type { McpService } from "./McpServiceList";

interface McpServiceDeleteDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  service: McpService | null;
  onSuccess: () => void;
}

export function McpServiceDeleteDialog({
  open,
  onOpenChange,
  service,
  onSuccess,
}: McpServiceDeleteDialogProps) {
  const { t } = useTranslation();
  const [isDeleting, setIsDeleting] = useState(false);

  const handleDelete = useCallback(async () => {
    if (!service) return;

    setIsDeleting(true);
    try {
      await invoke("delete_mcp_service", { id: service.id });
      feedback.success(t("hub.services.deleteSuccess", { name: service.name }));
      onOpenChange(false);
      onSuccess();
    } catch (error) {
      console.error("[McpServiceDeleteDialog] Failed to delete:", error);
      feedback.error(t("hub.services.deleteError"), (error as Error).message);
    } finally {
      setIsDeleting(false);
    }
  }, [service, onOpenChange, onSuccess, t]);

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-destructive" />
            <AlertDialogTitle>{t("hub.services.deleteTitle")}</AlertDialogTitle>
          </div>
          <AlertDialogDescription>
            {t("hub.services.deleteConfirm", { name: service?.name })}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={isDeleting}>
            {t("common.cancel")}
          </AlertDialogCancel>
          <AlertDialogAction asChild>
            <Button
              variant="destructive"
              onClick={handleDelete}
              disabled={isDeleting}
              data-testid="mcp-service-delete-confirm"
            >
              {isDeleting && <Loader2 className="h-4 w-4 mr-2 animate-spin" />}
              {t("common.delete")}
            </Button>
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

export default McpServiceDeleteDialog;
