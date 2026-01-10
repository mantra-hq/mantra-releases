/**
 * 视觉回归测试辅助函数
 * Story 9.5: Task 2
 *
 * 提供:
 * - 动态内容遮罩函数
 * - 等待动画完成函数
 * - 自定义 CSS 注入函数
 * - 视觉测试通用配置
 */

import { Page, Locator } from "@playwright/test";

// =============================================================================
// 视觉测试配置常量
// =============================================================================

/**
 * 需要遮罩的动态元素 test-id 列表
 * 这些元素内容会随时间/状态变化，需要在截图时遮罩
 */
export const DYNAMIC_ELEMENTS_TO_MASK = [
  "timestamp",
  "relative-time",
  "avatar",
  "session-count",
  "scroll-indicator",
  "loading-spinner",
  "progress-bar",
] as const;

/**
 * 禁用动画的 CSS
 * 用于确保截图时所有动画都处于最终状态
 */
export const DISABLE_ANIMATIONS_CSS = `
  *, *::before, *::after {
    animation-duration: 0s !important;
    animation-delay: 0s !important;
    transition-duration: 0s !important;
    transition-delay: 0s !important;
  }
`;

/**
 * 视觉测试阈值配置
 */
export const VISUAL_THRESHOLDS = {
  /** 全页面截图 */
  fullPage: { maxDiffPixels: 200, threshold: 0.2 },
  /** 组件截图 */
  component: { maxDiffPixels: 50, threshold: 0.15 },
  /** 代码高亮（Monaco 渲染差异较大） */
  codeHighlight: { maxDiffPixels: 100, threshold: 0.25 },
  /** 图标/按钮（非常严格） */
  iconButton: { maxDiffPixels: 10, threshold: 0.1 },
} as const;

// =============================================================================
// 动态内容遮罩函数 (Task 2.2)
// =============================================================================

/**
 * 获取页面中所有需要遮罩的动态元素定位器
 * @param page Playwright Page 对象
 * @returns 需要遮罩的 Locator 数组
 */
export function getDynamicElementsToMask(page: Page): Locator[] {
  return DYNAMIC_ELEMENTS_TO_MASK.map((testId) =>
    page.locator(`[data-testid="${testId}"]`)
  );
}

/**
 * 获取指定元素内需要遮罩的动态子元素
 * @param element 父元素 Locator
 * @returns 需要遮罩的 Locator 数组
 */
export function getDynamicChildrenToMask(element: Locator): Locator[] {
  return DYNAMIC_ELEMENTS_TO_MASK.map((testId) =>
    element.locator(`[data-testid="${testId}"]`)
  );
}

/**
 * 创建自定义遮罩定位器数组
 * @param page Playwright Page 对象
 * @param selectors 自定义选择器数组
 * @returns Locator 数组
 */
export function createMaskLocators(page: Page, selectors: string[]): Locator[] {
  return selectors.map((selector) => page.locator(selector));
}

// =============================================================================
// 等待动画完成函数 (Task 2.3)
// =============================================================================

/**
 * 等待页面动画完成
 * 包括: 网络空闲、字体加载、动画结束
 * @param page Playwright Page 对象
 * @param options 配置选项
 */
export async function waitForVisualStability(
  page: Page,
  options?: { timeout?: number }
): Promise<void> {
  const timeout = options?.timeout ?? 10000;

  // 1. 等待网络空闲
  await page.waitForLoadState("networkidle", { timeout });

  // 2. 等待字体加载完成
  await page.evaluate(() => document.fonts.ready);

  // 3. 等待所有动画帧完成
  await page.evaluate(() => {
    return new Promise<void>((resolve) => {
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          resolve();
        });
      });
    });
  });
}

/**
 * 等待特定元素稳定（不再变化）
 * @param locator 目标元素
 * @param options 配置选项
 */
