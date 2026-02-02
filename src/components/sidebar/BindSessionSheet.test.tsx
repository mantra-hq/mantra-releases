/**
 * BindSessionSheet 组件测试
 * Story 12.2: Dialog → Sheet 改造 - Code Review 补充测试
 */

import { describe, it, expect, vi, beforeEach, beforeAll } from "vitest";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { BindSessionSheet } from "./BindSessionSheet";
import type { Project, Session } from "@/types/project";

// Mock hasPointerCapture and scrollIntoView for Radix UI Select (jsdom limitation)
beforeAll(() => {
  Element.prototype.hasPointerCapture = vi.fn(() => false);
  Element.prototype.setPointerCapture = vi.fn();
  Element.prototype.releasePointerCapture = vi.fn();
  Element.prototype.scrollIntoView = vi.fn();
});

// Mock i18n
vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, params?: Record<string, unknown>) => {
      if (params?.session) return `${key} - ${params.session}`;
      if (params?.project) return `${key} - ${params.project}`;
      if (params?.error) return `${key} - ${params.error}`;
      return key;
    },
  }),
}));

// Mock sonner toast
vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

// Mock project hooks
vi.mock("@/hooks/useProjects", () => ({
  bindSessionToProject: vi.fn(),
  unbindSession: vi.fn(),
}));

// Import after mocking
import { bindSessionToProject, unbindSession } from "@/hooks/useProjects";
import { toast } from "sonner";

const mockBindSessionToProject = vi.mocked(bindSessionToProject);
const mockUnbindSession = vi.mocked(unbindSession);

const mockSession: Session = {
  id: "session-1",
  title: "Test Session",
  tool: "claude-code",
  created_at: "2024-01-01T00:00:00Z",
  updated_at: "2024-01-01T00:00:00Z",
} as Session;

const mockProjects: Project[] = [
  {
    id: "project-1",
    name: "Project Alpha",
    path: "/path/to/alpha",
    created_at: "2024-01-01T00:00:00Z",
    updated_at: "2024-01-01T00:00:00Z",
  } as Project,
  {
    id: "project-2",
    name: "Project Beta",
    path: "/path/to/beta",
    created_at: "2024-01-01T00:00:00Z",
    updated_at: "2024-01-01T00:00:00Z",
  } as Project,
];

