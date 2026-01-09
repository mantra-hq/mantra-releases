# Mantra 客户端 E2E 测试策略

本文档描述 Mantra Tauri 2 客户端的端到端 (E2E) 测试实施方案，使用 Playwright 作为测试框架。

## 概述

### 测试目标

- **前端表现优先**：重点验证 UI 交互、视觉一致性、用户流程
- **隔离测试**：通过 Mock IPC 层，使测试不依赖 Rust 后端
- **快速反馈**：支持本地开发和 CI 环境

### 测试分层

| 层级 | 工具 | 覆盖范围 | 运行频率 |
|------|------|----------|----------|
| 单元测试 | Vitest + RTL | 组件、Hooks、工具函数 | 每次提交 |
| **E2E 测试** | **Playwright** | **用户流程、页面交互** | **每次 PR** |
| 视觉测试 | Playwright Screenshot | UI 回归检测 | 每次 PR |

## 目录结构

```
apps/client/
├── e2e/                          # Playwright E2E 测试
│   ├── fixtures/                 # 测试夹具
│   │   ├── mock-data.ts          # Mock 数据定义
│   │   ├── mock-projects.ts      # 项目 Mock 数据
│   │   ├── mock-sessions.ts      # 会话 Mock 数据
│   │   └── ipc-mock.ts           # IPC Mock 层
│   ├── pages/                    # Page Object Model
│   │   ├── base.page.ts          # 基础页面类
│   │   ├── player.page.ts        # 播放器页面
│   │   ├── settings.page.ts      # 设置页面
│   │   └── import.page.ts        # 导入向导页面
│   ├── tests/
│   │   ├── player.spec.ts        # 播放器测试
│   │   ├── navigation.spec.ts    # 导航测试
│   │   ├── import.spec.ts        # 导入流程测试
│   │   ├── search.spec.ts        # 搜索功能测试
│   │   └── visual.spec.ts        # 视觉回归测试
│   └── playwright.config.ts      # Playwright 配置
├── src/
│   └── lib/
│       └── ipc-adapter.ts        # IPC 适配器（支持 Mock）
└── ...
```

## IPC Mock 策略

### 核心问题

Tauri 应用的 `invoke()` 调用 Rust 后端。在 Playwright 测试环境中，我们需要 Mock 这一层以实现：

1. 测试隔离 - 不依赖后端状态
2. 快速执行 - 无需启动 Tauri
3. 可控数据 - 使用预定义的 Mock 数据

### 方案：条件注入 Mock

#### 1. IPC 适配器

```typescript
// src/lib/ipc-adapter.ts
import { invoke as tauriInvoke } from "@tauri-apps/api/core";

// 检测测试环境
const isTestEnv = () => {
  if (typeof window === 'undefined') return false;
  return import.meta.env.MODE === 'test' ||
         (window as any).__PLAYWRIGHT_TEST__ === true;
};

// Mock invoke 实现（测试时注入）
let mockInvokeHandler: typeof tauriInvoke | null = null;

export function setMockInvoke(handler: typeof tauriInvoke) {
  mockInvokeHandler = handler;
}

// 统一的 invoke 函数
export const invoke: typeof tauriInvoke = async (cmd, args) => {
  if (isTestEnv() && mockInvokeHandler) {
    return mockInvokeHandler(cmd, args);
  }
  return tauriInvoke(cmd, args);
};
```

#### 2. Mock 处理器

```typescript
// e2e/fixtures/ipc-mock.ts
import { MOCK_PROJECTS, MOCK_SESSIONS } from './mock-data';

type MockHandler = (args?: Record<string, unknown>) => unknown;

const mockHandlers: Record<string, MockHandler> = {
  // 项目相关
  list_projects: () => MOCK_PROJECTS,
  get_project: ({ projectId }) =>
    MOCK_PROJECTS.find(p => p.id === projectId) || null,
  get_project_by_cwd: ({ cwd }) =>
    MOCK_PROJECTS.find(p => p.cwd === cwd) || null,
  get_project_sessions: ({ projectId }) =>
    MOCK_SESSIONS.filter(s => s.project_id === projectId),

  // 会话相关
  get_session: ({ sessionId }) =>
    MOCK_SESSIONS.find(s => s.id === sessionId) || null,

  // Git 相关
  detect_git_repo: () => '/mock/repo/path',
  get_representative_file: () => ({
    path: 'README.md',
    content: '# Mock Project',
  }),
  get_file_at_head: () => ({
    content: '// Mock file content',
    commit_hash: 'abc123',
    commit_message: 'Mock commit',
  }),

  // 搜索相关
  search_messages: () => [],

  // 导入相关
  get_imported_session_ids: () => [],
};

export async function mockInvoke(
  cmd: string,
  args?: Record<string, unknown>
): Promise<unknown> {
  const handler = mockHandlers[cmd];
  if (!handler) {
    console.warn(`[IPC Mock] No handler for: ${cmd}`);
    return null;
  }

  // 模拟网络延迟
  await new Promise(resolve => setTimeout(resolve, 10));
  return handler(args);
}
```

