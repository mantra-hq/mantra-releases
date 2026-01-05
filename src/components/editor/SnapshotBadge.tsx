/**
 * SnapshotBadge - 历史状态徽章组件
 * Story 2.14: Task 1 - AC #2, #3, #4, #5
 * Story 2.26: 国际化支持
 *
 * 功能:
 * - 显示历史状态类型标识
 * - 两种模式：Tab 图标模式 和 Breadcrumb Pill 模式
 * - 会话快照: Camera 图标 + 蓝色样式
 * - Git 历史: GitCommit 图标 + 琥珀色样式
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { Camera, GitCommit } from "lucide-react";
import { cn } from "@/lib/utils";

/**
 * 历史状态类型
 * - snapshot: 会话快照 (来自时间旅行)
 * - git-history: Git 历史 (来自 commit)
 */
export type SnapshotType = "snapshot" | "git-history";

/**
 * 显示模式
 * - icon: 仅显示图标 (用于标签页)
 * - pill: 显示图标+文字的胶囊样式 (用于面包屑)
 */
export type SnapshotBadgeMode = "icon" | "pill";

export interface SnapshotBadgeProps {
    /** 历史类型 */
    type: SnapshotType;
    /** 显示模式 */
    mode: SnapshotBadgeMode;
    /** 时间戳 (快照模式, Unix ms) */
    timestamp?: number;
    /** Commit hash (Git 历史模式) */
    commitHash?: string;
    /** 相对时间 (Git 历史模式, 如 "3天前") */
    relativeTime?: string;
    /** 自定义类名 */
    className?: string;
}

/**
 * 样式配置
 */
const STYLES = {
    snapshot: {
        icon: "text-blue-500",
        pill: "bg-blue-500/10 text-blue-500",
    },
    "git-history": {
        icon: "text-amber-500",
        pill: "bg-amber-500/10 text-amber-500",
    },
} as const;

/**
 * 格式化快照时间为 HH:MM 格式
 */
function formatSnapshotTime(timestamp: number, locale: string): string {
    const date = new Date(timestamp);
    return date.toLocaleTimeString(locale, {
        hour: "2-digit",
        minute: "2-digit",
    });
}

/**
 * 历史状态徽章组件
 */
export function SnapshotBadge({
    type,
    mode,
    timestamp,
    commitHash,
    relativeTime,
    className,
}: SnapshotBadgeProps) {
    const { i18n } = useTranslation();
    const styles = STYLES[type];
    const Icon = type === "snapshot" ? Camera : GitCommit;

    // 图标模式：仅渲染图标
    if (mode === "icon") {
        return (
            <Icon
                data-testid={`snapshot-badge-icon-${type}`}
                className={cn("h-3 w-3 flex-shrink-0", styles.icon, className)}
            />
        );
    }

    // Pill 模式：渲染图标+文字胶囊
    const pillContent = React.useMemo(() => {
        if (type === "snapshot" && timestamp) {
            // 会话快照: 📸 10:32
            return formatSnapshotTime(timestamp, i18n.language);
        }
        if (type === "git-history" && commitHash) {
            // Git 历史: 🔖 abc1234 · 3天前
            const shortHash = commitHash.slice(0, 7);
            return relativeTime ? `${shortHash} · ${relativeTime}` : shortHash;
        }
        return null;
    }, [type, timestamp, commitHash, relativeTime, i18n.language]);

    if (!pillContent) return null;

    return (
        <span
            data-testid={`snapshot-badge-pill-${type}`}
            className={cn(
                "inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium",
                styles.pill,
                className
            )}
        >
            <Icon className="h-3 w-3" />
            <span>{pillContent}</span>
        </span>
    );
}

export default SnapshotBadge;
