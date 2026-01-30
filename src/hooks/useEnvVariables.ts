/**
 * 环境变量管理 Hook
 * Story 11.4: 环境变量管理 - Task 6.2
 *
 * 提供环境变量的状态管理和操作方法
 */

import { useState, useEffect, useCallback } from "react";
import {
  listEnvVariables,
  setEnvVariable,
  deleteEnvVariable,
  getEnvVariableDecrypted,
  getAffectedMcpServices,
  type EnvVariable,
  type McpService,
} from "@/lib/env-variable-ipc";

interface UseEnvVariablesReturn {
  /** 环境变量列表 */
  variables: EnvVariable[];
  /** 是否正在加载 */
  isLoading: boolean;
  /** 错误信息 */
  error: string | null;
  /** 刷新列表 */
  refresh: () => Promise<void>;
  /** 添加/更新变量 */
  saveVariable: (
    name: string,
    value: string,
    description?: string
  ) => Promise<EnvVariable>;
  /** 删除变量 */
  removeVariable: (name: string) => Promise<void>;
  /** 获取解密后的值 */
  getDecryptedValue: (name: string) => Promise<string | null>;
  /** 获取受影响的服务 */
  getAffectedServices: (varName: string) => Promise<McpService[]>;
}

export function useEnvVariables(): UseEnvVariablesReturn {
  const [variables, setVariables] = useState<EnvVariable[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // 加载变量列表
  const refresh = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const vars = await listEnvVariables();
      setVariables(vars);
    } catch (err) {
      console.error("[useEnvVariables] Failed to load variables:", err);
      setError((err as Error).message);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // 初始加载
  useEffect(() => {
    refresh();
  }, [refresh]);

  // 保存变量
  const saveVariable = useCallback(
    async (
      name: string,
      value: string,
      description?: string
    ): Promise<EnvVariable> => {
      const result = await setEnvVariable(name, value, description);
      await refresh();
      return result;
    },
    [refresh]
  );

  // 删除变量
  const removeVariable = useCallback(
    async (name: string): Promise<void> => {
      await deleteEnvVariable(name);
      await refresh();
    },
    [refresh]
  );

  // 获取解密后的值
  const getDecryptedValue = useCallback(
    async (name: string): Promise<string | null> => {
      return getEnvVariableDecrypted(name);
    },
    []
  );

  // 获取受影响的服务
  const getAffectedServices = useCallback(
    async (varName: string): Promise<McpService[]> => {
      return getAffectedMcpServices(varName);
    },
    []
  );

  return {
    variables,
    isLoading,
    error,
    refresh,
    saveVariable,
    removeVariable,
    getDecryptedValue,
    getAffectedServices,
  };
}

export default useEnvVariables;