#### 3. 测试环境注入

```typescript
// src/main.tsx
import { setMockInvoke } from '@/lib/ipc-adapter';

// 在测试环境注入 Mock
if (import.meta.env.DEV) {
  const params = new URLSearchParams(window.location.search);
  if (params.has('playwright')) {
    (window as any).__PLAYWRIGHT_TEST__ = true;

    // 动态加载 mock（避免生产环境打包）
    import('../e2e/fixtures/ipc-mock').then(({ mockInvoke }) => {
      setMockInvoke(mockInvoke);
    });
  }
}
```

## Playwright 配置

```typescript
// e2e/playwright.config.ts
import { defineConfig, devices } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: [
    ['html', { open: 'never' }],
    ['list'],
  ],

  use: {
    baseURL: 'http://localhost:5173?playwright',
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
    // Firefox 可选
    // {
    //   name: 'firefox',
    //   use: { ...devices['Desktop Firefox'] },
    // },
  ],

  // 自动启动 Vite dev server
  webServer: {
    command: 'pnpm dev',
    url: 'http://localhost:5173',
    reuseExistingServer: !process.env.CI,
    timeout: 120 * 1000,
  },
});
```

## 测试用例设计

### P0 - 核心流程（必须覆盖）

#### 会话播放器

```typescript
// e2e/tests/player.spec.ts
import { test, expect } from '@playwright/test';

test.describe('会话播放器', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/player/mock-session-1');
  });

  test('正确渲染消息列表', async ({ page }) => {
    const messageList = page.locator('[data-testid="message-list"]');
    await expect(messageList).toBeVisible();

    const messages = page.locator('[data-testid="message-item"]');
    await expect(messages).toHaveCount(10); // 预期 Mock 数据量
  });

  test('点击消息高亮并滚动', async ({ page }) => {
    const firstMessage = page.locator('[data-testid="message-item"]').first();
    await firstMessage.click();

    await expect(firstMessage).toHaveClass(/selected/);
  });

  test('时间轴交互', async ({ page }) => {
    const timeline = page.locator('[data-testid="timeline"]');
    await expect(timeline).toBeVisible();

    // 拖动时间轴
    const slider = timeline.locator('[data-testid="timeline-slider"]');
    await slider.dragTo(timeline, { targetPosition: { x: 100, y: 0 } });
  });

  test('代码面板显示正确', async ({ page }) => {
    const codePanel = page.locator('[data-testid="code-panel"]');
    await expect(codePanel).toBeVisible();

    // 验证 Monaco 编辑器加载
    const editor = page.locator('.monaco-editor');
    await expect(editor).toBeVisible();
  });
});
```

#### 导入向导

```typescript
// e2e/tests/import.spec.ts
import { test, expect } from '@playwright/test';

test.describe('导入向导', () => {
  test('显示导入入口', async ({ page }) => {
    await page.goto('/');

    const importButton = page.locator('[data-testid="import-button"]');
    await expect(importButton).toBeVisible();
  });

  test('导入流程完整性', async ({ page }) => {
    await page.goto('/');

    // 打开导入向导
    await page.click('[data-testid="import-button"]');

    // 验证步骤 1: 选择来源
    await expect(page.locator('text=选择导入来源')).toBeVisible();

    // 选择 Claude
    await page.click('[data-testid="source-claude"]');

    // 验证步骤 2: 选择会话
    await expect(page.locator('text=选择会话')).toBeVisible();
  });
});
```

#### 导航

```typescript
// e2e/tests/navigation.spec.ts
import { test, expect } from '@playwright/test';

test.describe('导航', () => {
  test('侧边栏项目列表', async ({ page }) => {
    await page.goto('/player/mock-session-1');

    // 打开侧边栏
    const drawerToggle = page.locator('[data-testid="drawer-toggle"]');
    await drawerToggle.click();

    // 验证项目列表
    const projectList = page.locator('[data-testid="project-list"]');
    await expect(projectList).toBeVisible();
  });

  test('面包屑导航', async ({ page }) => {
    await page.goto('/player/mock-session-1');

    const breadcrumb = page.locator('[data-testid="breadcrumb"]');
    await expect(breadcrumb).toContainText('Mock Project');
  });

  test('会话切换', async ({ page }) => {
    await page.goto('/player/mock-session-1');

    // 打开会话列表
    await page.click('[data-testid="session-dropdown"]');

    // 选择另一个会话
    await page.click('[data-testid="session-item-2"]');

    // 验证 URL 变化
    await expect(page).toHaveURL(/mock-session-2/);
  });
});
```

