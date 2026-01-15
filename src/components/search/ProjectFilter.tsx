/**
 * ProjectFilter - 项目筛选器
 * Story 2.33: Task 3.3
 *
 * AC2: 下拉列表显示所有已导入项目，支持"全部项目"选项
 * 使用 shadcn/ui Select 组件
 */

import * as React from "react";
import { useTranslation } from "react-i18next";
import { FolderOpen, ChevronDown, Check } from "lucide-react";
import { cn } from "@/lib/utils";
import { useProjects } from "@/hooks/useProjects";

export interface ProjectFilterProps {
    /** 当前选中的项目 ID (null = 全部项目) */
    value: string | null;
    /** 项目变化回调 */
    onChange: (value: string | null) => void;
}

/**
 * 项目筛选器组件
 */
export function ProjectFilter({ value, onChange }: ProjectFilterProps) {
    const { t } = useTranslation();
    const { projects, isLoading } = useProjects();
    const [isOpen, setIsOpen] = React.useState(false);
    const containerRef = React.useRef<HTMLDivElement>(null);

    // 获取当前选中项目的名称
    const selectedProject = value ? projects.find((p) => p.id === value) : null;
    const displayName = selectedProject
        ? selectedProject.name
        : t("search.filters.allProjects");

    // 点击外部关闭下拉
    React.useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (
                containerRef.current &&
                !containerRef.current.contains(event.target as Node)
            ) {
                setIsOpen(false);
            }
        };

        if (isOpen) {
            document.addEventListener("mousedown", handleClickOutside);
            return () => {
                document.removeEventListener("mousedown", handleClickOutside);
            };
        }
    }, [isOpen]);

    // 键盘导航
    const handleKeyDown = (e: React.KeyboardEvent) => {
        if (e.key === "Escape") {
            setIsOpen(false);
        } else if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            setIsOpen(!isOpen);
        }
    };

    return (
        <div ref={containerRef} className="relative">
            <button
                type="button"
                onClick={() => setIsOpen(!isOpen)}
                onKeyDown={handleKeyDown}
                disabled={isLoading}
                className={cn(
                    "flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-xs",
                    "bg-muted/50 hover:bg-muted transition-colors",
                    "border border-transparent hover:border-border",
                    "focus:outline-none focus:ring-2 focus:ring-primary focus:ring-offset-1",
                    isLoading && "opacity-50 cursor-not-allowed"
                )}
                aria-expanded={isOpen}
                aria-haspopup="listbox"
            >
                <FolderOpen className="w-3.5 h-3.5 text-muted-foreground" />
                <span
                    className="max-w-[100px] sm:max-w-[140px] md:max-w-[180px] truncate text-foreground"
                    title={displayName}
                >
                    {displayName}
                </span>
                <ChevronDown
                    className={cn(
                        "w-3.5 h-3.5 text-muted-foreground transition-transform",
                        isOpen && "rotate-180"
                    )}
                />
            </button>

            {/* Dropdown Menu */}
            {isOpen && (
                <div
                    role="listbox"
                    className={cn(
                        "absolute top-full left-0 mt-1 z-50",
                        "min-w-[200px] max-w-[280px] max-h-[240px]",
                        "bg-popover border border-border rounded-md shadow-lg",
                        "overflow-y-auto",
                        "animate-in fade-in-0 zoom-in-95"
                    )}
                >
                    {/* All Projects Option */}
                    <button
                        type="button"
                        role="option"
                        aria-selected={value === null}
                        onClick={() => {
                            onChange(null);
                            setIsOpen(false);
                        }}
                        className={cn(
                            "flex items-center justify-between w-full px-3 py-2 text-sm",
                            "hover:bg-accent transition-colors",
                            value === null && "bg-accent"
                        )}
                    >
                        <span>{t("search.filters.allProjects")}</span>
                        {value === null && (
                            <Check className="w-4 h-4 text-primary" />
                        )}
                    </button>

                    {/* Divider */}
                    {projects.length > 0 && <div className="h-px bg-border" />}

                    {/* Project List */}
                    {projects.map((project) => (
                        <button
                            key={project.id}
                            type="button"
                            role="option"
                            aria-selected={value === project.id}
                            onClick={() => {
                                onChange(project.id);
                                setIsOpen(false);
                            }}
                            className={cn(
                                "flex items-center justify-between w-full px-3 py-2 text-sm",
                                "hover:bg-accent transition-colors",
                                value === project.id && "bg-accent"
                            )}
                        >
                            <span className="truncate" title={project.name}>
                                {project.name}
                            </span>
                            {value === project.id && (
                                <Check className="w-4 h-4 text-primary shrink-0 ml-2" />
                            )}
                        </button>
                    ))}

                    {/* Empty State */}
                    {projects.length === 0 && !isLoading && (
                        <div className="px-3 py-4 text-sm text-muted-foreground text-center">
                            {t("search.filters.noProjects")}
                        </div>
                    )}
                </div>
            )}
        </div>
    );
}

export default ProjectFilter;
