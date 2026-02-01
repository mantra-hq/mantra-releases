/**
 * ImportStepper 组件测试
 * Story 11.13: Task 6 - 步骤指示器组件测试
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { ImportStepper } from "./ImportStepper";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const translations: Record<string, string> = {
        "hub.import.stepSelect": "选择服务",
        "hub.import.stepConflicts": "处理冲突",
        "hub.import.stepEnv": "环境变量",
        "hub.import.stepConfirm": "确认",
        "hub.import.stepExecute": "执行",
      };
      return translations[key] || key;
    },
  }),
}));

describe("ImportStepper", () => {
  it("应该渲染 data-testid", () => {
    render(
      <ImportStepper
        currentStep="preview"
        hasConflicts={false}
        needsEnvVars={false}
      />
    );

    expect(screen.getByTestId("import-stepper")).toBeInTheDocument();
  });

  it("应该渲染基本步骤（无冲突无环境变量）", () => {
    render(
      <ImportStepper
        currentStep="preview"
        hasConflicts={false}
        needsEnvVars={false}
      />
    );

    expect(screen.getByText("选择服务")).toBeInTheDocument();
    expect(screen.getByText("确认")).toBeInTheDocument();
    expect(screen.getByText("执行")).toBeInTheDocument();
    // 冲突和环境变量步骤不应该显示
    expect(screen.queryByText("处理冲突")).not.toBeInTheDocument();
    expect(screen.queryByText("环境变量")).not.toBeInTheDocument();
  });

  it("有冲突时应该显示冲突步骤", () => {
    render(
      <ImportStepper
        currentStep="preview"
        hasConflicts={true}
        needsEnvVars={false}
      />
    );

    expect(screen.getByText("处理冲突")).toBeInTheDocument();
  });

  it("需要环境变量时应该显示环境变量步骤", () => {
    render(
      <ImportStepper
        currentStep="preview"
        hasConflicts={false}
        needsEnvVars={true}
      />
    );

    expect(screen.getByText("环境变量")).toBeInTheDocument();
  });

  it("同时有冲突和环境变量时应该显示所有步骤", () => {
    render(
      <ImportStepper
        currentStep="preview"
        hasConflicts={true}
        needsEnvVars={true}
      />
    );

    expect(screen.getByText("选择服务")).toBeInTheDocument();
    expect(screen.getByText("处理冲突")).toBeInTheDocument();
    expect(screen.getByText("环境变量")).toBeInTheDocument();
    expect(screen.getByText("确认")).toBeInTheDocument();
    expect(screen.getByText("执行")).toBeInTheDocument();
  });

  it("应该显示步骤编号", () => {
    render(
      <ImportStepper
        currentStep="preview"
        hasConflicts={false}
        needsEnvVars={false}
      />
    );

    // 基本步骤：选择(1), 确认(2), 执行(3)
    expect(screen.getByText("1")).toBeInTheDocument();
    expect(screen.getByText("2")).toBeInTheDocument();
    expect(screen.getByText("3")).toBeInTheDocument();
  });

  it("当前步骤应该高亮（通过 class 检测）", () => {
    const { container } = render(
      <ImportStepper
        currentStep="preview"
        hasConflicts={false}
        needsEnvVars={false}
      />
    );

    // 第一个步骤（preview）应该是当前步骤，应该有 bg-blue-500
    const stepCircles = container.querySelectorAll(".bg-blue-500");
    expect(stepCircles.length).toBeGreaterThanOrEqual(1);
  });

  it("已完成步骤应该显示勾选标记（通过 SVG 检测）", () => {
    const { container } = render(
      <ImportStepper
        currentStep="confirm"
        hasConflicts={false}
        needsEnvVars={false}
      />
    );

    // preview 步骤已完成，应该有勾选标记 (Check icon)
    // bg-green-500 表示已完成
    const completedCircles = container.querySelectorAll(".bg-green-500");
    expect(completedCircles.length).toBeGreaterThanOrEqual(1);
  });

  it("execute 步骤应该正确渲染", () => {
    render(
      <ImportStepper
        currentStep="execute"
        hasConflicts={false}
        needsEnvVars={false}
      />
    );

    // 所有之前的步骤都应该完成
    expect(screen.getByText("执行")).toBeInTheDocument();
  });

  it("confirm 步骤应该正确渲染", () => {
    render(
      <ImportStepper
        currentStep="confirm"
        hasConflicts={true}
        needsEnvVars={true}
      />
    );

    // 当前步骤是 confirm，之前的步骤（preview, conflicts, env）应该完成
    expect(screen.getByText("确认")).toBeInTheDocument();
  });
});
