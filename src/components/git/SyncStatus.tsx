/**
 * SyncStatus - 同步状态组件
 * Story 2.14: Task 7 - AC #11
 * Story 2.26: 国际化支持
 *
 * 功能:
 * - 显示四种同步状态: 已同步/同步中/有远程更新/离线
 * - 对应图标和颜色
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { Check, Loader2, AlertCircle, WifiOff } from "lucide-react";
import { cn } from "@/lib/utils";

/**
 * 同步状态类型
 */
export type SyncStatusType = "synced" | "syncing" | "behind" | "offline";

export interface SyncStatusProps {
    /** 同步状态 */
    status?: SyncStatusType;
    /** 自定义类名 */
    className?: string;
}

/**
 * 状态配置 (图标和样式)
 */
const STATUS_CONFIG: Record<SyncStatusType, {
    icon: React.ElementType;
    labelKey: string;
    className: string;
}> = {
    synced: {
        icon: Check,
        labelKey: "git.synced",
        className: "text-green-500",
    },
    syncing: {
        icon: Loader2,
        labelKey: "git.syncing",
        className: "text-blue-500",
    },
    behind: {
        icon: AlertCircle,
        labelKey: "git.hasRemoteUpdates",
        className: "text-amber-500",
    },
    offline: {
        icon: WifiOff,
        labelKey: "git.offline",
        className: "text-muted-foreground",
    },
};

/**
 * 同步状态组件
 */
export function SyncStatus({
    status = "synced",
    className,
}: SyncStatusProps) {
    const { t } = useTranslation();
    const config = STATUS_CONFIG[status];
    const Icon = config.icon;

    return (
        <div
            data-testid="sync-status"
            data-status={status}
            className={cn(
                "flex items-center gap-1 text-xs",
                config.className,
                className
            )}
        >
            <Icon
                className={cn(
                    "h-3 w-3",
                    status === "syncing" && "animate-spin"
                )}
            />
            <span>{t(config.labelKey)}</span>
        </div>
    );
}

export default SyncStatus;
