/**
 * useImportStore - 导入状态管理
 * Story 2.9: Task 7 + UX Redesign
 *
 * 管理导入向导的所有状态:
 * - Modal 开关状态
 * - 当前步骤
 * - 选择的来源
 * - 发现的文件和选择状态
 * - 项目分组和展开状态
 * - 搜索过滤
 * - 导入进度
 * - 导入结果
 */

import { create } from "zustand";
import type { ImportStep, ImportSource, DiscoveredFile, ImportProgressData, ImportResult, ImportError } from "@/components/import";

/**
 * 导入状态接口
 */
export interface ImportState {
  // ======== 状态 ========
  /** Modal 是否打开 */
  isOpen: boolean;
  /** 当前步骤 */
  step: ImportStep;
  /** 选择的导入来源 */
  source: ImportSource | null;
  /** 发现的文件列表 */
  discoveredFiles: DiscoveredFile[];
  /** 已选择的文件路径集合 */
  selectedFiles: Set<string>;
  /** 展开的项目路径集合 */
  expandedProjects: Set<string>;
  /** 搜索查询 */
  searchQuery: string;
  /** 导入进度 */
  progress: ImportProgressData | null;
  /** 导入结果 */
  results: ImportResult[];
  /** 是否正在加载 */
  isLoading: boolean;
  /** 错误列表 */
  errors: ImportError[];

  // ======== Actions ========
  /** 打开 Modal */
  open: () => void;
  /** 关闭 Modal */
  close: () => void;
  /** 设置当前步骤 */
  setStep: (step: ImportStep) => void;
  /** 设置来源 */
  setSource: (source: ImportSource) => void;
  /** 设置发现的文件 (默认全选) */
  setDiscoveredFiles: (files: DiscoveredFile[]) => void;
  /** 切换单个文件选择 */
  toggleFile: (path: string) => void;
  /** 全选所有文件 */
  selectAll: () => void;
  /** 清空所有选择 */
  clearAll: () => void;
  /** 反选 */
  invertSelection: () => void;
  /** 切换项目下所有文件的选择 */
  toggleProject: (projectPath: string) => void;
  /** 切换项目展开状态 */
  toggleProjectExpand: (projectPath: string) => void;
  /** 设置搜索查询 */
  setSearchQuery: (query: string) => void;
  /** 设置进度 */
  setProgress: (progress: ImportProgressData) => void;
  /** 添加结果 */
  addResult: (result: ImportResult) => void;
  /** 设置加载状态 */
  setLoading: (loading: boolean) => void;
  /** 添加错误 */
  addError: (error: ImportError) => void;
  /** 重置状态 */
  reset: () => void;
}

/**
 * 初始状态
 */
const initialState = {
  isOpen: false,
  step: "source" as ImportStep,
  source: null as ImportSource | null,
  discoveredFiles: [] as DiscoveredFile[],
  selectedFiles: new Set<string>(),
  expandedProjects: new Set<string>(),
  searchQuery: "",
  progress: null as ImportProgressData | null,
  results: [] as ImportResult[],
  isLoading: false,
  errors: [] as ImportError[],
};

/**
 * 导入状态 Store
 */
export const useImportStore = create<ImportState>((set) => ({
  ...initialState,

  open: () =>
    set({
      isOpen: true,
      step: "source",
    }),

  close: () =>
    set({
      isOpen: false,
    }),

  setStep: (step) =>
    set({
      step,
    }),

  setSource: (source) =>
    set({
      source,
    }),

  setDiscoveredFiles: (files) =>
    set({
      discoveredFiles: files,
      selectedFiles: new Set(files.map((f) => f.path)),
      expandedProjects: new Set<string>(),
      searchQuery: "",
    }),

  toggleFile: (path) =>
    set((state) => {
      const newSelected = new Set(state.selectedFiles);
      if (newSelected.has(path)) {
        newSelected.delete(path);
      } else {
        newSelected.add(path);
      }
      return { selectedFiles: newSelected };
    }),

  selectAll: () =>
    set((state) => ({
      selectedFiles: new Set(state.discoveredFiles.map((f) => f.path)),
    })),

  clearAll: () =>
    set({
      selectedFiles: new Set(),
    }),

  invertSelection: () =>
    set((state) => {
      const allPaths = state.discoveredFiles.map((f) => f.path);
      const newSelected = new Set<string>();
      for (const path of allPaths) {
        if (!state.selectedFiles.has(path)) {
          newSelected.add(path);
        }
      }
      return { selectedFiles: newSelected };
    }),

  toggleProject: (projectPath) =>
    set((state) => {
      const projectFiles = state.discoveredFiles.filter(
        (f) => f.projectPath === projectPath
      );
      const projectFilePaths = projectFiles.map((f) => f.path);

      // 检查项目下所有文件是否都已选中
      const allSelected = projectFilePaths.every((p) =>
        state.selectedFiles.has(p)
      );

      const newSelected = new Set(state.selectedFiles);

      if (allSelected) {
        // 取消选择项目下所有文件
        for (const path of projectFilePaths) {
          newSelected.delete(path);
        }
      } else {
        // 选择项目下所有文件
        for (const path of projectFilePaths) {
          newSelected.add(path);
        }
      }

      return { selectedFiles: newSelected };
    }),

  toggleProjectExpand: (projectPath) =>
    set((state) => {
      const newExpanded = new Set(state.expandedProjects);
      if (newExpanded.has(projectPath)) {
        newExpanded.delete(projectPath);
      } else {
        newExpanded.add(projectPath);
      }
      return { expandedProjects: newExpanded };
    }),

  setSearchQuery: (query) =>
    set({
      searchQuery: query,
    }),

  setProgress: (progress) =>
    set({
      progress,
    }),

  addResult: (result) =>
    set((state) => ({
      results: [...state.results, result],
    })),

  setLoading: (loading) =>
    set({
      isLoading: loading,
    }),

  addError: (error) =>
    set((state) => ({
      errors: [...state.errors, error],
    })),

  reset: () =>
    set({
      step: "source",
      source: null,
      discoveredFiles: [],
      selectedFiles: new Set(),
      expandedProjects: new Set(),
      searchQuery: "",
      progress: null,
      results: [],
      isLoading: false,
      errors: [],
    }),
}));

export default useImportStore;

