/**
 * useSanitizationRulesStore 测试
 * Story 3-3: Task 1.5
 */

import { describe, it, expect, beforeEach, vi } from 'vitest';
import { useSanitizationRulesStore } from './useSanitizationRulesStore';
import type { RuleFormData } from '@/components/settings/types';

// Mock tauri plugin-store
vi.mock('@tauri-apps/plugin-store', () => ({
    Store: {
        load: vi.fn().mockResolvedValue({
            get: vi.fn().mockResolvedValue(null),
            set: vi.fn().mockResolvedValue(undefined),
            delete: vi.fn().mockResolvedValue(undefined),
            save: vi.fn().mockResolvedValue(undefined),
        }),
    },
}));

// Mock crypto.randomUUID
vi.stubGlobal('crypto', {
    randomUUID: vi.fn(() => 'test-uuid-' + Math.random().toString(36).substr(2, 9)),
});

describe('useSanitizationRulesStore', () => {
    beforeEach(() => {
        // 重置 store 状态
        useSanitizationRulesStore.setState({ rules: [], isLoading: false, error: null });
    });

    describe('addRule', () => {
        it('should add a new rule with generated id and timestamps', () => {
            const formData: RuleFormData = {
                name: 'Company Email',
                pattern: '\\w+@company\\.com',
                sensitiveType: 'custom',
            };

            useSanitizationRulesStore.getState().addRule(formData);

            const { rules } = useSanitizationRulesStore.getState();
            expect(rules).toHaveLength(1);
            expect(rules[0]).toMatchObject({
                name: 'Company Email',
                pattern: '\\w+@company\\.com',
                sensitiveType: 'custom',
                enabled: true,
            });
            expect(rules[0].id).toBeDefined();
            expect(rules[0].createdAt).toBeDefined();
            expect(rules[0].updatedAt).toBeDefined();
        });

        it('should add multiple rules', () => {
            useSanitizationRulesStore.getState().addRule({
                name: 'Rule 1',
                pattern: 'pattern1',
                sensitiveType: 'api_key',
            });
            useSanitizationRulesStore.getState().addRule({
                name: 'Rule 2',
                pattern: 'pattern2',
                sensitiveType: 'secret',
            });

            const { rules } = useSanitizationRulesStore.getState();
            expect(rules).toHaveLength(2);
        });
    });

    describe('updateRule', () => {
        it('should update rule name', () => {
            // Add a rule first
            useSanitizationRulesStore.getState().addRule({
                name: 'Old Name',
                pattern: 'pattern',
                sensitiveType: 'custom',
            });

            const ruleId = useSanitizationRulesStore.getState().rules[0].id;
            useSanitizationRulesStore.getState().updateRule(ruleId, { name: 'New Name' });

            const rule = useSanitizationRulesStore.getState().rules[0];
            expect(rule.name).toBe('New Name');
            expect(rule.pattern).toBe('pattern'); // unchanged
        });

        it('should update updatedAt timestamp', async () => {
            useSanitizationRulesStore.getState().addRule({
                name: 'Test',
                pattern: 'pattern',
                sensitiveType: 'custom',
            });

            const originalUpdatedAt = useSanitizationRulesStore.getState().rules[0].updatedAt;

            // Wait a tiny bit to ensure different timestamp
            await new Promise((r) => setTimeout(r, 5));

            const ruleId = useSanitizationRulesStore.getState().rules[0].id;
            useSanitizationRulesStore.getState().updateRule(ruleId, { pattern: 'new-pattern' });

            const newUpdatedAt = useSanitizationRulesStore.getState().rules[0].updatedAt;
            expect(newUpdatedAt).not.toBe(originalUpdatedAt);
        });
    });

    describe('deleteRule', () => {
        it('should delete a rule by id', () => {
            useSanitizationRulesStore.getState().addRule({
                name: 'To Delete',
                pattern: 'pattern',
                sensitiveType: 'custom',
            });

            const ruleId = useSanitizationRulesStore.getState().rules[0].id;
            expect(useSanitizationRulesStore.getState().rules).toHaveLength(1);

            useSanitizationRulesStore.getState().deleteRule(ruleId);

            expect(useSanitizationRulesStore.getState().rules).toHaveLength(0);
        });

        it('should not affect other rules', () => {
            useSanitizationRulesStore.getState().addRule({
                name: 'Rule 1',
                pattern: 'pattern1',
                sensitiveType: 'custom',
            });
            useSanitizationRulesStore.getState().addRule({
                name: 'Rule 2',
                pattern: 'pattern2',
                sensitiveType: 'custom',
            });

            const ruleId = useSanitizationRulesStore.getState().rules[0].id;
            useSanitizationRulesStore.getState().deleteRule(ruleId);

            const { rules } = useSanitizationRulesStore.getState();
            expect(rules).toHaveLength(1);
            expect(rules[0].name).toBe('Rule 2');
        });
    });

    describe('toggleRule', () => {
        it('should toggle rule enabled state', () => {
            useSanitizationRulesStore.getState().addRule({
                name: 'Toggle Test',
                pattern: 'pattern',
                sensitiveType: 'custom',
            });

            const ruleId = useSanitizationRulesStore.getState().rules[0].id;
            expect(useSanitizationRulesStore.getState().rules[0].enabled).toBe(true);

            useSanitizationRulesStore.getState().toggleRule(ruleId);
            expect(useSanitizationRulesStore.getState().rules[0].enabled).toBe(false);

            useSanitizationRulesStore.getState().toggleRule(ruleId);
            expect(useSanitizationRulesStore.getState().rules[0].enabled).toBe(true);
        });
    });

    describe('importRules', () => {
        it('should import rules and merge with existing', () => {
            useSanitizationRulesStore.getState().addRule({
                name: 'Existing',
                pattern: 'existing',
                sensitiveType: 'custom',
            });

            useSanitizationRulesStore.getState().importRules([
                {
                    id: 'imported-1',
                    name: 'Imported 1',
                    pattern: 'imported1',
                    sensitiveType: 'api_key',
                    enabled: true,
                    createdAt: '2024-01-01T00:00:00Z',
                    updatedAt: '2024-01-01T00:00:00Z',
                },
                {
                    id: 'imported-2',
                    name: 'Imported 2',
                    pattern: 'imported2',
                    sensitiveType: 'secret',
                    enabled: false,
                    createdAt: '2024-01-01T00:00:00Z',
                    updatedAt: '2024-01-01T00:00:00Z',
                },
            ]);

            const { rules } = useSanitizationRulesStore.getState();
            expect(rules).toHaveLength(3);
            expect(rules[1].name).toBe('Imported 1');
            expect(rules[2].enabled).toBe(false);
        });
    });

    describe('getEnabledRules', () => {
        it('should return only enabled rules', () => {
            useSanitizationRulesStore.getState().addRule({
                name: 'Enabled 1',
                pattern: 'enabled1',
                sensitiveType: 'custom',
            });
            useSanitizationRulesStore.getState().addRule({
                name: 'Enabled 2',
                pattern: 'enabled2',
                sensitiveType: 'custom',
            });

            // Disable second rule
            const secondRuleId = useSanitizationRulesStore.getState().rules[1].id;
            useSanitizationRulesStore.getState().toggleRule(secondRuleId);

            const enabledRules = useSanitizationRulesStore.getState().getEnabledRules();
            expect(enabledRules).toHaveLength(1);
            expect(enabledRules[0].name).toBe('Enabled 1');
        });
    });

    describe('clearRules', () => {
        it('should remove all rules', () => {
            useSanitizationRulesStore.getState().addRule({
                name: 'Rule 1',
                pattern: 'pattern1',
                sensitiveType: 'custom',
            });
            useSanitizationRulesStore.getState().addRule({
                name: 'Rule 2',
                pattern: 'pattern2',
                sensitiveType: 'custom',
            });

            expect(useSanitizationRulesStore.getState().rules).toHaveLength(2);

            useSanitizationRulesStore.getState().clearRules();

            expect(useSanitizationRulesStore.getState().rules).toHaveLength(0);
        });
    });
});
