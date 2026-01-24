/**
 * MetricCard Tests - 指标卡片组件测试
 * Story 2.34: Code Review - M1 修复
 */

import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { MessageSquare } from "lucide-react";
import { MetricCard } from "./MetricCard";

describe("MetricCard", () => {
  describe("rendering", () => {
    it("should render title and value", () => {
      render(<MetricCard title="Sessions" value={42} />);

      expect(screen.getByText("Sessions")).toBeInTheDocument();
      expect(screen.getByText("42")).toBeInTheDocument();
    });

    it("should render string value", () => {
      render(<MetricCard title="Duration" value="2h 30m" />);

      expect(screen.getByText("Duration")).toBeInTheDocument();
      expect(screen.getByText("2h 30m")).toBeInTheDocument();
    });

    it("should render with unit", () => {
      render(<MetricCard title="Active Days" value={7} unit="days" />);

      expect(screen.getByText("7")).toBeInTheDocument();
      expect(screen.getByText("days")).toBeInTheDocument();
    });

    it("should render with icon", () => {
      render(<MetricCard title="Messages" value={100} icon={MessageSquare} />);

      // Icon should be rendered (SVG element)
      const container = screen.getByText("Messages").closest("div")?.parentElement;
      expect(container?.querySelector("svg")).toBeInTheDocument();
    });

    it("should render with description", () => {
      render(
        <MetricCard
          title="Error Rate"
          value="2.5%"
          description="Last 7 days"
        />
      );

      expect(screen.getByText("Last 7 days")).toBeInTheDocument();
    });

    it("should apply custom className", () => {
      render(
        <MetricCard
          title="Test"
          value={1}
          className="custom-class"
          data-testid="metric-card"
        />
      );

      expect(screen.getByTestId("metric-card")).toHaveClass("custom-class");
    });

    it("should apply data-testid", () => {
      render(
        <MetricCard title="Test" value={1} data-testid="my-metric" />
      );

      expect(screen.getByTestId("my-metric")).toBeInTheDocument();
    });
  });

  describe("styling", () => {
    it("should have correct base classes", () => {
      render(<MetricCard title="Test" value={1} data-testid="metric" />);

      const card = screen.getByTestId("metric");
      expect(card).toHaveClass("flex", "flex-col", "gap-2", "p-4", "rounded-lg");
    });

    it("should render value with large font", () => {
      render(<MetricCard title="Test" value={999} />);

      const valueElement = screen.getByText("999");
      expect(valueElement).toHaveClass("text-2xl", "font-semibold");
    });

    it("should render title with muted color", () => {
      render(<MetricCard title="My Title" value={1} />);

      const titleElement = screen.getByText("My Title");
      expect(titleElement).toHaveClass("text-muted-foreground");
    });
  });
});
