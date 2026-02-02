/**
 * 环境变量管理主组件
 * Story 11.4: 环境变量管理 - Task 4.1
 *
 * 集成变量列表、添加/编辑对话框和删除确认对话框
 */

import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { Key, Plus, RefreshCw, Loader2 } from "lucide-react";
import { EnvVariableList } from "./EnvVariableList";
import { EnvVariableSheet } from "./EnvVariableSheet";
import { EnvVariableDeleteDialog } from "./EnvVariableDeleteDialog";
import { listEnvVariables, type EnvVariable } from "@/lib/env-variable-ipc";

export function EnvVariableManager() {
  const { t } = useTranslation();
  const [variables, setVariables] = useState<EnvVariable[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");

  // 对话框状态
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [editVariable, setEditVariable] = useState<EnvVariable | null>(null);
  const [deleteVariable, setDeleteVariable] = useState<EnvVariable | null>(null);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);

  // 加载变量列表
  const loadVariables = useCallback(async () => {
    setIsLoading(true);
    try {
      const vars = await listEnvVariables();
      setVariables(vars);
    } catch (error) {
      console.error("[EnvVariableManager] Failed to load variables:", error);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    loadVariables();
  }, [loadVariables]);

  // 打开添加对话框
  const handleAdd = useCallback(() => {
    setEditVariable(null);
    setIsDialogOpen(true);
  }, []);

  // 打开编辑对话框
  const handleEdit = useCallback((variable: EnvVariable) => {
    setEditVariable(variable);
    setIsDialogOpen(true);
  }, []);

  // 打开删除确认对话框
  const handleDelete = useCallback((variable: EnvVariable) => {
    setDeleteVariable(variable);
    setIsDeleteDialogOpen(true);
  }, []);

  // 操作成功后刷新列表
  const handleSuccess = useCallback(() => {
    loadVariables();
  }, [loadVariables]);

  return (
    <div className="space-y-4">
      {/* 标题栏 */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="p-2 rounded-md bg-amber-500/10">
            <Key className="h-5 w-5 text-amber-500" />
          </div>
          <div>
            <h3 className="text-sm font-medium">{t("hub.envVariables.title")}</h3>
            <p className="text-xs text-muted-foreground">
              {t("hub.envVariables.description")}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={loadVariables}
            disabled={isLoading}
            title={t("common.refresh")}
          >
            <RefreshCw className={`h-4 w-4 ${isLoading ? "animate-spin" : ""}`} />
          </Button>
          <Button
            size="sm"
            onClick={handleAdd}
            data-testid="env-variable-add-button"
          >
            <Plus className="h-4 w-4 mr-1" />
            {t("hub.envVariables.add")}
          </Button>
        </div>
      </div>

      {/* 变量列表 */}
      {isLoading ? (
        <div className="flex items-center justify-center py-8">
          <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        </div>
      ) : (
        <EnvVariableList
          variables={variables}
          searchQuery={searchQuery}
          onSearchChange={setSearchQuery}
          onEdit={handleEdit}
          onDelete={handleDelete}
        />
      )}

      {/* 统计信息 */}
      {!isLoading && variables.length > 0 && (
        <div className="text-xs text-muted-foreground text-center">
          {t("hub.envVariables.count", { count: variables.length })}
        </div>
      )}

      {/* 添加/编辑 Sheet */}
      <EnvVariableSheet
        open={isDialogOpen}
        onOpenChange={setIsDialogOpen}
        editVariable={editVariable}
        onSuccess={handleSuccess}
      />

      {/* 删除确认对话框 */}
      <EnvVariableDeleteDialog
        open={isDeleteDialogOpen}
        onOpenChange={setIsDeleteDialogOpen}
        variable={deleteVariable}
        onSuccess={handleSuccess}
      />
    </div>
  );
}

export default EnvVariableManager;
