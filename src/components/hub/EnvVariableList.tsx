/**
 * 环境变量列表组件
 * Story 11.4: 环境变量管理 - Task 4.2
 *
 * 显示环境变量列表，支持搜索过滤
 */

import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { Key, Search } from "lucide-react";
import { Input } from "@/components/ui/input";
import { EnvVariableItem } from "./EnvVariableItem";
import type { EnvVariable } from "@/lib/env-variable-ipc";

interface EnvVariableListProps {
  variables: EnvVariable[];
  searchQuery: string;
  onSearchChange: (query: string) => void;
  onEdit: (variable: EnvVariable) => void;
  onDelete: (variable: EnvVariable) => void;
}

export function EnvVariableList({
  variables,
  searchQuery,
  onSearchChange,
  onEdit,
  onDelete,
}: EnvVariableListProps) {
  const { t } = useTranslation();

  // 过滤变量
  const filteredVariables = useMemo(() => {
    if (!searchQuery.trim()) return variables;

    const query = searchQuery.toLowerCase();
    return variables.filter(
      (v) =>
        v.name.toLowerCase().includes(query) ||
        v.description?.toLowerCase().includes(query)
    );
  }, [variables, searchQuery]);

  return (
    <div className="space-y-3">
      {/* 搜索框 */}
      {variables.length > 3 && (
        <div className="relative">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            value={searchQuery}
            onChange={(e) => onSearchChange(e.target.value)}
            placeholder={t("hub.envVariables.searchPlaceholder")}
            className="pl-9"
            data-testid="env-variable-search"
          />
        </div>
      )}

      {/* 变量列表 */}
      {filteredVariables.length > 0 ? (
        <div className="space-y-2">
          {filteredVariables.map((variable) => (
            <EnvVariableItem
              key={variable.id}
              variable={variable}
              onEdit={onEdit}
              onDelete={onDelete}
            />
          ))}
        </div>
      ) : variables.length > 0 ? (
        // 有变量但搜索无结果
        <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
          <Search className="h-8 w-8 mb-2 opacity-50" />
          <p className="text-sm">{t("hub.envVariables.noSearchResults")}</p>
        </div>
      ) : (
        // 没有任何变量
        <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
          <Key className="h-8 w-8 mb-2 opacity-50" />
          <p className="text-sm">{t("hub.envVariables.empty")}</p>
          <p className="text-xs mt-1">{t("hub.envVariables.emptyHint")}</p>
        </div>
      )}
    </div>
  );
}

export default EnvVariableList;
