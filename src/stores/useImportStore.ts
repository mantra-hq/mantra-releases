/**
 * useImportStore - 导入状态管理
 * Story 2.9: Task 7 + UX Redesign
 * Story 2.20: Import Status Enhancement
 * Story 2.23: Import Progress + Quick Navigation
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
 * - 已导入项目路径 (Story 2.20)
 * - 刚导入的项目列表 (Story 2.23)
 * - 上次扫描的源 (Story 2.24)
 */

import { create } from "zustand";
import type { ImportStep, ImportSource, DiscoveredFile, ImportProgressData, ImportResult, ImportError } from "@/components/import";

/**
 * Story 2.23: 刚导入的项目信息
 */
export interface ImportedProject {
  /** 项目 ID */
  id: string;
  /** 项目名称 */
  name: string;
  /** 会话数量 */
  sessionCount: number;
  /** 第一个会话 ID (用于快速跳转) */
  firstSessionId: string;
}

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
  /** 已导入项目路径集合 (Story 2.20) */
  importedPaths: Set<string>;
  /** 刚导入的项目列表 (Story 2.23) */
  importedProjects: ImportedProject[];
  /** 上次扫描的源 (Story 2.24) */
  lastScannedSource: ImportSource | null;
  /** 跳过空会话 (Story 2.29) */
  skipEmptySessions: boolean;

  // ======== Actions ========
  /** 打开 Modal */
  open: () => void;
  /** 关闭 Modal */
  close: () => void;
  /** 设置当前步骤 */
  setStep: (step: ImportStep) => void;
  /** 设置来源 */
  setSource: (source: ImportSource) => void;
  /** 设置发现的文件 (默认全选新项目) */
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
  /** 设置已导入路径 (Story 2.20) */
  setImportedPaths: (paths: string[]) => void;
  /** 全选新项目 (Story 2.20) */
  selectAllNew: () => void;
  /** 添加导入的项目 (Story 2.23) */
  addImportedProject: (projectId: string, sessionId: string, projectName: string) => void;
  /** 清空导入的项目列表 (Story 2.23) */
  clearImportedProjects: () => void;
  /** 清空错误列表 (Story 2.23) */
  clearErrors: () => void;
  /** 合并重试结果 (Story 2.23) */
  mergeRetryResults: (newResults: ImportResult[]) => void;
  /** 设置上次扫描的源 (Story 2.24) */
  setLastScannedSource: (source: ImportSource | null) => void;
  /** 清除发现的文件（源变化时） (Story 2.24) */
  clearDiscoveredFiles: () => void;
  /** 设置跳过空会话 (Story 2.29) */
  setSkipEmptySessions: (skip: boolean) => void;
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
  importedPaths: new Set<string>(),
  importedProjects: [] as ImportedProject[],
  lastScannedSource: null as ImportSource | null,
  skipEmptySessions: true,
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
    set((state) => {
      // Story 2.20: 默认仅选中新项目的文件
      const selectableFiles = files.filter(
        (f) => !state.importedPaths.has(f.projectPath)
      );
      return {
        discoveredFiles: files,
        selectedFiles: new Set(selectableFiles.map((f) => f.path)),
        expandedProjects: new Set<string>(),
        searchQuery: "",
      };
    }),

  toggleFile: (path) =>
    set((state) => {
      // Story 2.20: 检查文件是否属于已导入项目
      const file = state.discoveredFiles.find((f) => f.path === path);
      if (file && state.importedPaths.has(file.projectPath)) {
        // 不允许选中已导入项目的文件
        return state;
      }

      const newSelected = new Set(state.selectedFiles);
      if (newSelected.has(path)) {
        newSelected.delete(path);
      } else {
        newSelected.add(path);
      }
      return { selectedFiles: newSelected };
    }),

  selectAll: () =>
    set((state) => {
      // Story 2.20: 全选也应该排除已导入项目
      const selectableFiles = state.discoveredFiles.filter(
        (f) => !state.importedPaths.has(f.projectPath)
      );
      return {
        selectedFiles: new Set(selectableFiles.map((f) => f.path)),
      };
    }),

  clearAll: () =>
    set({
      selectedFiles: new Set(),
    }),

  invertSelection: () =>
    set((state) => {
      // Story 2.20: 反选时排除已导入项目
      const selectableFiles = state.discoveredFiles.filter(
        (f) => !state.importedPaths.has(f.projectPath)
      );
      const newSelected = new Set<string>();
      for (const file of selectableFiles) {
        if (!state.selectedFiles.has(file.path)) {
          newSelected.add(file.path);
        }
      }
      return { selectedFiles: newSelected };
    }),

  toggleProject: (projectPath) =>
    set((state) => {
      // Story 2.20: 不允许切换已导入项目
      if (state.importedPaths.has(projectPath)) {
        return state;
      }

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
      importedProjects: [],
      lastScannedSource: null,
      // Note: importedPaths 不重置，保持已加载的数据
    }),

  // Story 2.20: 设置已导入路径
  setImportedPaths: (paths) =>
    set({
      importedPaths: new Set(paths),
    }),

  // Story 2.20: 全选新项目
  selectAllNew: () =>
    set((state) => {
      const newProjectFiles = state.discoveredFiles.filter(
        (f) => !state.importedPaths.has(f.projectPath)
      );
      return {
        selectedFiles: new Set(newProjectFiles.map((f) => f.path)),
      };
    }),

  // Story 2.23: 添加导入的项目
  addImportedProject: (projectId, sessionId, projectName) =>
    set((state) => {
      // 检查项目是否已存在
      const existingProject = state.importedProjects.find((p) => p.id === projectId);
      if (existingProject) {
        // 更新会话数量
        return {
          importedProjects: state.importedProjects.map((p) =>
            p.id === projectId
              ? { ...p, sessionCount: p.sessionCount + 1 }
              : p
          ),
        };
      }

      // 添加新项目（最多保留 10 个）
      const newProject: ImportedProject = {
        id: projectId,
        name: projectName,
        sessionCount: 1,
        firstSessionId: sessionId,
      };

      return {
        importedProjects: [newProject, ...state.importedProjects].slice(0, 10),
      };
    }),

  // Story 2.23: 清空导入的项目列表
  clearImportedProjects: () =>
    set({
      importedProjects: [],
    }),

  // Story 2.23: 清空错误列表
  clearErrors: () =>
    set({
      errors: [],
    }),

  // Story 2.23: 合并重试结果
  mergeRetryResults: (newResults) =>
    set((state) => {
      // 创建一个 map 用于快速查找原始结果
      const resultsMap = new Map(
        state.results.map((r) => [r.filePath, r])
      );

      // 用新结果替换失败的结果
      for (const newResult of newResults) {
        resultsMap.set(newResult.filePath, newResult);
      }

      // 更新错误列表：移除重试成功的文件
      const successfulPaths = new Set(
        newResults.filter((r) => r.success).map((r) => r.filePath)
      );
      const updatedErrors = state.errors.filter(
        (e) => !successfulPaths.has(e.filePath)
      );

      // 添加新的失败
      for (const newResult of newResults) {
        if (!newResult.success && newResult.error) {
          const existingError = updatedErrors.find((e) => e.filePath === newResult.filePath);
          if (!existingError) {
            updatedErrors.push({
              filePath: newResult.filePath,
              error: newResult.error,
              message: newResult.error,
            });
          }
        }
      }

      return {
        results: Array.from(resultsMap.values()),
        errors: updatedErrors,
      };
    }),

  // Story 2.24: 设置上次扫描的源
  setLastScannedSource: (source) =>
    set({
      lastScannedSource: source,
    }),

  // Story 2.24: 清除发现的文件（源变化时）
  clearDiscoveredFiles: () =>
    set({
      discoveredFiles: [],
      selectedFiles: new Set(),
      expandedProjects: new Set(),
      searchQuery: "",
    }),

  // Story 2.29: 设置跳过空会话
  setSkipEmptySessions: (skip) =>
    set({
      skipEmptySessions: skip,
    }),
}));

export default useImportStore;

