/**
 * EnvVariableList 组件测试
 * Story 11.4: 环境变量管理 - Task 7.3
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { EnvVariableList } from "./EnvVariableList";
import type { EnvVariable } from "@/lib/env-variable-ipc";

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

// Mock IPC
vi.mock("@/lib/env-variable-ipc", () => ({
  getEnvVariableDecrypted: vi.fn(),
}));

describe("EnvVariableList", () => {
  const mockVariables: EnvVariable[] = [
    {
      id: "1",
      name: "OPENAI_API_KEY",
      masked_value: "sk-****...****xyz",
      description: "OpenAI API Key",
      created_at: "2026-01-30T00:00:00Z",
      updated_at: "2026-01-30T00:00:00Z",
    },
    {
      id: "2",
      name: "ANTHROPIC_API_KEY",
      masked_value: "sk-****...****abc",
      description: "Anthropic API Key",
      created_at: "2026-01-30T00:00:00Z",
      updated_at: "2026-01-30T00:00:00Z",
    },
    {
      id: "3",
      name: "DATABASE_URL",
      masked_value: "post****...****5432",
      description: null,
      created_at: "2026-01-30T00:00:00Z",
      updated_at: "2026-01-30T00:00:00Z",
    },
    {
      id: "4",
      name: "SECRET_KEY",
      masked_value: "****",
      description: "Secret key for encryption",
      created_at: "2026-01-30T00:00:00Z",
      updated_at: "2026-01-30T00:00:00Z",
    },
  ];

  const mockOnEdit = vi.fn();
  const mockOnDelete = vi.fn();
  const mockOnSearchChange = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders all variables", () => {
    render(
      <EnvVariableList
        variables={mockVariables}
        searchQuery=""
        onSearchChange={mockOnSearchChange}
        onEdit={mockOnEdit}
        onDelete={mockOnDelete}
      />
    );

    expect(screen.getByText("OPENAI_API_KEY")).toBeInTheDocument();
    expect(screen.getByText("ANTHROPIC_API_KEY")).toBeInTheDocument();
    expect(screen.getByText("DATABASE_URL")).toBeInTheDocument();
    expect(screen.getByText("SECRET_KEY")).toBeInTheDocument();
  });

  it("shows search input when more than 3 variables", () => {
    render(
      <EnvVariableList
        variables={mockVariables}
        searchQuery=""
        onSearchChange={mockOnSearchChange}
        onEdit={mockOnEdit}
        onDelete={mockOnDelete}
      />
    );

    expect(screen.getByTestId("env-variable-search")).toBeInTheDocument();
  });

  it("hides search input when 3 or fewer variables", () => {
    render(
      <EnvVariableList
        variables={mockVariables.slice(0, 3)}
        searchQuery=""
        onSearchChange={mockOnSearchChange}
        onEdit={mockOnEdit}
        onDelete={mockOnDelete}
      />
    );

    expect(screen.queryByTestId("env-variable-search")).not.toBeInTheDocument();
  });

  it("filters variables by name", () => {
    render(
      <EnvVariableList
        variables={mockVariables}
        searchQuery="openai"
        onSearchChange={mockOnSearchChange}
        onEdit={mockOnEdit}
        onDelete={mockOnDelete}
      />
    );

    expect(screen.getByText("OPENAI_API_KEY")).toBeInTheDocument();
    expect(screen.queryByText("ANTHROPIC_API_KEY")).not.toBeInTheDocument();
    expect(screen.queryByText("DATABASE_URL")).not.toBeInTheDocument();
  });

  it("filters variables by description", () => {
    render(
      <EnvVariableList
        variables={mockVariables}
        searchQuery="encryption"
        onSearchChange={mockOnSearchChange}
        onEdit={mockOnEdit}
        onDelete={mockOnDelete}
      />
    );

    expect(screen.getByText("SECRET_KEY")).toBeInTheDocument();
    expect(screen.queryByText("OPENAI_API_KEY")).not.toBeInTheDocument();
  });

  it("shows empty state when no variables", () => {
    render(
      <EnvVariableList
        variables={[]}
        searchQuery=""
        onSearchChange={mockOnSearchChange}
        onEdit={mockOnEdit}
        onDelete={mockOnDelete}
      />
    );

    expect(screen.getByText("hub.envVariables.empty")).toBeInTheDocument();
    expect(screen.getByText("hub.envVariables.emptyHint")).toBeInTheDocument();
  });

  it("shows no results message when search has no matches", () => {
    render(
      <EnvVariableList
        variables={mockVariables}
        searchQuery="nonexistent"
        onSearchChange={mockOnSearchChange}
        onEdit={mockOnEdit}
        onDelete={mockOnDelete}
      />
    );

    expect(screen.getByText("hub.envVariables.noSearchResults")).toBeInTheDocument();
  });

  it("calls onSearchChange when search input changes", () => {
    render(
      <EnvVariableList
        variables={mockVariables}
        searchQuery=""
        onSearchChange={mockOnSearchChange}
        onEdit={mockOnEdit}
        onDelete={mockOnDelete}
      />
    );

    const searchInput = screen.getByTestId("env-variable-search");
    fireEvent.change(searchInput, { target: { value: "test" } });

    expect(mockOnSearchChange).toHaveBeenCalledWith("test");
  });
});
