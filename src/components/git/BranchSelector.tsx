/**
 * BranchSelector - 分支选择器组件
 * Story 2.14: Task 6 - AC #10
 *
 * 功能:
 * - 显示当前分支名
 * - 点击展开分支列表下拉菜单
 * - MVP: 只显示，不实现切换功能
 */

import { GitBranch, ChevronDown } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
    DropdownMenu,
    DropdownMenuContent,
    DropdownMenuItem,
    DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";

export interface Branch {
    /** 分支名 */
    name: string;
    /** 是否当前分支 */
    isCurrent: boolean;
}

export interface BranchSelectorProps {
    /** 当前分支名 */
    currentBranch?: string;
    /** 分支列表 */
    branches?: Branch[];
    /** 是否加载中 */
    isLoading?: boolean;
    /** 自定义类名 */
    className?: string;
}

/**
 * 分支选择器组件
 */
export function BranchSelector({
    currentBranch = "main",
    branches = [],
    isLoading = false,
    className,
}: BranchSelectorProps) {
    return (
        <DropdownMenu>
            <DropdownMenuTrigger asChild>
                <Button
                    variant="ghost"
                    size="sm"
                    data-testid="branch-selector"
                    className={cn(
                        "h-5 px-2 gap-1 text-xs font-normal",
                        "hover:bg-muted",
                        className
                    )}
                    disabled={isLoading}
                >
                    <GitBranch className="h-3 w-3" />
                    <span data-testid="current-branch">{isLoading ? "加载中..." : currentBranch}</span>
                    <ChevronDown className="h-3 w-3 opacity-50" />
                </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="start" className="min-w-[150px]">
                {branches.length === 0 ? (
                    <DropdownMenuItem disabled>
                        无其他分支
                    </DropdownMenuItem>
                ) : (
                    branches.map((branch) => (
                        <DropdownMenuItem
                            key={branch.name}
                            className={cn(branch.isCurrent && "bg-accent")}
                        >
                            <GitBranch className="h-3 w-3 mr-2" />
                            {branch.name}
                        </DropdownMenuItem>
                    ))
                )}
            </DropdownMenuContent>
        </DropdownMenu>
    );
}

export default BranchSelector;