describe("BindSessionSheet", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockBindSessionToProject.mockResolvedValue(undefined);
    mockUnbindSession.mockResolvedValue(undefined);
  });

  describe("渲染测试", () => {
    it("应该正确渲染 Sheet", async () => {
      render(
        <BindSessionSheet
          isOpen={true}
          onOpenChange={vi.fn()}
          session={mockSession}
          projects={mockProjects}
        />
      );

      expect(screen.getByText("session.bindToProject")).toBeInTheDocument();
    });

    it("isOpen 为 false 时不应该渲染内容", () => {
      render(
        <BindSessionSheet
          isOpen={false}
          onOpenChange={vi.fn()}
          session={mockSession}
          projects={mockProjects}
        />
      );

      expect(screen.queryByText("session.bindToProject")).not.toBeInTheDocument();
    });

    it("session 为 null 时不应该渲染", () => {
      render(
        <BindSessionSheet
          isOpen={true}
          onOpenChange={vi.fn()}
          session={null}
          projects={mockProjects}
        />
      );

      expect(screen.queryByText("session.bindToProject")).not.toBeInTheDocument();
    });

    it("应该显示会话描述", async () => {
      render(
        <BindSessionSheet
          isOpen={true}
          onOpenChange={vi.fn()}
          session={mockSession}
          projects={mockProjects}
        />
      );

      // 会话名称显示在 i18n 翻译的描述文本中
      expect(screen.getByText(/session.bindDescription/)).toBeInTheDocument();
    });
  });

  describe("绑定操作", () => {
    it("未选择项目时绑定按钮应禁用", async () => {
      render(
        <BindSessionSheet
          isOpen={true}
          onOpenChange={vi.fn()}
          session={mockSession}
          projects={mockProjects}
        />
      );

      const bindButton = screen.getByText("session.bind").closest("button");
      expect(bindButton).toBeDisabled();
    });

    it("绑定成功应该调用回调并关闭 Sheet", async () => {
      const user = userEvent.setup();
      const onOpenChange = vi.fn();
      const onBindSuccess = vi.fn();

      render(
        <BindSessionSheet
          isOpen={true}
          onOpenChange={onOpenChange}
          session={mockSession}
          projects={mockProjects}
          onBindSuccess={onBindSuccess}
        />
      );

      // 选择项目 - 点击 Select trigger
      const selectTrigger = screen.getByRole("combobox");
      await user.click(selectTrigger);

      // 等待下拉选项出现并选择
      const option = await screen.findByRole("option", { name: "Project Alpha" });
      await user.click(option);

      // 点击绑定
      const bindButton = screen.getByText("session.bind").closest("button");
      await user.click(bindButton!);

      await waitFor(() => {
        expect(mockBindSessionToProject).toHaveBeenCalledWith("session-1", "project-1");
      });

      expect(toast.success).toHaveBeenCalled();
      expect(onBindSuccess).toHaveBeenCalled();
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });

    it("绑定失败应该显示错误", async () => {
      mockBindSessionToProject.mockRejectedValue(new Error("Bind failed"));

      const user = userEvent.setup();

      render(
        <BindSessionSheet
          isOpen={true}
          onOpenChange={vi.fn()}
          session={mockSession}
          projects={mockProjects}
        />
      );

      // 选择项目
      const selectTrigger = screen.getByRole("combobox");
      await user.click(selectTrigger);

      const option = await screen.findByRole("option", { name: "Project Alpha" });
      await user.click(option);

      // 点击绑定
      const bindButton = screen.getByText("session.bind").closest("button");
      await user.click(bindButton!);

      await waitFor(() => {
        expect(toast.error).toHaveBeenCalled();
      });
    });
  });

  describe("解绑操作", () => {
    it("已绑定时应该显示解绑按钮", async () => {
      render(
        <BindSessionSheet
          isOpen={true}
          onOpenChange={vi.fn()}
          session={mockSession}
          projects={mockProjects}
          currentProjectId="project-1"
        />
      );

      expect(screen.getByText("session.unbind")).toBeInTheDocument();
    });

    it("未绑定时不应该显示解绑按钮", async () => {
      render(
        <BindSessionSheet
          isOpen={true}
          onOpenChange={vi.fn()}
          session={mockSession}
          projects={mockProjects}
        />
      );

      expect(screen.queryByText("session.unbind")).not.toBeInTheDocument();
    });

    it("解绑成功应该调用回调并关闭 Sheet", async () => {
      const user = userEvent.setup();
      const onOpenChange = vi.fn();
      const onBindSuccess = vi.fn();

      render(
        <BindSessionSheet
          isOpen={true}
          onOpenChange={onOpenChange}
          session={mockSession}
          projects={mockProjects}
          currentProjectId="project-1"
          onBindSuccess={onBindSuccess}
        />
      );

      const unbindButton = screen.getByText("session.unbind").closest("button");
      await user.click(unbindButton!);

      await waitFor(() => {
        expect(mockUnbindSession).toHaveBeenCalledWith("session-1");
      });

      expect(toast.success).toHaveBeenCalled();
      expect(onBindSuccess).toHaveBeenCalled();
      expect(onOpenChange).toHaveBeenCalledWith(false);
    });
  });

  describe("已绑定状态", () => {
    it("应该显示当前绑定的项目信息", async () => {
      render(
        <BindSessionSheet
          isOpen={true}
          onOpenChange={vi.fn()}
          session={mockSession}
          projects={mockProjects}
          currentProjectId="project-1"
        />
      );

      // 检查绑定提示文本是否显示（包含项目名称的翻译键）
      expect(screen.getByText(/session.currentlyBoundTo/)).toBeInTheDocument();
    });
  });
});
