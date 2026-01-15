/**
 * ImageBlock Component Tests
 * Story 8.16: Task 6
 */

import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ImageBlock } from "./ImageBlock";

// Mock 小型 1x1 PNG 图片 base64
const MOCK_PNG_BASE64 =
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";

describe("ImageBlock", () => {
  describe("base64 images", () => {
    it("renders base64 image with correct data URI", () => {
      render(
        <ImageBlock
          data={MOCK_PNG_BASE64}
          mediaType="image/png"
          sourceType="base64"
        />
      );

      const img = screen.getByRole("img");
      expect(img).toHaveAttribute(
        "src",
        `data:image/png;base64,${MOCK_PNG_BASE64}`
      );
    });

    it("uses base64 as default sourceType", () => {
      render(<ImageBlock data={MOCK_PNG_BASE64} mediaType="image/jpeg" />);

      const img = screen.getByRole("img");
      expect(img.getAttribute("src")).toContain("data:image/jpeg;base64,");
    });

    it("renders with alt text when provided", () => {
      render(
        <ImageBlock
          data={MOCK_PNG_BASE64}
          mediaType="image/png"
          altText="Screenshot of error"
        />
      );

      const img = screen.getByRole("img");
      expect(img).toHaveAttribute("alt", "Screenshot of error");
    });

    it("uses default alt text when not provided", () => {
      render(<ImageBlock data={MOCK_PNG_BASE64} mediaType="image/png" />);

      const img = screen.getByRole("img");
      expect(img).toHaveAttribute("alt", "用户上传的图片");
    });
  });

  describe("URL images", () => {
    it("renders URL image directly", () => {
      const imageUrl = "https://example.com/image.png";
      render(
        <ImageBlock data={imageUrl} mediaType="image/png" sourceType="url" />
      );

      const img = screen.getByRole("img");
      expect(img).toHaveAttribute("src", imageUrl);
    });
  });

  describe("loading and error states", () => {
    it("shows loading indicator initially", () => {
      render(<ImageBlock data={MOCK_PNG_BASE64} mediaType="image/png" />);

      // Image should be hidden during loading (has 'hidden' class)
      const img = screen.getByRole("img");
      expect(img).toHaveClass("hidden");
    });

    it("shows image after load event", () => {
      render(<ImageBlock data={MOCK_PNG_BASE64} mediaType="image/png" />);

      const img = screen.getByRole("img");
      fireEvent.load(img);

      expect(img).not.toHaveClass("hidden");
    });

    it("shows error state when image fails to load", () => {
      render(<ImageBlock data="invalid-data" mediaType="image/png" />);

      const img = screen.getByRole("img");
      fireEvent.error(img);

      expect(screen.getByText("图片加载失败")).toBeInTheDocument();
    });

    it("shows external link for URL images on error", () => {
      const imageUrl = "https://example.com/broken.png";
      render(
        <ImageBlock data={imageUrl} mediaType="image/png" sourceType="url" />
      );

      const img = screen.getByRole("img");
      fireEvent.error(img);

      const link = screen.getByText("查看原图");
      expect(link).toHaveAttribute("href", imageUrl);
      expect(link).toHaveAttribute("target", "_blank");
    });
  });

  describe("expand/zoom functionality", () => {
    it("opens modal on click", () => {
      render(<ImageBlock data={MOCK_PNG_BASE64} mediaType="image/png" />);

      // Simulate image loaded
      const img = screen.getByRole("img");
      fireEvent.load(img);

      // Click the container
      const container = screen.getByRole("button");
      fireEvent.click(container);

      // Modal should appear
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });

    it("closes modal on backdrop click", () => {
      render(<ImageBlock data={MOCK_PNG_BASE64} mediaType="image/png" />);

      const img = screen.getByRole("img");
      fireEvent.load(img);

      // Open modal
      const container = screen.getByRole("button");
      fireEvent.click(container);

      // Click backdrop (the dialog itself)
      const dialog = screen.getByRole("dialog");
      fireEvent.click(dialog);

      // Modal should close
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });

    it("closes modal on Escape key", () => {
      render(<ImageBlock data={MOCK_PNG_BASE64} mediaType="image/png" />);

      const img = screen.getByRole("img");
      fireEvent.load(img);

      // Open modal
      const container = screen.getByRole("button");
      fireEvent.click(container);

      // Press Escape
      const dialog = screen.getByRole("dialog");
      fireEvent.keyDown(dialog, { key: "Escape" });

      // Modal should close
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });
  });

  describe("accessibility", () => {
    it("container is keyboard accessible", () => {
      render(<ImageBlock data={MOCK_PNG_BASE64} mediaType="image/png" />);

      const container = screen.getByRole("button");
      expect(container).toHaveAttribute("tabIndex", "0");
    });

    it("modal has proper ARIA attributes", () => {
      render(<ImageBlock data={MOCK_PNG_BASE64} mediaType="image/png" />);

      const img = screen.getByRole("img");
      fireEvent.load(img);

      const container = screen.getByRole("button");
      fireEvent.click(container);

      const dialog = screen.getByRole("dialog");
      expect(dialog).toHaveAttribute("aria-modal", "true");
      expect(dialog).toHaveAttribute("aria-label", "图片预览");
    });
  });

  describe("styling", () => {
    it("applies custom className", () => {
      render(
        <ImageBlock
          data={MOCK_PNG_BASE64}
          mediaType="image/png"
          className="custom-class"
        />
      );

      const container = screen.getByRole("button");
      expect(container).toHaveClass("custom-class");
    });
  });
});
