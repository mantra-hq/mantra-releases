/**
 * 环境变量管理 IPC - 环境变量通信模块
 * Story 11.4: 环境变量管理 - Task 4
 *
 * 提供环境变量的 CRUD 操作和相关功能
 */

import { invoke } from "./ipc-adapter";

/**
 * 环境变量（值已脱敏）
 */
export interface EnvVariable {
  /** 唯一标识符 */
  id: string;
  /** 变量名称 */
  name: string;
  /** 脱敏后的值 */
  masked_value: string;
  /** 变量描述 */
  description: string | null;
  /** 创建时间 */
  created_at: string;
  /** 更新时间 */
  updated_at: string;
}

/**
 * 设置环境变量请求
 */
export interface SetEnvVariableRequest {
  /** 变量名称 */
  name: string;
  /** 变量值（明文） */
  value: string;
  /** 变量描述 */
  description?: string;
}

/**
 * 环境变量名称校验结果
 */
export interface EnvVariableNameValidation {
  /** 是否有效 */
  is_valid: boolean;
  /** 格式化建议 */
  suggestion: string | null;
  /** 错误信息 */
  error_message: string | null;
}

/**
 * MCP 服务（简化版，用于显示受影响服务）
 */
export interface McpService {
  /** 唯一标识符 */
  id: string;
  /** 服务名称 */
  name: string;
  /** 启动命令 */
  command: string;
  /** 是否启用 */
  enabled: boolean;
}

/**
 * 获取环境变量列表（值已脱敏）
 *
 * @returns 环境变量列表
 */
export async function listEnvVariables(): Promise<EnvVariable[]> {
  return invoke<EnvVariable[]>("list_env_variables");
}

/**
 * 设置环境变量
 *
 * @param name - 变量名称
 * @param value - 变量值（明文）
 * @param description - 变量描述
 * @returns 创建/更新的环境变量
 */
export async function setEnvVariable(
  name: string,
  value: string,
  description?: string
): Promise<EnvVariable> {
  return invoke<EnvVariable>("set_env_variable", { name, value, description });
}

/**
 * 删除环境变量
 *
 * @param name - 变量名称
 */
export async function deleteEnvVariable(name: string): Promise<void> {
  return invoke<void>("delete_env_variable", { name });
}

/**
 * 检查环境变量是否存在
 *
 * @param name - 变量名称
 * @returns 是否存在
 */
export async function envVariableExists(name: string): Promise<boolean> {
  return invoke<boolean>("env_variable_exists", { name });
}

/**
 * 获取解密后的环境变量值（临时显示用）
 *
 * @param name - 变量名称
 * @returns 解密后的值，如果不存在则返回 null
 */
export async function getEnvVariableDecrypted(
  name: string
): Promise<string | null> {
  return invoke<string | null>("get_env_variable_decrypted", { name });
}

/**
 * 获取引用指定环境变量的 MCP 服务列表
 *
 * @param varName - 环境变量名称
 * @returns 引用该变量的服务列表
 */
export async function getAffectedMcpServices(
  varName: string
): Promise<McpService[]> {
  return invoke<McpService[]>("get_affected_mcp_services", { varName });
}

/**
 * 批量设置环境变量
 *
 * @param variables - 环境变量列表
 * @returns 创建/更新的环境变量列表
 */
export async function batchSetEnvVariables(
  variables: SetEnvVariableRequest[]
): Promise<EnvVariable[]> {
  return invoke<EnvVariable[]>("batch_set_env_variables", { variables });
}

/**
 * 校验环境变量名格式
 *
 * @param name - 待校验的变量名
 * @returns 校验结果
 */
export async function validateEnvVariableName(
  name: string
): Promise<EnvVariableNameValidation> {
  return invoke<EnvVariableNameValidation>("validate_env_variable_name", {
    name,
  });
}

/**
 * 前端变量名格式校验（同步版本）
 *
 * @param name - 待校验的变量名
 * @returns 校验结果
 */
export function validateEnvVarNameSync(name: string): EnvVariableNameValidation {
  // SCREAMING_SNAKE_CASE 格式：以大写字母开头，只包含大写字母、数字和下划线
  const re = /^[A-Z][A-Z0-9_]*$/;
  const isValid = re.test(name);

  if (isValid) {
    return {
      is_valid: true,
      suggestion: null,
      error_message: null,
    };
  }

  // 生成格式化建议
  let suggestion = name
    .toUpperCase()
    .replace(/-/g, "_")
    .replace(/ /g, "_")
    .replace(/[^A-Z0-9_]/g, "");

  // 确保以字母开头
  if (/^\d/.test(suggestion)) {
    suggestion = `VAR_${suggestion}`;
  } else if (suggestion === "") {
    suggestion = "VARIABLE_NAME";
  }

  return {
    is_valid: false,
    suggestion,
    error_message: "变量名必须为 SCREAMING_SNAKE_CASE 格式（大写字母、数字和下划线）",
  };
}
