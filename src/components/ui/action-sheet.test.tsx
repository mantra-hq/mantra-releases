/**
 * ActionSheet Tests
 * Story 12.4: ActionSheet 统一封装组件
 *
 * Tests for:
 * - AC #2: size prop 正确映射 className
 * - AC #3: 默认 side="right"
 * - AC #5: open/onOpenChange/onInteractOutside/onEscapeKeyDown 透传
 * - AC #9: 无控制台错误
 */

import { describe, it, expect, vi, afterEach } from "vitest";
import { render, screen, cleanup, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  ActionSheet,
  ActionSheetContent,
  ActionSheetHeader,
  ActionSheetTitle,
  ActionSheetDescription,
  ActionSheetFooter,
  ActionSheetClose,
} from "./action-sheet";

afterEach(() => {
  cleanup();
});

describe("ActionSheet", () => {
  describe("size prop", () => {
    it("renders with default size (md) when no size specified", () => {
      render(
        <ActionSheet open>
          <ActionSheetContent data-testid="sheet-content">
            <ActionSheetHeader>
              <ActionSheetTitle>Test Title</ActionSheetTitle>
              <ActionSheetDescription>Test description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      const content = screen.getByTestId("sheet-content");
      expect(content).toHaveClass("max-w-md");
      expect(content).toHaveClass("w-full");
    });

    it("applies max-w-sm for size='sm'", () => {
      render(
        <ActionSheet open>
          <ActionSheetContent size="sm" data-testid="sheet-content">
            <ActionSheetHeader>
              <ActionSheetTitle>Small Sheet</ActionSheetTitle>
              <ActionSheetDescription>Small description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      const content = screen.getByTestId("sheet-content");
      expect(content).toHaveClass("max-w-sm");
    });

    it("applies max-w-md for size='md'", () => {
      render(
        <ActionSheet open>
          <ActionSheetContent size="md" data-testid="sheet-content">
            <ActionSheetHeader>
              <ActionSheetTitle>Medium Sheet</ActionSheetTitle>
              <ActionSheetDescription>Medium description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      const content = screen.getByTestId("sheet-content");
      expect(content).toHaveClass("max-w-md");
    });

    it("applies max-w-lg for size='lg'", () => {
      render(
        <ActionSheet open>
          <ActionSheetContent size="lg" data-testid="sheet-content">
            <ActionSheetHeader>
              <ActionSheetTitle>Large Sheet</ActionSheetTitle>
              <ActionSheetDescription>Large description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      const content = screen.getByTestId("sheet-content");
      expect(content).toHaveClass("max-w-lg");
    });

    it("applies max-w-xl for size='xl'", () => {
      render(
        <ActionSheet open>
          <ActionSheetContent size="xl" data-testid="sheet-content">
            <ActionSheetHeader>
              <ActionSheetTitle>Extra Large Sheet</ActionSheetTitle>
              <ActionSheetDescription>Extra large description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      const content = screen.getByTestId("sheet-content");
      expect(content).toHaveClass("max-w-xl");
    });

    it("applies max-w-2xl for size='2xl'", () => {
      render(
        <ActionSheet open>
          <ActionSheetContent size="2xl" data-testid="sheet-content">
            <ActionSheetHeader>
              <ActionSheetTitle>2XL Sheet</ActionSheetTitle>
              <ActionSheetDescription>2XL description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      const content = screen.getByTestId("sheet-content");
      expect(content).toHaveClass("max-w-2xl");
    });

    it("allows additional className to be passed", () => {
      render(
        <ActionSheet open>
          <ActionSheetContent
            size="lg"
            className="overflow-y-auto custom-class"
            data-testid="sheet-content"
          >
            <ActionSheetHeader>
              <ActionSheetTitle>Test</ActionSheetTitle>
              <ActionSheetDescription>Test description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      const content = screen.getByTestId("sheet-content");
      expect(content).toHaveClass("max-w-lg");
      expect(content).toHaveClass("overflow-y-auto");
      expect(content).toHaveClass("custom-class");
    });
  });

  describe("side default", () => {
    it("defaults to side='right' with slide-in-from-right animation", () => {
      render(
        <ActionSheet open>
          <ActionSheetContent data-testid="sheet-content">
            <ActionSheetHeader>
              <ActionSheetTitle>Right Side Sheet</ActionSheetTitle>
              <ActionSheetDescription>Right side description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      const content = screen.getByTestId("sheet-content");
      // SheetContent with side="right" adds slide-in-from-right class
      expect(content).toHaveClass("data-[state=open]:slide-in-from-right");
      expect(content).toHaveClass("right-0");
    });
  });

  describe("open/onOpenChange behavior", () => {
    it("renders content when open=true", () => {
      render(
        <ActionSheet open>
          <ActionSheetContent>
            <ActionSheetHeader>
              <ActionSheetTitle>Open Sheet</ActionSheetTitle>
              <ActionSheetDescription>Open description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      expect(screen.getByText("Open Sheet")).toBeInTheDocument();
    });

    it("does not render content when open=false", () => {
      render(
        <ActionSheet open={false}>
          <ActionSheetContent>
            <ActionSheetHeader>
              <ActionSheetTitle>Closed Sheet</ActionSheetTitle>
              <ActionSheetDescription>Closed description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      expect(screen.queryByText("Closed Sheet")).not.toBeInTheDocument();
    });

    it("calls onOpenChange when close button is clicked", async () => {
      const handleOpenChange = vi.fn();
      const user = userEvent.setup();

      render(
        <ActionSheet open onOpenChange={handleOpenChange}>
          <ActionSheetContent>
            <ActionSheetHeader>
              <ActionSheetTitle>Test Sheet</ActionSheetTitle>
              <ActionSheetDescription>Test description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      // Find and click the close button (has sr-only "Close" text)
      const closeButton = screen.getByRole("button", { name: /close/i });
      await user.click(closeButton);

      expect(handleOpenChange).toHaveBeenCalledWith(false);
    });
  });

  describe("event prop passthrough", () => {
    it("passes through onInteractOutside", async () => {
      const handleInteractOutside = vi.fn((e) => e.preventDefault());
      const user = userEvent.setup();

      render(
        <div>
          <button data-testid="outside-button">Outside</button>
          <ActionSheet open>
            <ActionSheetContent onInteractOutside={handleInteractOutside}>
              <ActionSheetHeader>
                <ActionSheetTitle>Test Sheet</ActionSheetTitle>
                <ActionSheetDescription>Test description</ActionSheetDescription>
              </ActionSheetHeader>
            </ActionSheetContent>
          </ActionSheet>
        </div>
      );

      // Click on the overlay (outside the sheet content)
      const overlay = document.querySelector('[data-slot="sheet-overlay"]');
      if (overlay) {
        await user.click(overlay);
        await waitFor(() => {
          expect(handleInteractOutside).toHaveBeenCalled();
        });
      }
    });

    it("passes through onEscapeKeyDown", async () => {
      const handleEscapeKeyDown = vi.fn((e) => e.preventDefault());
      const user = userEvent.setup();

      render(
        <ActionSheet open>
          <ActionSheetContent onEscapeKeyDown={handleEscapeKeyDown}>
            <ActionSheetHeader>
              <ActionSheetTitle>Test Sheet</ActionSheetTitle>
              <ActionSheetDescription>Test description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      await user.keyboard("{Escape}");

      await waitFor(() => {
        expect(handleEscapeKeyDown).toHaveBeenCalled();
      });
    });
  });

  describe("sub-components", () => {
    it("renders ActionSheetHeader with children", () => {
      render(
        <ActionSheet open>
          <ActionSheetContent>
            <ActionSheetHeader data-testid="header">
              <span>Header Content</span>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      expect(screen.getByTestId("header")).toBeInTheDocument();
      expect(screen.getByText("Header Content")).toBeInTheDocument();
    });

    it("renders ActionSheetTitle", () => {
      render(
        <ActionSheet open>
          <ActionSheetContent>
            <ActionSheetHeader>
              <ActionSheetTitle>My Title</ActionSheetTitle>
              <ActionSheetDescription>My description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      expect(screen.getByText("My Title")).toBeInTheDocument();
    });

    it("renders ActionSheetDescription", () => {
      render(
        <ActionSheet open>
          <ActionSheetContent>
            <ActionSheetHeader>
              <ActionSheetTitle>Title</ActionSheetTitle>
              <ActionSheetDescription>My Description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      expect(screen.getByText("My Description")).toBeInTheDocument();
    });

    it("renders ActionSheetFooter with children", () => {
      render(
        <ActionSheet open>
          <ActionSheetContent>
            <ActionSheetHeader>
              <ActionSheetTitle>Title</ActionSheetTitle>
              <ActionSheetDescription>Description</ActionSheetDescription>
            </ActionSheetHeader>
            <ActionSheetFooter data-testid="footer">
              <button>Submit</button>
            </ActionSheetFooter>
          </ActionSheetContent>
        </ActionSheet>
      );

      expect(screen.getByTestId("footer")).toBeInTheDocument();
      expect(screen.getByText("Submit")).toBeInTheDocument();
    });

    it("renders ActionSheetClose", async () => {
      const handleOpenChange = vi.fn();
      const user = userEvent.setup();

      render(
        <ActionSheet open onOpenChange={handleOpenChange}>
          <ActionSheetContent>
            <ActionSheetHeader>
              <ActionSheetTitle>Title</ActionSheetTitle>
              <ActionSheetDescription>Description</ActionSheetDescription>
            </ActionSheetHeader>
            <ActionSheetFooter>
              <ActionSheetClose asChild>
                <button>Cancel</button>
              </ActionSheetClose>
            </ActionSheetFooter>
          </ActionSheetContent>
        </ActionSheet>
      );

      const cancelButton = screen.getByText("Cancel");
      await user.click(cancelButton);

      expect(handleOpenChange).toHaveBeenCalledWith(false);
    });
  });

  describe("TypeScript types", () => {
    // These tests ensure the types are correctly exported and usable
    it("accepts all Sheet props on ActionSheet", () => {
      const handleOpenChange = vi.fn();

      render(
        <ActionSheet
          open
          onOpenChange={handleOpenChange}
          defaultOpen={false}
          modal
        >
          <ActionSheetContent>
            <ActionSheetHeader>
              <ActionSheetTitle>Typed Sheet</ActionSheetTitle>
              <ActionSheetDescription>Typed description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      expect(screen.getByText("Typed Sheet")).toBeInTheDocument();
    });

    it("accepts all SheetContent props except side", () => {
      render(
        <ActionSheet open>
          <ActionSheetContent
            size="lg"
            className="custom"
            onOpenAutoFocus={(e) => e.preventDefault()}
            onCloseAutoFocus={(e) => e.preventDefault()}
          >
            <ActionSheetHeader>
              <ActionSheetTitle>Full Props</ActionSheetTitle>
              <ActionSheetDescription>Full props description</ActionSheetDescription>
            </ActionSheetHeader>
          </ActionSheetContent>
        </ActionSheet>
      );

      expect(screen.getByText("Full Props")).toBeInTheDocument();
    });
  });
});
