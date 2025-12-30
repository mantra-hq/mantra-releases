/**
 * useDiffFadeOut - Diff 装饰器淡出控制 Hook
 * Story 2.7: Task 3 - AC #5
 *
 * 管理 Diff 装饰器的显示和 3 秒后自动淡出
 */

import { useState, useCallback, useRef, useEffect } from "react";

/**
 * useDiffFadeOut Hook 返回值
 */
export interface UseDiffFadeOutResult {
    /** 是否应该显示 Diff 装饰器 */
    shouldShow: boolean;
    /** 触发淡出 (显示后 3 秒消失) */
    triggerFadeOut: () => void;
    /** 取消淡出并立即隐藏 */
    cancelFadeOut: () => void;
}

/**
 * React Hook: 管理 Diff 装饰器的淡出
 *
 * @param fadeOutDelay - 淡出延迟 (毫秒)，默认 3000ms
 * @returns { shouldShow, triggerFadeOut, cancelFadeOut }
 */
export function useDiffFadeOut(fadeOutDelay = 3000): UseDiffFadeOutResult {
    const [shouldShow, setShouldShow] = useState(false);
    const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

    const triggerFadeOut = useCallback(() => {
        // 显示装饰器
        setShouldShow(true);

        // 清除之前的定时器
        if (timerRef.current) {
            clearTimeout(timerRef.current);
        }

        // 设置淡出定时器
        timerRef.current = setTimeout(() => {
            setShouldShow(false);
        }, fadeOutDelay);
    }, [fadeOutDelay]);

    // 清理定时器
    useEffect(() => {
        return () => {
            if (timerRef.current) {
                clearTimeout(timerRef.current);
            }
        };
    }, []);

    const cancelFadeOut = useCallback(() => {
        if (timerRef.current) {
            clearTimeout(timerRef.current);
            timerRef.current = null;
        }
        setShouldShow(false);
    }, []);

    return { shouldShow, triggerFadeOut, cancelFadeOut };
}

export default useDiffFadeOut;
