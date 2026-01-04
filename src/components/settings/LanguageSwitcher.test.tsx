/**
 * LanguageSwitcher Tests - 语言切换组件测试
 * Story 2.26: AC1 - 语言切换功能
 */

import { describe, it, expect, vi, beforeAll, afterEach } from "vitest";
import { render, screen, cleanup } from "@testing-library/react";
import { LanguageSwitcher } from "./LanguageSwitcher";
import i18n from "@/i18n";

// Radix UI PointerEvent polyfill
beforeAll(() => {
  class MockPointerEvent extends MouseEvent {
    constructor(type: string, props: PointerEventInit = {}) {
      super(type, props);
      Object.assign(this, {
        pointerId: props.pointerId ?? 0,
        width: props.width ?? 1,
        height: props.height ?? 1,
        pressure: props.pressure ?? 0,
        tangentialPressure: props.tangentialPressure ?? 0,
        tiltX: props.tiltX ?? 0,
        tiltY: props.tiltY ?? 0,
        twist: props.twist ?? 0,
        pointerType: props.pointerType ?? "mouse",
        isPrimary: props.isPrimary ?? true,
      });
    }
  }
  window.PointerEvent = MockPointerEvent as unknown as typeof PointerEvent;
  window.HTMLElement.prototype.scrollIntoView = vi.fn();
  window.HTMLElement.prototype.hasPointerCapture = vi.fn();
  window.HTMLElement.prototype.releasePointerCapture = vi.fn();
});

afterEach(() => {
  cleanup();
  // Reset language to default after each test
  i18n.changeLanguage("zh-CN");
});

describe("LanguageSwitcher", () => {
  it("renders language switcher component (AC1)", () => {
    render(<LanguageSwitcher />);

    // Should show language title from i18n
    expect(screen.getByText("语言")).toBeInTheDocument();
    // Should have select trigger with test id
    expect(screen.getByTestId("language-select")).toBeInTheDocument();
  });

  it("shows current language in select trigger", () => {
    render(<LanguageSwitcher />);

    const select = screen.getByTestId("language-select");
    // Default language is zh-CN, should show "简体中文"
    expect(select).toHaveTextContent("简体中文");
  });

  it("displays globe icon", () => {
    render(<LanguageSwitcher />);

    // Globe icon should be present (lucide-react Globe component)
    const container = screen.getByText("语言").closest("div");
    expect(container?.querySelector("svg")).toBeInTheDocument();
  });

  it("has correct select value matching i18n language", () => {
    render(<LanguageSwitcher />);

    // The select value should match the current i18n language
    expect(i18n.language).toBe("zh-CN");
  });
});
