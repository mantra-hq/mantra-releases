/**
 * 配置差异对比组件
 * Story 11.13: Task 2 - 冲突差异对比 (AC: #2)
 *
 * 并排展示已有配置和候选配置的关键字段对比，差异部分高亮显示。
 */

import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

// ===== 类型定义 =====

interface McpServiceConfig {
  command: string;
  args: string[] | null;
  env: Record<string, string> | null;
}

interface DetectedService extends McpServiceConfig {
  name: string;
  source_file: string;
  adapter_id: string;
}

interface ConfigDiffViewProps {
  /** 服务名称 */
  serviceName: string;
  /** 已有配置 */
  existing: McpServiceConfig | null;
  /** 候选配置列表 */
  candidates: DetectedService[];
  /** 获取来源显示文本 */
  getSourceText: (adapterId: string) => string;
}

/** 比较两个值是否相同（对象属性顺序无关） */
function valuesEqual(a: unknown, b: unknown): boolean {
  if (a === b) return true;
  if (a === null || b === null) return a === b;
  if (typeof a !== typeof b) return false;
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) return false;
    return a.every((v, i) => valuesEqual(v, b[i]));
  }
  if (typeof a === "object" && typeof b === "object") {
    const keysA = Object.keys(a as object).sort();
    const keysB = Object.keys(b as object).sort();
    if (keysA.length !== keysB.length) return false;
    return keysA.every((k, i) =>
      k === keysB[i] && valuesEqual((a as Record<string, unknown>)[k], (b as Record<string, unknown>)[k])
    );
  }
  return false;
}

/** 渲染单个字段值 */
function renderFieldValue(value: unknown): string {
  if (value === null || value === undefined) {
    return "-";
  }
  if (Array.isArray(value)) {
    return value.length === 0 ? "[]" : JSON.stringify(value);
  }
  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>);
    return entries.length === 0 ? "{}" : JSON.stringify(value);
  }
  return String(value);
}

export function ConfigDiffView({
  serviceName,
  existing,
  candidates,
  getSourceText,
}: ConfigDiffViewProps) {
  const { t } = useTranslation();

  const fields: { key: keyof McpServiceConfig; label: string }[] = [
    { key: "command", label: t("hub.import.diffCommand") },
    { key: "args", label: t("hub.import.diffArgs") },
    { key: "env", label: t("hub.import.diffEnv") },
  ];

  const columnCount = (existing ? 1 : 0) + candidates.length;

  return (
    <div className="border rounded-lg overflow-hidden" data-testid={`config-diff-${serviceName}`}>
      {/* 表头 */}
      <div className="grid gap-0 border-b bg-muted/50" style={{ gridTemplateColumns: `140px repeat(${columnCount}, 1fr)` }}>
        <div className="p-2 text-xs font-medium text-muted-foreground border-r">
          {t("hub.import.diffField")}
        </div>
        {existing && (
          <div className="p-2 text-xs font-medium border-r">
            <Badge variant="secondary" className="text-xs">
              {t("hub.import.diffExisting")}
            </Badge>
          </div>
        )}
        {candidates.map((candidate, idx) => (
          <div key={idx} className="p-2 text-xs font-medium border-r last:border-r-0">
            <Badge variant="outline" className="text-xs">
              {t("hub.import.diffCandidate", { index: idx + 1 })} ({getSourceText(candidate.adapter_id)})
            </Badge>
          </div>
        ))}
      </div>

      {/* 字段行 */}
      {fields.map((field) => {
        const existingValue = existing ? existing[field.key] : null;

        return (
          <div
            key={field.key}
            className="grid gap-0 border-b last:border-b-0"
            style={{ gridTemplateColumns: `140px repeat(${columnCount}, 1fr)` }}
          >
            <div className="p-2 text-xs font-medium text-muted-foreground border-r bg-muted/30">
              {field.label}
            </div>
            {existing && (
              <div className="p-2 text-xs border-r">
                <code className="break-all">{renderFieldValue(existingValue)}</code>
              </div>
            )}
            {candidates.map((candidate, idx) => {
              const candidateValue = candidate[field.key];
              const isDifferent = existing ? !valuesEqual(existingValue, candidateValue) : false;

              return (
                <div
                  key={idx}
                  className={cn(
                    "p-2 text-xs border-r last:border-r-0",
                    isDifferent && "bg-amber-500/5"
                  )}
                >
                  <code
                    className={cn(
                      "break-all",
                      isDifferent && "text-amber-500 font-medium"
                    )}
                  >
                    {renderFieldValue(candidateValue)}
                  </code>
                </div>
              );
            })}
          </div>
        );
      })}
    </div>
  );
}

export default ConfigDiffView;
