/**
 * EnvVariableItem 组件测试
 * Story 11.4: 环境变量管理 - Task 7.3
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { EnvVariableItem } from "./EnvVariableItem";
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

import { getEnvVariableDecrypted } from "@/lib/env-variable-ipc";

describe("EnvVariableItem", () => {
  const mockVariable: EnvVariable = {
    id: "test-id",
    name: "OPENAI_API_KEY",
    masked_value: "sk-****...****xyz",
    description: "OpenAI API Key",
    created_at: "2026-01-30T00:00:00Z",
    updated_at: "2026-01-30T00:00:00Z",
  };

  const mockOnEdit = vi.fn();
  const mockOnDelete = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders variable name and masked value", () => {
    render(
      <EnvVariableItem
        variable={mockVariable}
        onEdit={mockOnEdit}
        onDelete={mockOnDelete}
      />
    );

    expect(screen.getByText("OPENAI_API_KEY")).toBeInTheDocument();
    expect(screen.getByText("sk-****...****xyz")).toBeInTheDocument();
    expect(screen.getByText("OpenAI API Key")).toBeInTheDocument();
  });

  it("calls onEdit when edit button is clicked", () => {
    render(
      <EnvVariableItem
        variable={mockVariable}
        onEdit={mockOnEdit}
        onDelete={mockOnDelete}
      />
    );

    const editButton = screen.getByTestId("env-variable-edit-OPENAI_API_KEY");
    fireEvent.click(editButton);

    expect(mockOnEdit).toHaveBeenCalledWith(mockVariable);
  });

  it("calls onDelete when delete button is clicked", () => {
    render(
      <EnvVariableItem
        variable={mockVariable}
        onEdit={mockOnEdit}
        onDelete={mockOnDelete}
      />
    );

    const deleteButton = screen.getByTestId("env-variable-delete-OPENAI_API_KEY");
    fireEvent.click(deleteButton);

    expect(mockOnDelete).toHaveBeenCalledWith(mockVariable);
  });

  it("shows decrypted value when toggle button is clicked", async () => {
    const mockDecrypted = vi.mocked(getEnvVariableDecrypted);
    mockDecrypted.mockResolvedValue("sk-real-secret-key");

    render(
      <EnvVariableItem
        variable={mockVariable}
        onEdit={mockOnEdit}
        onDelete={mockOnDelete}
      />
    );

    const toggleButton = screen.getByTestId("env-variable-toggle-OPENAI_API_KEY");
    fireEvent.click(toggleButton);

    await waitFor(() => {
      expect(screen.getByText("sk-real-secret-key")).toBeInTheDocument();
    });

    expect(mockDecrypted).toHaveBeenCalledWith("OPENAI_API_KEY");
  });

  it("hides value when toggle button is clicked again", async () => {
    const mockDecrypted = vi.mocked(getEnvVariableDecrypted);
    mockDecrypted.mockResolvedValue("sk-real-secret-key");

    render(
      <EnvVariableItem
        variable={mockVariable}
        onEdit={mockOnEdit}
        onDelete={mockOnDelete}
      />
    );

    const toggleButton = screen.getByTestId("env-variable-toggle-OPENAI_API_KEY");
    
    // Show
    fireEvent.click(toggleButton);
    await waitFor(() => {
      expect(screen.getByText("sk-real-secret-key")).toBeInTheDocument();
    });

    // Hide
    fireEvent.click(toggleButton);
    await waitFor(() => {
      expect(screen.getByText("sk-****...****xyz")).toBeInTheDocument();
    });
  });

  it("renders without description", () => {
    const variableWithoutDesc: EnvVariable = {
      ...mockVariable,
      description: null,
    };

    render(
      <EnvVariableItem
        variable={variableWithoutDesc}
        onEdit={mockOnEdit}
        onDelete={mockOnDelete}
      />
    );

    expect(screen.getByText("OPENAI_API_KEY")).toBeInTheDocument();
    expect(screen.queryByText("OpenAI API Key")).not.toBeInTheDocument();
  });
});
