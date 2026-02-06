/**
 * 配置差异对比组件
 * Story 11.13: Task 2 - 冲突差异对比 (AC: #2)
 *
 * 并排展示已有配置和候选配置的关键字段对比，差异部分高亮显示。
 * 支持 Stdio 和 HTTP 两种传输类型。
 */

import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ui/badge";
import { cn } from "@/lib/utils";

// ===== 类型定义 =====

/** MCP 传输类型 */
type McpTransportType = "stdio" | "http";

interface McpServiceConfig {
  transport_type?: McpTransportType;
  // Stdio 类型字段
  command?: string;
  args?: string[] | null;
  env?: Record<string, string> | null;
  // HTTP 类型字段
  url?: string | null;
  headers?: Record<string, string> | null;
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

  // 根据传输类型确定要显示的字段
  // 优先从候选配置判断，因为候选配置更可能有 transport_type 字段
  const isHttpType = candidates.some(c => c.transport_type === "http") ||
    (candidates.length > 0 && candidates[0].url && !candidates[0].command);

  const fields: { key: keyof McpServiceConfig; label: string }[] = isHttpType
    ? [
        { key: "url", label: t("hub.import.diffUrl", "URL") },
        { key: "headers", label: t("hub.import.diffHeaders", "Headers") },
        { key: "env", label: t("hub.import.diffEnv") },
      ]
    : [
        { key: "command", label: t("hub.import.diffCommand") },
        { key: "args", label: t("hub.import.diffArgs") },
        { key: "env", label: t("hub.import.diffEnv") },
      ];

  const columnCount = (existing ? 1 : 0) + candidates.length;
  // 计算数据列的最小宽度：单列时 300px，多列时 200px
  const dataColMinWidth = columnCount === 1 ? "300px" : "200px";

  return (
    <div
      className="border rounded-lg overflow-x-auto"
      data-testid={`config-diff-${serviceName}`}
    >
      {/* 表格容器 - 使用 min-width 确保列不会过窄 */}
      <div className="min-w-fit">
        {/* 表头 */}
        <div
          className="grid gap-0 border-b bg-muted/50"
          style={{ gridTemplateColumns: `minmax(80px, auto) repeat(${columnCount}, minmax(${dataColMinWidth}, 1fr))` }}
        >
          <div className="p-2 text-xs font-medium text-muted-foreground border-r whitespace-nowrap">
            {t("hub.import.diffField")}
          </div>
          {existing && (
            <div className="p-2 text-xs font-medium border-r">
              <Badge variant="secondary" className="text-xs whitespace-nowrap">
                {t("hub.import.diffExisting")}
              </Badge>
            </div>
          )}
          {candidates.map((candidate, idx) => (
            <div key={idx} className="p-2 text-xs font-medium border-r last:border-r-0">
              <Badge variant="outline" className="text-xs whitespace-nowrap">
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
              style={{ gridTemplateColumns: `minmax(80px, auto) repeat(${columnCount}, minmax(${dataColMinWidth}, 1fr))` }}
            >
              <div className="p-2 text-xs font-medium text-muted-foreground border-r bg-muted/30 whitespace-nowrap">
                {field.label}
              </div>
              {existing && (
                <div className="p-2 text-xs border-r min-w-0">
                  <code className="break-all text-[11px] leading-relaxed block">{renderFieldValue(existingValue)}</code>
                </div>
              )}
              {candidates.map((candidate, idx) => {
                const candidateValue = candidate[field.key];
                const isDifferent = existing ? !valuesEqual(existingValue, candidateValue) : false;

                return (
                  <div
                    key={idx}
                    className={cn(
                      "p-2 text-xs border-r last:border-r-0 min-w-0",
                      isDifferent && "bg-amber-500/5"
                    )}
                  >
                    <code
                      className={cn(
                        "break-all text-[11px] leading-relaxed block",
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
    </div>
  );
}

export default ConfigDiffView;
