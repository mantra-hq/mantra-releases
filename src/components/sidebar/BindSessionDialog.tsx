/**
 * BindSessionDialog Component - 会话绑定对话框
 * Story 1.12: Task 5.4 - 会话手动绑定
 *
 * 允许用户将未分类会话手动绑定到指定项目
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import {
    Dialog,
    DialogContent,
    DialogHeader,
    DialogTitle,
    DialogDescription,
    DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import { Loader2, Link2, Unlink } from "lucide-react";
import { bindSessionToProject, unbindSession } from "@/hooks/useProjects";
import { toast } from "sonner";
import type { Project } from "@/types/project";
import type { Session } from "@/types/project";

/**
 * BindSessionDialog Props
 */
export interface BindSessionDialogProps {
    /** 是否打开 */
    isOpen: boolean;
    /** 打开状态变化回调 */
    onOpenChange: (open: boolean) => void;
    /** 要绑定的会话 */
    session: Session | null;
    /** 可选的项目列表 */
    projects: Project[];
    /** 当前绑定的项目 ID (如果已绑定) */
    currentProjectId?: string;
    /** 绑定成功回调 */
    onBindSuccess?: () => void;
}

/**
 * BindSessionDialog 组件
 * 显示会话绑定对话框
 */
export function BindSessionDialog({
    isOpen,
    onOpenChange,
    session,
    projects,
    currentProjectId,
    onBindSuccess,
}: BindSessionDialogProps) {
    const { t } = useTranslation();
    const [selectedProjectId, setSelectedProjectId] = React.useState<string>("");
    const [isBinding, setIsBinding] = React.useState(false);
    const [isUnbinding, setIsUnbinding] = React.useState(false);

    // 当对话框打开时，设置当前绑定的项目
    React.useEffect(() => {
        if (isOpen && currentProjectId) {
            setSelectedProjectId(currentProjectId);
        } else if (isOpen) {
            setSelectedProjectId("");
        }
    }, [isOpen, currentProjectId]);

    /**
     * 处理绑定
     */
    const handleBind = async () => {
        if (!session || !selectedProjectId) return;

        setIsBinding(true);
        try {
            await bindSessionToProject(session.id, selectedProjectId);
            const projectName = projects.find(p => p.id === selectedProjectId)?.name || selectedProjectId;
            toast.success(t("session.bindSuccess", "会话已绑定到 {{project}}", { project: projectName }));
            onBindSuccess?.();
            onOpenChange(false);
        } catch (error) {
            console.error("Failed to bind session:", error);
            toast.error(
                t("session.bindFailed", "绑定失败: {{error}}", {
                    error: error instanceof Error ? error.message : String(error),
                })
            );
        } finally {
            setIsBinding(false);
        }
    };

    /**
     * 处理解绑
     */
    const handleUnbind = async () => {
        if (!session) return;

        setIsUnbinding(true);
        try {
            await unbindSession(session.id);
            toast.success(t("session.unbindSuccess", "会话绑定已解除"));
            onBindSuccess?.();
            onOpenChange(false);
        } catch (error) {
            console.error("Failed to unbind session:", error);
            toast.error(
                t("session.unbindFailed", "解绑失败: {{error}}", {
                    error: error instanceof Error ? error.message : String(error),
                })
            );
        } finally {
            setIsUnbinding(false);
        }
    };

    if (!session) return null;

    const sessionDisplayName = session.title || session.id.slice(0, 8);
    const isAlreadyBound = !!currentProjectId;

    return (
        <Dialog open={isOpen} onOpenChange={onOpenChange}>
            <DialogContent size="md">
                <DialogHeader>
                    <DialogTitle className="flex items-center gap-2">
                        <Link2 className="h-5 w-5" />
                        {t("session.bindToProject", "绑定到项目")}
                    </DialogTitle>
                    <DialogDescription>
                        {t("session.bindDescription", "将会话 \"{{session}}\" 手动绑定到指定项目", {
                            session: sessionDisplayName,
                        })}
                    </DialogDescription>
                </DialogHeader>

                <div className="py-4">
                    <label className="text-sm font-medium mb-2 block">
                        {t("session.selectProject", "选择项目")}
                    </label>
                    <Select
                        value={selectedProjectId}
                        onValueChange={setSelectedProjectId}
                    >
                        <SelectTrigger className="w-full">
                            <SelectValue placeholder={t("session.selectProjectPlaceholder", "选择一个项目...")} />
                        </SelectTrigger>
                        <SelectContent>
                            {projects.map((project) => (
                                <SelectItem key={project.id} value={project.id}>
                                    {project.name}
                                </SelectItem>
                            ))}
                        </SelectContent>
                    </Select>

                    {isAlreadyBound && (
                        <p className="text-xs text-muted-foreground mt-2">
                            {t("session.currentlyBoundTo", "当前已绑定到: {{project}}", {
                                project: projects.find(p => p.id === currentProjectId)?.name || currentProjectId,
                            })}
                        </p>
                    )}
                </div>

                <DialogFooter className="flex gap-2">
                    {isAlreadyBound && (
                        <Button
                            variant="outline"
                            onClick={handleUnbind}
                            disabled={isUnbinding || isBinding}
                            className="text-destructive hover:text-destructive"
                        >
                            {isUnbinding ? (
                                <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                            ) : (
                                <Unlink className="h-4 w-4 mr-2" />
                            )}
                            {t("session.unbind", "解除绑定")}
                        </Button>
                    )}
                    <Button
                        onClick={handleBind}
                        disabled={!selectedProjectId || isBinding || isUnbinding}
                    >
                        {isBinding ? (
                            <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                        ) : (
                            <Link2 className="h-4 w-4 mr-2" />
                        )}
                        {t("session.bind", "绑定")}
                    </Button>
                </DialogFooter>
            </DialogContent>
        </Dialog>
    );
}
