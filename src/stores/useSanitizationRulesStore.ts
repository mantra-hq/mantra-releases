/**
 * useSanitizationRulesStore - 自定义清洗规则状态管理
 * Story 3-3: Task 1 - AC #3, #5
 *
 * 管理用户自定义的脱敏规则:
 * - 规则 CRUD 操作
 * - 启用/禁用规则
 * - 持久化到 tauri-plugin-store
 */

import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { Store } from '@tauri-apps/plugin-store';
import type { CustomRule, RuleFormData } from '@/components/settings/types';

interface SanitizationRulesState {
    rules: CustomRule[];
    isLoading: boolean;
    error: string | null;

    // 动作
    addRule: (data: RuleFormData) => void;
    updateRule: (id: string, data: Partial<RuleFormData>) => void;
    deleteRule: (id: string) => void;
    toggleRule: (id: string) => void;
    importRules: (rules: CustomRule[]) => void;
    clearRules: () => void;
    getEnabledRules: () => CustomRule[];
}

// Tauri Store 单例
let storeInstance: Store | null = null;

async function getStore(): Promise<Store> {
    if (!storeInstance) {
        storeInstance = await Store.load('settings.json');
    }
    return storeInstance;
}

// Tauri Store 适配器
const tauriStorage = {
    getItem: async (name: string): Promise<string | null> => {
        try {
            const store = await getStore();
            const value = await store.get<string>(name);
            return value ?? null;
        } catch {
            // 降级到 localStorage (测试环境)
            return localStorage.getItem(name);
        }
    },
    setItem: async (name: string, value: string): Promise<void> => {
        try {
            const store = await getStore();
            await store.set(name, value);
            await store.save();
        } catch {
            // 降级到 localStorage (测试环境)
            localStorage.setItem(name, value);
        }
    },
    removeItem: async (name: string): Promise<void> => {
        try {
            const store = await getStore();
            await store.delete(name);
            await store.save();
        } catch {
            // 降级到 localStorage (测试环境)
            localStorage.removeItem(name);
        }
    },
};

export const useSanitizationRulesStore = create<SanitizationRulesState>()(
    persist(
        (set, get) => ({
            rules: [],
            isLoading: false,
            error: null,

            addRule: (data) => {
                const now = new Date().toISOString();
                const newRule: CustomRule = {
                    id: crypto.randomUUID(),
                    ...data,
                    enabled: true,
                    createdAt: now,
                    updatedAt: now,
                };
                set((state) => ({ rules: [...state.rules, newRule] }));
            },

            updateRule: (id, data) => {
                set((state) => ({
                    rules: state.rules.map((rule) =>
                        rule.id === id
                            ? { ...rule, ...data, updatedAt: new Date().toISOString() }
                            : rule
                    ),
                }));
            },

            deleteRule: (id) => {
                set((state) => ({
                    rules: state.rules.filter((rule) => rule.id !== id),
                }));
            },

            toggleRule: (id) => {
                set((state) => ({
                    rules: state.rules.map((rule) =>
                        rule.id === id
                            ? { ...rule, enabled: !rule.enabled, updatedAt: new Date().toISOString() }
                            : rule
                    ),
                }));
            },

            importRules: (importedRules) => {
                const now = new Date().toISOString();
                const normalizedRules = importedRules.map((rule) => ({
                    ...rule,
                    id: rule.id || crypto.randomUUID(),
                    createdAt: rule.createdAt || now,
                    updatedAt: now,
                }));
                set((state) => ({ rules: [...state.rules, ...normalizedRules] }));
            },

            clearRules: () => {
                set({ rules: [] });
            },

            getEnabledRules: () => {
                return get().rules.filter((rule) => rule.enabled);
            },
        }),
        {
            name: 'sanitization-rules',
            storage: createJSONStorage(() => tauriStorage),
        }
    )
);

export default useSanitizationRulesStore;
