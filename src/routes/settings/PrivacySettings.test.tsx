/**
 * PrivacySettings Tests - 隐私与安全设置页面测试
 * Story 2-35: Task 3.3
 */

import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { PrivacySettings } from "./PrivacySettings";

// Mock navigate
const mockNavigate = vi.fn();
vi.mock("react-router-dom", async () => {
  const actual = await vi.importActual("react-router-dom");
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

// Mock child components
vi.mock("@/components/settings/SystemRuleList", () => ({
  SystemRuleList: ({ defaultCollapsed }: { defaultCollapsed?: boolean }) => (
    <div data-testid="system-rule-list" data-collapsed={defaultCollapsed}>
      SystemRuleList
    </div>
  ),
}));

vi.mock("@/components/settings/RuleList", () => ({
  RuleList: ({ onImport, onExport }: { onImport?: () => void; onExport?: () => void }) => (
    <div data-testid="rule-list">
      <button data-testid="import-btn" onClick={onImport}>Import</button>
      <button data-testid="export-btn" onClick={onExport}>Export</button>
    </div>
  ),
}));

vi.mock("@/components/settings/RuleTestPanel", () => ({
  RuleTestPanel: () => <div data-testid="rule-test-panel">RuleTestPanel</div>,
}));

// Mock stores and libs
vi.mock("@/stores/useSanitizationRulesStore", () => ({
  useSanitizationRulesStore: () => ({
    rules: [],
    importRules: vi.fn(),
  }),
}));

vi.mock("@/lib/rule-io", () => ({
  exportRules: vi.fn(),
  importRules: vi.fn(),
}));

vi.mock("@/lib/feedback", () => ({
  feedback: {
    imported: vi.fn(),
    exported: vi.fn(),
    error: vi.fn(),
  },
}));

const renderWithRouter = (ui: React.ReactElement) => {
  return render(<MemoryRouter>{ui}</MemoryRouter>);
};

describe("PrivacySettings", () => {
  it("renders SystemRuleList with defaultCollapsed=true", () => {
    renderWithRouter(<PrivacySettings />);
    const srl = screen.getByTestId("system-rule-list");
    expect(srl).toBeInTheDocument();
    expect(srl.getAttribute("data-collapsed")).toBe("true");
  });

  it("renders RuleList component", () => {
    renderWithRouter(<PrivacySettings />);
    expect(screen.getByTestId("rule-list")).toBeInTheDocument();
  });

  it("renders RuleTestPanel component", () => {
    renderWithRouter(<PrivacySettings />);
    expect(screen.getByTestId("rule-test-panel")).toBeInTheDocument();
  });

  it("renders privacy records entry link", () => {
    renderWithRouter(<PrivacySettings />);
    expect(screen.getByTestId("privacy-records-link")).toBeInTheDocument();
  });

  it("navigates to /privacy-records when entry link is clicked", () => {
    renderWithRouter(<PrivacySettings />);

    fireEvent.click(screen.getByTestId("privacy-records-link"));
    expect(mockNavigate).toHaveBeenCalledWith("/privacy-records");
  });
});
