/**
 * UpdateCheckerContext - 更新检查共享上下文
 * Story 14.7 Code Review Fix: 消除 useUpdateChecker 重复实例化
 *
 * 在根级别调用一次 useUpdateChecker，通过 Context 共享给所有消费者
 * (UpdateNotificationBar, GeneralSettings 等)
 */

/* eslint-disable react-refresh/only-export-components */

import { createContext, useContext } from 'react';
import { useUpdateChecker, type UseUpdateCheckerResult } from '@/hooks/useUpdateChecker';

const UpdateCheckerContext = createContext<UseUpdateCheckerResult | null>(null);

export function UpdateCheckerProvider({ children }: { children: React.ReactNode }) {
    const value = useUpdateChecker();
    return (
        <UpdateCheckerContext.Provider value={value}>
            {children}
        </UpdateCheckerContext.Provider>
    );
}

export function useUpdateCheckerContext(): UseUpdateCheckerResult {
    const ctx = useContext(UpdateCheckerContext);
    if (!ctx) {
        throw new Error('useUpdateCheckerContext must be used within <UpdateCheckerProvider>');
    }
    return ctx;
}