### P1 - 重要功能

#### 搜索功能

```typescript
// e2e/tests/search.spec.ts
import { test, expect } from '@playwright/test';

test.describe('搜索功能', () => {
  test('全局搜索打开', async ({ page }) => {
    await page.goto('/player/mock-session-1');

    // Cmd/Ctrl + K 打开搜索
    await page.keyboard.press('Meta+k');

    const searchDialog = page.locator('[data-testid="global-search"]');
    await expect(searchDialog).toBeVisible();
  });

  test('搜索结果展示', async ({ page }) => {
    await page.goto('/player/mock-session-1');

    await page.keyboard.press('Meta+k');
    await page.fill('[data-testid="search-input"]', 'test query');

    // 等待搜索结果
    const results = page.locator('[data-testid="search-results"]');
    await expect(results).toBeVisible();
  });
});
```

### P2 - 视觉回归测试

```typescript
// e2e/tests/visual.spec.ts
import { test, expect } from '@playwright/test';

test.describe('视觉回归', () => {
  test('播放器页面 - 默认状态', async ({ page }) => {
    await page.goto('/player/mock-session-1');
    await page.waitForLoadState('networkidle');

    await expect(page).toHaveScreenshot('player-default.png', {
      maxDiffPixels: 100, // 允许微小差异
    });
  });

  test('播放器页面 - 消息选中状态', async ({ page }) => {
    await page.goto('/player/mock-session-1');
    await page.click('[data-testid="message-item"]:first-child');

    await expect(page).toHaveScreenshot('player-message-selected.png');
  });

  test('设置页面', async ({ page }) => {
    await page.goto('/settings');
    await page.waitForLoadState('networkidle');

    await expect(page).toHaveScreenshot('settings.png');
  });

  test('空状态页面', async ({ page }) => {
    // Mock 无项目状态
    await page.goto('/?empty=true');

    await expect(page).toHaveScreenshot('empty-state.png');
  });
});
```

## NPM 脚本

在 `package.json` 中添加：

```json
{
  "scripts": {
    "test:e2e": "playwright test",
    "test:e2e:ui": "playwright test --ui",
    "test:e2e:debug": "playwright test --debug",
    "test:e2e:headed": "playwright test --headed",
    "test:e2e:update-snapshots": "playwright test --update-snapshots",
    "test:e2e:report": "playwright show-report"
  }
}
```

## CI 集成

### GitHub Actions

```yaml
# .github/workflows/e2e.yml
name: E2E Tests

on:
  pull_request:
    paths:
      - 'apps/client/**'

jobs:
  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: pnpm/action-setup@v2
        with:
          version: 9

      - uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: 'pnpm'

      - name: Install dependencies
        run: pnpm install
        working-directory: apps/client

      - name: Install Playwright browsers
        run: pnpm exec playwright install --with-deps
        working-directory: apps/client

      - name: Run E2E tests
        run: pnpm test:e2e
        working-directory: apps/client

      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: playwright-report
          path: apps/client/playwright-report/
          retention-days: 7
```

## 实施路线图

### Phase 1: 基础设施（预计 1 个 Story）

- [ ] 安装 Playwright 依赖
- [ ] 创建目录结构
- [ ] 实现 IPC 适配器
- [ ] 配置 Playwright
- [ ] 编写基础 Mock 数据

### Phase 2: 核心测试（预计 2 个 Story）

- [ ] Player 页面测试
- [ ] 导入向导测试
- [ ] 导航测试
- [ ] Page Object 封装

### Phase 3: 视觉测试（预计 1 个 Story）

- [ ] 视觉回归测试用例
- [ ] 截图基线建立
- [ ] 差异阈值调优

### Phase 4: CI 集成（预计 0.5 个 Story）

- [ ] GitHub Actions 配置
- [ ] 报告归档
- [ ] PR 检查集成

## 参考资源

- [Playwright 官方文档](https://playwright.dev/docs/intro)
- [Tauri Testing Guide](https://tauri.app/v1/guides/testing/)
- [Page Object Model](https://playwright.dev/docs/pom)
- [Visual Comparisons](https://playwright.dev/docs/test-snapshots)
