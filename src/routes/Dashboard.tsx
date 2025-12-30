/**
 * Dashboard Page - Dashboard 页面
 * Story 2.8: Task 1, Task 8
 *
 * 项目列表主页面，聚合展示所有项目及其会话
 */

import * as React from "react";
import { useNavigate } from "react-router-dom";
import { DashboardHeader, EmptyDashboard } from "@/components/layout";
import { ProjectCard } from "@/components/cards";
import { useProjects, useDebouncedValue } from "@/hooks";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

/** 骨架屏显示的卡片数量 */
const SKELETON_COUNT = 6;

/**
 * ProjectCardSkeleton - 项目卡片骨架屏
 */
function ProjectCardSkeleton() {
  return (
    <div className="bg-card border border-border rounded-lg overflow-hidden">
      <div className="flex items-center justify-between p-4">
        <div className="flex items-center gap-3">
          <Skeleton className="w-10 h-10 rounded-lg" />
          <div className="space-y-2">
            <Skeleton className="h-4 w-32" />
            <Skeleton className="h-3 w-24" />
          </div>
        </div>
        <Skeleton className="w-5 h-5" />
      </div>
    </div>
  );
}

/**
 * DashboardSkeleton - Dashboard 骨架屏
 */
function DashboardSkeleton() {
  return (
    <div
      className={cn(
        "grid gap-4 p-6",
        "grid-cols-1 md:grid-cols-2 xl:grid-cols-3"
      )}
    >
      {Array.from({ length: SKELETON_COUNT }, (_, i) => (
        <ProjectCardSkeleton key={i} />
      ))}
    </div>
  );
}

/**
 * NoSearchResults - 搜索无结果
 */
function NoSearchResults({ query }: { query: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 px-4 text-center">
      <p className="text-lg text-muted-foreground mb-2">
        未找到匹配的项目
      </p>
      <p className="text-sm text-muted-foreground/70">
        没有找到包含 "{query}" 的项目
      </p>
    </div>
  );
}

/**
 * Dashboard 页面组件
 * 展示项目列表，支持搜索和过滤
 */
export default function Dashboard() {
  const navigate = useNavigate();
  
  // 项目数据
  const { projects, isLoading, error, refetch } = useProjects();
  
  // 搜索状态
  const [searchQuery, setSearchQuery] = React.useState("");
  const debouncedQuery = useDebouncedValue(searchQuery, 300);
  
  // 展开状态 (记录哪些项目被展开)
  const [expandedProjects, setExpandedProjects] = React.useState<Set<string>>(
    new Set()
  );

  // 过滤项目
  const filteredProjects = React.useMemo(() => {
    if (!debouncedQuery.trim()) return projects;
    const query = debouncedQuery.toLowerCase();
    return projects.filter(
      (p) =>
        p.name.toLowerCase().includes(query) ||
        p.path.toLowerCase().includes(query)
    );
  }, [projects, debouncedQuery]);

  // 搜索处理
  const handleSearch = React.useCallback((query: string) => {
    setSearchQuery(query);
  }, []);

  // 导入处理 (Story 2-9 实现)
  const handleImport = React.useCallback(() => {
    // TODO: Story 2-9 实现日志导入功能
    // 当前为占位符，等待导入界面实现后替换
  }, []);

  // 切换项目展开状态
  const toggleProject = React.useCallback((projectId: string) => {
    setExpandedProjects((prev) => {
      const next = new Set(prev);
      if (next.has(projectId)) {
        next.delete(projectId);
      } else {
        next.add(projectId);
      }
      return next;
    });
  }, []);

  // 点击会话进入 Player
  const handleSessionClick = React.useCallback(
    (sessionId: string) => {
      navigate(`/session/${sessionId}`);
    },
    [navigate]
  );

  // 错误状态
  if (error) {
    return (
      <div className="min-h-screen bg-background flex flex-col">
        <DashboardHeader onSearch={handleSearch} onImport={handleImport} />
        <div className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <p className="text-destructive mb-4">{error}</p>
            <button
              onClick={() => refetch()}
              className="text-primary hover:underline"
            >
              重试
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background flex flex-col">
      {/* Header */}
      <DashboardHeader onSearch={handleSearch} onImport={handleImport} />

      {/* Content */}
      <main className="flex-1">
        {/* Loading State */}
        {isLoading && <DashboardSkeleton />}

        {/* Empty State */}
        {!isLoading && projects.length === 0 && (
          <EmptyDashboard onImport={handleImport} />
        )}

        {/* No Search Results */}
        {!isLoading &&
          projects.length > 0 &&
          filteredProjects.length === 0 &&
          debouncedQuery.trim() && (
            <NoSearchResults query={debouncedQuery} />
          )}

        {/* Project List */}
        {!isLoading && filteredProjects.length > 0 && (
          <div
            className={cn(
              "grid gap-4 p-6",
              "grid-cols-1 md:grid-cols-2 xl:grid-cols-3"
            )}
          >
            {filteredProjects.map((project) => (
              <ProjectCard
                key={project.id}
                project={project}
                isExpanded={expandedProjects.has(project.id)}
                onToggle={() => toggleProject(project.id)}
                onSessionClick={handleSessionClick}
              />
            ))}
          </div>
        )}
      </main>
    </div>
  );
}