export async function waitForElementStability(
  locator: Locator,
  options?: { timeout?: number }
): Promise<void> {
  const timeout = options?.timeout ?? 5000;

  // 等待元素可见
  await locator.waitFor({ state: "visible", timeout });

  // 等待元素位置/大小稳定
  const page = locator.page();
  await page.evaluate(async (selector) => {
    const element = document.querySelector(selector);
    if (!element) return;

    return new Promise<void>((resolve) => {
      let lastRect = element.getBoundingClientRect();
      let stableCount = 0;
      const requiredStableFrames = 3;

      const checkStability = () => {
        const currentRect = element.getBoundingClientRect();
        if (
          Math.abs(currentRect.x - lastRect.x) < 1 &&
          Math.abs(currentRect.y - lastRect.y) < 1 &&
          Math.abs(currentRect.width - lastRect.width) < 1 &&
          Math.abs(currentRect.height - lastRect.height) < 1
        ) {
          stableCount++;
          if (stableCount >= requiredStableFrames) {
            resolve();
            return;
          }
        } else {
          stableCount = 0;
        }
        lastRect = currentRect;
        requestAnimationFrame(checkStability);
      };

      requestAnimationFrame(checkStability);
    });
  }, await locator.evaluate((el) => {
    // 获取元素的唯一选择器
    const path: string[] = [];
    let current: Element | null = el;
    while (current && current !== document.body) {
      let selector = current.tagName.toLowerCase();
      if (current.id) {
        selector += `#${current.id}`;
        path.unshift(selector);
        break;
      } else {
        const parent = current.parentElement;
        if (parent) {
          const siblings = Array.from(parent.children).filter(
            (c) => c.tagName === current!.tagName
          );
          if (siblings.length > 1) {
            const index = siblings.indexOf(current) + 1;
            selector += `:nth-of-type(${index})`;
          }
        }
        path.unshift(selector);
        current = parent;
      }
    }
    return path.join(" > ");
  }));
}

// =============================================================================
// 自定义 CSS 注入函数 (Task 2.4)
// =============================================================================

/**
 * 注入禁用动画的 CSS
 * @param page Playwright Page 对象
 */
export async function injectDisableAnimationsCSS(page: Page): Promise<void> {
  await page.addStyleTag({ content: DISABLE_ANIMATIONS_CSS });
}

/**
 * 注入自定义 CSS
 * @param page Playwright Page 对象
 * @param css CSS 内容
 */
export async function injectCustomCSS(page: Page, css: string): Promise<void> {
  await page.addStyleTag({ content: css });
}

/**
 * 注入隐藏特定元素的 CSS
 * @param page Playwright Page 对象
 * @param selectors 要隐藏的元素选择器
 */
export async function injectHideElementsCSS(
  page: Page,
  selectors: string[]
): Promise<void> {
  const css = selectors
    .map((selector) => `${selector} { visibility: hidden !important; }`)
    .join("\n");
  await page.addStyleTag({ content: css });
}

// =============================================================================
// 视口和页面准备函数
// =============================================================================

/**
 * 设置标准视口尺寸用于视觉测试
 * @param page Playwright Page 对象
 * @param preset 预设尺寸
 */
export async function setStandardViewport(
  page: Page,
  preset: "desktop" | "tablet" | "mobile" = "desktop"
): Promise<void> {
  const sizes = {
    desktop: { width: 1280, height: 720 },
    tablet: { width: 768, height: 1024 },
    mobile: { width: 375, height: 667 },
  };
  await page.setViewportSize(sizes[preset]);
}

/**
 * 为视觉测试准备页面
 * 执行所有必要的准备步骤：禁用动画、设置视口、等待稳定
 * @param page Playwright Page 对象
 * @param options 配置选项
 */
export async function preparePageForVisualTest(
  page: Page,
  options?: {
    viewport?: "desktop" | "tablet" | "mobile";
    disableAnimations?: boolean;
    waitForStability?: boolean;
  }
): Promise<void> {
  const {
    viewport = "desktop",
    disableAnimations = true,
    waitForStability = true,
  } = options ?? {};

  // 1. 设置视口
  await setStandardViewport(page, viewport);

  // 2. 禁用动画
  if (disableAnimations) {
    await injectDisableAnimationsCSS(page);
  }

  // 3. 等待页面稳定
  if (waitForStability) {
    await waitForVisualStability(page);
  }
}

// =============================================================================
// 截图配置生成函数
// =============================================================================

/**
 * 生成页面级截图配置
 * @param page Playwright Page 对象
 * @param options 自定义选项
 */
export function getPageScreenshotOptions(
  page: Page,
  options?: {
    additionalMasks?: string[];
    fullPage?: boolean;
  }
) {
  const { additionalMasks = [], fullPage = false } = options ?? {};

  const masks = [
    ...getDynamicElementsToMask(page),
    ...createMaskLocators(page, additionalMasks),
  ];

  return {
    mask: masks,
    fullPage,
    ...VISUAL_THRESHOLDS.fullPage,
    animations: "disabled" as const,
  };
}

/**
 * 生成组件级截图配置
 * @param element 目标组件 Locator
 * @param options 自定义选项
 */
export function getComponentScreenshotOptions(
  element: Locator,
  options?: {
    additionalMasks?: Locator[];
    thresholdType?: keyof typeof VISUAL_THRESHOLDS;
  }
) {
  const { additionalMasks = [], thresholdType = "component" } = options ?? {};

  const masks = [...getDynamicChildrenToMask(element), ...additionalMasks];

  return {
    mask: masks,
    ...VISUAL_THRESHOLDS[thresholdType],
    animations: "disabled" as const,
  };
}
