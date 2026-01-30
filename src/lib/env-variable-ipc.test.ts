/**
 * 环境变量 IPC 测试
 * Story 11.4: 环境变量管理 - Task 7.3
 */

import { describe, it, expect } from "vitest";
import { validateEnvVarNameSync } from "./env-variable-ipc";

describe("validateEnvVarNameSync", () => {
  it("validates correct SCREAMING_SNAKE_CASE names", () => {
    expect(validateEnvVarNameSync("OPENAI_API_KEY").is_valid).toBe(true);
    expect(validateEnvVarNameSync("API_KEY").is_valid).toBe(true);
    expect(validateEnvVarNameSync("DATABASE_URL").is_valid).toBe(true);
    expect(validateEnvVarNameSync("MY_VAR_123").is_valid).toBe(true);
    expect(validateEnvVarNameSync("A").is_valid).toBe(true);
    expect(validateEnvVarNameSync("A1").is_valid).toBe(true);
  });

  it("rejects lowercase names", () => {
    const result = validateEnvVarNameSync("openai_api_key");
    expect(result.is_valid).toBe(false);
    expect(result.suggestion).toBe("OPENAI_API_KEY");
  });

  it("rejects names with hyphens", () => {
    const result = validateEnvVarNameSync("OPENAI-API-KEY");
    expect(result.is_valid).toBe(false);
    expect(result.suggestion).toBe("OPENAI_API_KEY");
  });

  it("rejects names with spaces", () => {
    const result = validateEnvVarNameSync("OPENAI API KEY");
    expect(result.is_valid).toBe(false);
    expect(result.suggestion).toBe("OPENAI_API_KEY");
  });

  it("rejects names starting with numbers", () => {
    const result = validateEnvVarNameSync("123_VAR");
    expect(result.is_valid).toBe(false);
    expect(result.suggestion).toBe("VAR_123_VAR");
  });

  it("rejects empty names", () => {
    const result = validateEnvVarNameSync("");
    expect(result.is_valid).toBe(false);
    expect(result.suggestion).toBe("VARIABLE_NAME");
  });

  it("rejects names with special characters", () => {
    const result = validateEnvVarNameSync("MY@VAR!");
    expect(result.is_valid).toBe(false);
    expect(result.suggestion).toBe("MYVAR");
  });

  it("provides error message for invalid names", () => {
    const result = validateEnvVarNameSync("invalid-name");
    expect(result.is_valid).toBe(false);
    expect(result.error_message).toBeTruthy();
    expect(result.error_message).toContain("SCREAMING_SNAKE_CASE");
  });

  it("returns null suggestion and error for valid names", () => {
    const result = validateEnvVarNameSync("VALID_NAME");
    expect(result.is_valid).toBe(true);
    expect(result.suggestion).toBeNull();
    expect(result.error_message).toBeNull();
  });

  it("handles mixed case names", () => {
    const result = validateEnvVarNameSync("OpenAI_Api_Key");
    expect(result.is_valid).toBe(false);
    expect(result.suggestion).toBe("OPENAI_API_KEY");
  });

  it("handles camelCase names", () => {
    const result = validateEnvVarNameSync("openaiApiKey");
    expect(result.is_valid).toBe(false);
    expect(result.suggestion).toBe("OPENAIAPIKEY");
  });
});
