/**
 * Settings Types - Story 3-3
 * 自定义清洗规则相关类型定义
 */

import type { SensitiveType } from '@/components/sanitizer/types';

/** 自定义规则 (与 Rust SanitizationRule 保持一致) */
export interface CustomRule {
    /** 唯一标识符 */
    id: string;
    /** 规则名称 */
    name: string;
    /** 正则表达式模式 */
    pattern: string;
    /** 敏感信息类型 */
    sensitiveType: SensitiveType;
    /** 是否启用 */
    enabled: boolean;
    /** 创建时间 */
    createdAt: string;
    /** 更新时间 */
    updatedAt: string;
}

/** 规则编辑表单 */
export interface RuleFormData {
    name: string;
    pattern: string;
    sensitiveType: SensitiveType;
}

/** 规则验证结果 */
export interface ValidationResult {
    valid: boolean;
    error?: string;
}

/** 规则导入/导出格式 */
export interface RuleExportData {
    version: string;
    exportedAt: string;
    rules: Omit<CustomRule, 'id' | 'createdAt' | 'updatedAt'>[];
}
