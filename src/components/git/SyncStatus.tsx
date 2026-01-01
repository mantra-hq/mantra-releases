/**
 * SyncStatus - 同步状态组件
 * Story 2.14: Task 7 - AC #11
 *
 * 功能:
 * - 显示四种同步状态: 已同步/同步中/有远程更新/离线
 * - 对应图标和颜色
 */

import * as React from "react";
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
 * 状态配置
 */
const STATUS_CONFIG: Record<SyncStatusType, {
    icon: React.ElementType;
    label: string;
    className: string;
}> = {
    synced: {
        icon: Check,
        label: "已同步",
        className: "text-green-500",
    },
    syncing: {
        icon: Loader2,
        label: "同步中",
        className: "text-blue-500",
    },
    behind: {
        icon: AlertCircle,
        label: "有远程更新",
        className: "text-amber-500",
    },
    offline: {
        icon: WifiOff,
        label: "离线",
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
            <span>{config.label}</span>
        </div>
    );
}

export default SyncStatus;
