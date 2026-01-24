/**
 * 规则导入/导出工具
 * Story 3-3: Task 7 - AC #7
 */

import { save, open } from '@tauri-apps/plugin-dialog';
import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs';
import type { CustomRule, RuleExportData } from '@/components/settings/types';

/** 导出版本 */
const EXPORT_VERSION = '1.0.0';

/**
 * 导出规则到 JSON 文件
 * @param rules 要导出的规则
 */
export async function exportRules(rules: CustomRule[]): Promise<boolean> {
    const exportData: RuleExportData = {
        version: EXPORT_VERSION,
        exportedAt: new Date().toISOString(),
        rules: rules.map(({ id: _id, createdAt: _createdAt, updatedAt: _updatedAt, ...rest }) => rest),
    };

    const filePath = await save({
        title: '导出清洗规则',
        defaultPath: 'sanitization-rules.json',
        filters: [
            { name: '规则文件', extensions: ['json'] },
        ],
    });

    if (!filePath) return false;

    await writeTextFile(filePath, JSON.stringify(exportData, null, 2));
    return true;
}

/**
 * 从 JSON 文件导入规则
 * @returns 导入的规则数组，如果取消则返回 null
 */
export async function importRules(): Promise<CustomRule[] | null> {
    const filePath = await open({
        title: '导入清洗规则',
        filters: [
            { name: '规则文件', extensions: ['json'] },
        ],
        multiple: false,
    });

    if (!filePath || typeof filePath !== 'string') return null;

    const content = await readTextFile(filePath);
    const data = JSON.parse(content) as RuleExportData;

    // 验证版本
    if (!data.version) {
        throw new Error('无效的规则文件格式');
    }

    // 验证规则数据
    if (!Array.isArray(data.rules)) {
        throw new Error('规则文件不包含有效的规则数据');
    }

    // 转换为完整的 CustomRule
    const now = new Date().toISOString();
    return data.rules.map((rule) => ({
        ...rule,
        id: crypto.randomUUID(),
        enabled: rule.enabled ?? true,
        createdAt: now,
        updatedAt: now,
    })) as CustomRule[];
}

/**
 * 验证导入的规则
 * @param rules 要验证的规则
 */
export function validateImportedRules(rules: unknown[]): string[] {
    const errors: string[] = [];

    if (!Array.isArray(rules)) {
        errors.push('规则数据必须是数组');
        return errors;
    }

    rules.forEach((rule, index) => {
        if (typeof rule !== 'object' || rule === null) {
            errors.push(`规则 #${index + 1} 不是有效的对象`);
            return;
        }

        const r = rule as Record<string, unknown>;
        if (!r.name || typeof r.name !== 'string') {
            errors.push(`规则 #${index + 1} 缺少有效的名称`);
        }
        if (!r.pattern || typeof r.pattern !== 'string') {
            errors.push(`规则 #${index + 1} 缺少有效的正则表达式`);
        }
    });

    return errors;
}
