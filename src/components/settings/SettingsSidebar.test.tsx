/**
 * SettingsSidebar Tests - 设置侧边栏导航组件测试
 * Story 2-35: Task 1
 */

import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { SettingsSidebar } from "./SettingsSidebar";

const renderWithRouter = (ui: React.ReactElement, initialEntry = "/settings/general") => {
  return render(
    <MemoryRouter initialEntries={[initialEntry]}>{ui}</MemoryRouter>
  );
};

describe("SettingsSidebar", () => {
  it("renders nav element with correct test id", () => {
    renderWithRouter(<SettingsSidebar />);
    expect(screen.getByTestId("settings-sidebar")).toBeInTheDocument();
  });

  it("renders all 3 group NavLinks", () => {
    renderWithRouter(<SettingsSidebar />);

    expect(screen.getByTestId("settings-nav-general")).toBeInTheDocument();
    expect(screen.getByTestId("settings-nav-development")).toBeInTheDocument();
    expect(screen.getByTestId("settings-nav-privacy")).toBeInTheDocument();
  });

  it("renders group titles with i18n keys (zh-CN default)", () => {
    renderWithRouter(<SettingsSidebar />);

    expect(screen.getByText("通用")).toBeInTheDocument();
    expect(screen.getByText("开发环境")).toBeInTheDocument();
    expect(screen.getByText("隐私与安全")).toBeInTheDocument();
  });

  it("renders sub-item labels for each group", () => {
    renderWithRouter(<SettingsSidebar />);

    // General group items
    expect(screen.getByText("语言")).toBeInTheDocument();
    expect(screen.getByText("帮助与支持")).toBeInTheDocument();

    // Development group items
    expect(screen.getByText("本地服务器")).toBeInTheDocument();
    expect(screen.getByText("工具路径")).toBeInTheDocument();
    expect(screen.getByText("环境变量")).toBeInTheDocument();

    // Privacy group items
    expect(screen.getByText("系统规则")).toBeInTheDocument();
    expect(screen.getByText("自定义规则")).toBeInTheDocument();
    expect(screen.getByText("规则测试")).toBeInTheDocument();
    expect(screen.getByText("拦截记录")).toBeInTheDocument();
  });

  it("highlights active group with blue-500 border class", () => {
    renderWithRouter(<SettingsSidebar />, "/settings/general");

    const generalLink = screen.getByTestId("settings-nav-general");
    expect(generalLink.className).toContain("border-blue-500");

    const devLink = screen.getByTestId("settings-nav-development");
    expect(devLink.className).toContain("border-transparent");
  });

  it("highlights development group when on /settings/development", () => {
    renderWithRouter(<SettingsSidebar />, "/settings/development");

    const devLink = screen.getByTestId("settings-nav-development");
    expect(devLink.className).toContain("border-blue-500");

    const generalLink = screen.getByTestId("settings-nav-general");
    expect(generalLink.className).toContain("border-transparent");
  });

  it("has fixed width of 200px", () => {
    renderWithRouter(<SettingsSidebar />);

    const nav = screen.getByTestId("settings-sidebar");
    expect(nav.className).toContain("w-[200px]");
  });

  it("renders sub-items as non-interactive div elements", () => {
    renderWithRouter(<SettingsSidebar />);

    // Sub-items should NOT be links
    const langItem = screen.getByText("语言");
    expect(langItem.closest("a")).toBeNull();
    expect(langItem.closest("div")).not.toBeNull();
  });
});
