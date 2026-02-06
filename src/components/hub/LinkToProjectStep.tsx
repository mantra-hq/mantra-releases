/**
 * 关联到项目步骤组件
 * Story 11.29: Task 3 - 导入完成后引导用户将服务关联到项目
 *
 * AC #1: 显示"关联到项目"步骤，列出所有可关联的服务，默认全选
 * AC #2: 显示服务名称、来源工具图标、已关联状态
 * AC #6: 所有服务已关联时显示提示
 */

import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { Checkbox } from "@/components/ui/checkbox";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { CheckCircle2, Info } from "lucide-react";
import { SourceIcon } from "@/components/import/SourceIcons";
import { cn } from "@/lib/utils";

// ===== 类型定义 =====

/** 可关联服务信息 */
export interface LinkableService {
  id: string;
  name: string;
  adapterId: string;
  alreadyLinked: boolean;
}

interface LinkToProjectStepProps {
  /** 可关联的服务列表 */
  services: LinkableService[];
  /** 项目名称 */
  projectName?: string;
  /** 当前选中的服务 ID */
  selectedIds: Set<string>;
  /** 选择变更回调 */
  onSelectionChange: (ids: Set<string>) => void;
  /** 是否所有服务都已关联 */
  allLinked: boolean;
}

export function LinkToProjectStep({
  services,
  projectName,
  selectedIds,
  onSelectionChange,
  allLinked,
}: LinkToProjectStepProps) {
  const { t } = useTranslation();

  const linkableServices = useMemo(
    () => services.filter((s) => !s.alreadyLinked),
    [services]
  );

  // AC6: 所有服务已关联时的展示
  if (allLinked) {
    return (
      <div className="flex flex-col items-center py-8 space-y-4">
        <CheckCircle2 className="h-12 w-12 text-green-500" />
        <p className="text-lg font-medium">
          {t("hub.import.allLinkedTitle")}
        </p>
        <div className="space-y-2 w-full max-w-sm">
          {services.map((service) => (
            <div
              key={service.id}
              className="flex items-center gap-3 p-3 border rounded-lg bg-muted/30"
            >
              <SourceIcon source={service.adapterId} className="h-5 w-5 shrink-0" />
              <span className="text-sm flex-1">{service.name}</span>
              <Badge
                variant="outline"
                className="text-xs bg-green-500/10 text-green-500 border-green-500/20"
              >
                {t("hub.import.alreadyLinked")}
              </Badge>
            </div>
          ))}
        </div>
      </div>
    );
  }

  // AC1, AC2: 服务选择列表
  const toggleService = (id: string, checked: boolean) => {
    const next = new Set(selectedIds);
    if (checked) {
      next.add(id);
    } else {
      next.delete(id);
    }
    onSelectionChange(next);
  };

  return (
    <div className="flex flex-col h-full space-y-4">
      {/* 描述文本 */}
      <p className="shrink-0 text-sm text-muted-foreground">
        {projectName
          ? t("hub.import.linkDescription", { project: projectName })
          : t("hub.import.linkDescriptionGeneric")}
      </p>

      {/* 服务列表 */}
      <ScrollArea className="flex-1 min-h-0 pr-4">
        <div className="space-y-2">
          {services.map((service) => (
            <div
              key={service.id}
              className={cn(
                "flex items-center gap-3 p-3 border rounded-lg bg-muted/30 cursor-pointer hover:bg-muted/50 transition-colors",
                service.alreadyLinked && "opacity-60"
              )}
              onClick={() =>
                !service.alreadyLinked &&
                toggleService(service.id, !selectedIds.has(service.id))
              }
              data-testid={`link-service-${service.name}`}
            >
              <Checkbox
                checked={service.alreadyLinked || selectedIds.has(service.id)}
                disabled={service.alreadyLinked}
                onCheckedChange={(checked) =>
                  toggleService(service.id, checked as boolean)
                }
                onClick={(e: React.MouseEvent) => e.stopPropagation()}
                className="border-zinc-400 data-[state=unchecked]:bg-zinc-700/30"
                data-testid={`link-checkbox-${service.name}`}
              />
              <SourceIcon source={service.adapterId} className="h-5 w-5 shrink-0" />
              <span className="text-sm flex-1">{service.name}</span>
              {service.alreadyLinked && (
                <Badge
                  variant="outline"
                  className="text-xs bg-green-500/10 text-green-500 border-green-500/20"
                >
                  {t("hub.import.alreadyLinked")}
                </Badge>
              )}
            </div>
          ))}
        </div>
      </ScrollArea>

      {/* 提示 */}
      <Alert className="shrink-0 bg-muted/50">
        <Info className="h-4 w-4" />
        <AlertDescription className="text-xs">
          {t("hub.import.linkHint")}
        </AlertDescription>
      </Alert>

      {/* 选择控制 */}
      <div className="shrink-0 flex items-center justify-between text-sm text-muted-foreground">
        <span>
          {t("hub.import.linkSelectedCount", { count: selectedIds.size })}
        </span>
        <div className="flex gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={() =>
              onSelectionChange(new Set(linkableServices.map((s) => s.id)))
            }
          >
            {t("hub.import.selectAll")}
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onClick={() => onSelectionChange(new Set())}
          >
            {t("hub.import.selectNone")}
          </Button>
        </div>
      </div>
    </div>
  );
}

export default LinkToProjectStep;
