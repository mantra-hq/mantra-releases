/**
 * DevelopmentSettings Tests - 开发环境设置页面测试
 * Story 2-35: Task 3.2
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import { DevelopmentSettings } from "./DevelopmentSettings";

// Mock child components
vi.mock("@/components/settings/LocalServerConfig", () => ({
  LocalServerConfig: () => <div data-testid="local-server-config">LocalServerConfig</div>,
}));

vi.mock("@/components/settings/ToolConfigPathManager", () => ({
  ToolConfigPathManager: () => <div data-testid="tool-config-path-manager">ToolConfigPathManager</div>,
}));

vi.mock("@/components/hub/EnvVariableManager", () => ({
  EnvVariableManager: () => <div data-testid="env-variable-manager">EnvVariableManager</div>,
}));

describe("DevelopmentSettings", () => {
  it("renders LocalServerConfig component", () => {
    render(<DevelopmentSettings />);
    expect(screen.getByTestId("local-server-config")).toBeInTheDocument();
  });

  it("renders ToolConfigPathManager component", () => {
    render(<DevelopmentSettings />);
    expect(screen.getByTestId("tool-config-path-manager")).toBeInTheDocument();
  });

  it("renders EnvVariableManager component", () => {
    render(<DevelopmentSettings />);
    expect(screen.getByTestId("env-variable-manager")).toBeInTheDocument();
  });

  it("renders all 3 sections in correct order", () => {
    const { container } = render(<DevelopmentSettings />);
    const sections = container.querySelectorAll("section");
    expect(sections).toHaveLength(3);

    expect(sections[0]).toContainElement(screen.getByTestId("local-server-config"));
    expect(sections[1]).toContainElement(screen.getByTestId("tool-config-path-manager"));
    expect(sections[2]).toContainElement(screen.getByTestId("env-variable-manager"));
  });
});
