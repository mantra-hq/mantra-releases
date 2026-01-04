/**
 * RuleList - 规则列表组件
 * Story 3-3: Task 3 - AC #1, #5
 */

import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
} from '@/components/ui/dialog';
import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
    AlertDialogTrigger,
} from '@/components/ui/alert-dialog';
import { Plus, Pencil, Trash2, FileUp, FileDown, Shield } from 'lucide-react';
import { useSanitizationRulesStore } from '@/stores/useSanitizationRulesStore';
import { feedback } from '@/lib/feedback';
import { RuleEditor } from './RuleEditor';
import type { CustomRule, RuleFormData } from './types';

export interface RuleListProps {
    onImport?: () => void;
    onExport?: () => void;
}

export function RuleList({ onImport, onExport }: RuleListProps) {
    const { rules, addRule, updateRule, deleteRule, toggleRule } =
        useSanitizationRulesStore();
    const [isEditorOpen, setIsEditorOpen] = useState(false);
    const [editingRule, setEditingRule] = useState<CustomRule | null>(null);

    const handleAdd = () => {
        setEditingRule(null);
        setIsEditorOpen(true);
    };

    const handleEdit = (rule: CustomRule) => {
        setEditingRule(rule);
        setIsEditorOpen(true);
    };

    const handleSave = (data: RuleFormData) => {
        if (editingRule) {
            updateRule(editingRule.id, data);
            feedback.saved(data.name);
        } else {
            addRule(data);
            feedback.saved(data.name);
        }
        setIsEditorOpen(false);
        setEditingRule(null);
    };

    const handleDelete = (rule: CustomRule) => {
        deleteRule(rule.id);
        feedback.deleted(rule.name);
    };

    return (
        <div className="space-y-4" data-testid="rule-list">
            <div className="flex justify-between items-center">
                <h3 className="text-lg font-medium flex items-center gap-2">
                    <Shield className="h-5 w-5" />
                    自定义清洗规则
                </h3>
                <div className="flex gap-2">
                    {onImport && (
                        <Button variant="outline" size="sm" onClick={onImport} data-testid="import-button">
                            <FileUp className="h-4 w-4 mr-1" />
                            导入
                        </Button>
                    )}
                    {onExport && (
                        <Button
                            variant="outline"
                            size="sm"
                            onClick={onExport}
                            disabled={rules.length === 0}
                            data-testid="export-button"
                        >
                            <FileDown className="h-4 w-4 mr-1" />
                            导出
                        </Button>
                    )}
                    <Button size="sm" onClick={handleAdd} data-testid="add-rule-button">
                        <Plus className="h-4 w-4 mr-1" />
                        添加规则
                    </Button>
                </div>
            </div>

            {rules.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground border border-dashed rounded-lg" data-testid="empty-state">
                    <Shield className="h-12 w-12 mx-auto mb-4 opacity-50" />
                    <p className="font-medium">暂无自定义规则</p>
                    <p className="text-sm mt-1">
                        添加规则以匹配特定的敏感信息，如公司邮箱、内部域名等
                    </p>
                    <Button className="mt-4" onClick={handleAdd}>
                        <Plus className="h-4 w-4 mr-1" />
                        添加第一条规则
                    </Button>
                </div>
            ) : (
                <div className="space-y-2">
                    {rules.map((rule) => (
                        <div
                            key={rule.id}
                            className="flex items-center justify-between p-3 border rounded-lg bg-card hover:bg-accent/50 transition-colors"
                            data-testid={`rule-item-${rule.id}`}
                        >
                            <div className="flex-1 min-w-0">
                                <p className={`font-medium ${!rule.enabled ? 'text-muted-foreground' : ''}`}>
                                    {rule.name}
                                </p>
                                <p className="text-sm text-muted-foreground font-mono truncate max-w-md">
                                    {rule.pattern}
                                </p>
                            </div>
                            <div className="flex items-center gap-2 ml-4">
                                <Switch
                                    checked={rule.enabled}
                                    onCheckedChange={() => toggleRule(rule.id)}
                                    aria-label={rule.enabled ? '禁用规则' : '启用规则'}
                                    data-testid={`toggle-rule-${rule.id}`}
                                />
                                <Button
                                    variant="ghost"
                                    size="icon"
                                    onClick={() => handleEdit(rule)}
                                    aria-label="编辑规则"
                                    data-testid={`edit-rule-${rule.id}`}
                                >
                                    <Pencil className="h-4 w-4" />
                                </Button>
                                <AlertDialog>
                                    <AlertDialogTrigger asChild>
                                        <Button
                                            variant="ghost"
                                            size="icon"
                                            aria-label="删除规则"
                                            data-testid={`delete-rule-${rule.id}`}
                                        >
                                            <Trash2 className="h-4 w-4 text-destructive" />
                                        </Button>
                                    </AlertDialogTrigger>
                                    <AlertDialogContent>
                                        <AlertDialogHeader>
                                            <AlertDialogTitle>确认删除</AlertDialogTitle>
                                            <AlertDialogDescription>
                                                确定要删除规则 "{rule.name}" 吗？此操作无法撤销。
                                            </AlertDialogDescription>
                                        </AlertDialogHeader>
                                        <AlertDialogFooter>
                                            <AlertDialogCancel>取消</AlertDialogCancel>
                                            <AlertDialogAction
                                                onClick={() => handleDelete(rule)}
                                                data-testid={`confirm-delete-${rule.id}`}
                                            >
                                                删除
                                            </AlertDialogAction>
                                        </AlertDialogFooter>
                                    </AlertDialogContent>
                                </AlertDialog>
                            </div>
                        </div>
                    ))}
                </div>
            )}

            <Dialog open={isEditorOpen} onOpenChange={setIsEditorOpen}>
                <DialogContent>
                    <DialogHeader>
                        <DialogTitle>
                            {editingRule ? '编辑规则' : '添加规则'}
                        </DialogTitle>
                    </DialogHeader>
                    <RuleEditor
                        initialData={
                            editingRule
                                ? {
                                    name: editingRule.name,
                                    pattern: editingRule.pattern,
                                    sensitiveType: editingRule.sensitiveType,
                                }
                                : undefined
                        }
                        onSave={handleSave}
                        onCancel={() => setIsEditorOpen(false)}
                    />
                </DialogContent>
            </Dialog>
        </div>
    );
}

export default RuleList;
