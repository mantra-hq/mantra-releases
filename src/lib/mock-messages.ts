/**
 * Mock Messages - 开发用模拟数据
 * Story 2.3: Task 6.1, Story 2.4: Task 6
 *
 * 生成包含各种内容块类型的消息用于验证渲染
 */

import type { NarrativeMessage, ContentBlock } from "@/types/message";

/**
 * 示例用户提问
 */
const USER_PROMPTS = [
  "帮我实现一个虚拟化列表组件",
  "这个错误是什么意思？如何修复？",
  "能解释一下这段代码的作用吗？",
  "请重构这个函数，让它更简洁",
  "添加单元测试覆盖这个模块",
  "优化这个组件的性能",
  "实现深色/浅色主题切换",
  "如何处理这个边界情况？",
  "帮我设计一个数据模型",
  "创建一个自定义 Hook",
];

/**
 * 示例 AI 回复
 */
const AI_RESPONSES = [
  "好的，我来帮你实现这个功能。首先，我们需要安装必要的依赖...",
  "这个错误是因为类型不匹配。让我解释一下原因并提供解决方案...",
  "这段代码的作用是处理异步数据流。具体来说，它会...",
  "我建议采用以下方式重构：\n\n1. 提取公共逻辑\n2. 使用组合模式\n3. 添加适当的错误处理",
  "我会为这个模块添加全面的测试覆盖，包括：\n\n- 单元测试\n- 边界情况测试\n- 错误处理测试",
  "性能优化建议：\n\n1. 使用 React.memo 避免不必要的重渲染\n2. 使用 useMemo 缓存计算结果\n3. 实现虚拟化以处理大列表",
  "主题切换可以通过 CSS 变量实现。我们需要：\n\n1. 定义深色和浅色的变量集\n2. 创建主题切换按钮\n3. 持久化用户偏好",
  "这个边界情况可以通过添加防护性检查来处理。让我展示具体实现...",
  "基于你的需求，我建议采用以下数据模型设计...",
  "我来创建一个自定义 Hook 来封装这个逻辑，使其更易复用...",
];

/**
 * 生成随机时间戳
 */
function generateTimestamp(baseTime: Date, offsetMinutes: number): string {
  const time = new Date(baseTime.getTime() + offsetMinutes * 60 * 1000);
  return time.toISOString();
}

/**
 * 生成随机内容块
 */
function generateContentBlocks(text: string): ContentBlock[] {
  return [
    {
      type: "text",
      content: text,
    },
  ];
}

/**
 * 生成指定数量的 mock 消息
 * @param count 消息数量
 * @returns 消息数组
 */
export function generateMockMessages(count: number = 100): NarrativeMessage[] {
  const messages: NarrativeMessage[] = [];
  const baseTime = new Date("2024-12-30T10:00:00Z");

  for (let i = 0; i < count; i++) {
    const isUser = i % 2 === 0;
    const promptIndex = Math.floor(i / 2) % USER_PROMPTS.length;
    const responseIndex = Math.floor(i / 2) % AI_RESPONSES.length;

    messages.push({
      id: `msg-${i + 1}`,
      role: isUser ? "user" : "assistant",
      timestamp: generateTimestamp(baseTime, i * 2),
      content: generateContentBlocks(
        isUser ? USER_PROMPTS[promptIndex] : AI_RESPONSES[responseIndex]
      ),
    });
  }

  return messages;
}

/**
 * 预生成的 100 条 mock 消息
 */
export const MOCK_MESSAGES = generateMockMessages(100);

/**
 * 预生成的少量 mock 消息 (用于快速测试)
 */
export const MOCK_MESSAGES_SMALL = generateMockMessages(10);

// ============================================
// Story 2.4: 包含各种内容块类型的 Mock 消息
// ============================================

/**
 * 包含思维链的消息
 */
export const MOCK_THINKING_MESSAGE: NarrativeMessage = {
  id: "msg-thinking-1",
  role: "assistant",
  timestamp: "2024-12-30T10:05:00Z",
  content: [
    {
      type: "thinking",
      content: `让我分析这个问题...

1. 首先需要理解用户的需求：实现一个高性能的虚拟化列表
2. 考虑技术选型：@tanstack/react-virtual 是目前最好的选择
3. 需要处理动态高度测量
4. 还要考虑无障碍访问性

我认为最好的方案是...`,
    },
    {
      type: "text",
      content: "根据我的分析，我建议使用 @tanstack/react-virtual 来实现虚拟化列表。这个库有以下优势：\n\n- 支持动态高度\n- 内置测量功能\n- 轻量且高性能",
    },
  ],
};

/**
 * 包含工具调用的消息
 */
export const MOCK_TOOL_USE_MESSAGE: NarrativeMessage = {
  id: "msg-tooluse-1",
  role: "assistant",
  timestamp: "2024-12-30T10:06:00Z",
  content: [
    {
      type: "text",
      content: "让我读取一下这个文件来了解现有的代码结构：",
    },
    {
      type: "tool_use",
      content: "",
      toolName: "Read",
      toolInput: {
        file_path: "/src/components/App.tsx",
        offset: 0,
        limit: 100,
      },
    },
  ],
};

/**
 * 包含工具成功结果的消息
 */
export const MOCK_TOOL_RESULT_SUCCESS: NarrativeMessage = {
  id: "msg-toolresult-success-1",
  role: "assistant",
  timestamp: "2024-12-30T10:06:30Z",
  content: [
    {
      type: "tool_result",
      content: `import React from 'react';
import { ThemeProvider } from './providers/ThemeProvider';
import { DualStreamLayout } from './components/layout/DualStreamLayout';

export function App() {
  return (
    <ThemeProvider>
      <DualStreamLayout />
    </ThemeProvider>
  );
}`,
      isError: false,
    },
    {
      type: "text",
      content: "我已经读取了文件内容。这是一个典型的 React 应用入口组件，使用了 ThemeProvider 和 DualStreamLayout。",
    },
  ],
};

/**
 * 包含工具错误结果的消息
 */
export const MOCK_TOOL_RESULT_ERROR: NarrativeMessage = {
  id: "msg-toolresult-error-1",
  role: "assistant",
  timestamp: "2024-12-30T10:07:00Z",
  content: [
    {
      type: "tool_result",
      content: "Error: ENOENT: no such file or directory, open '/src/missing-file.ts'\n\nThe file does not exist at the specified path.",
      isError: true,
    },
    {
      type: "text",
      content: "抱歉，这个文件不存在。让我检查一下正确的路径...",
    },
  ],
};

/**
 * 混合内容块的消息 (思维链 + 工具调用 + 结果)
 */
export const MOCK_MIXED_CONTENT_MESSAGE: NarrativeMessage = {
  id: "msg-mixed-1",
  role: "assistant",
  timestamp: "2024-12-30T10:08:00Z",
  content: [
    {
      type: "thinking",
      content: "用户想要了解这个组件的实现细节。我需要先读取源代码，然后分析并解释关键部分。",
    },
    {
      type: "tool_use",
      content: "",
      toolName: "Read",
      toolInput: {
        file_path: "/src/components/VirtualList.tsx",
      },
    },
    {
      type: "tool_result",
      content: `export function VirtualList({ items, renderItem }) {
  const virtualizer = useVirtualizer({
    count: items.length,
    getScrollElement: () => containerRef.current,
    estimateSize: () => 50,
  });
  
  return (
    <div ref={containerRef}>
      {virtualizer.getVirtualItems().map((virtualRow) => (
        <div key={virtualRow.key}>
          {renderItem(items[virtualRow.index])}
        </div>
      ))}
    </div>
  );
}`,
      isError: false,
    },
    {
      type: "text",
      content: "这个组件使用了 `useVirtualizer` Hook 来实现虚拟化。关键点是：\n\n1. `estimateSize` 提供初始高度估算\n2. `getVirtualItems()` 返回当前可见的虚拟行\n3. 每个虚拟行都有唯一的 `key` 用于 React 渲染优化",
    },
  ],
};

/**
 * 复杂工具调用消息 (多个参数)
 */
export const MOCK_COMPLEX_TOOL_CALL: NarrativeMessage = {
  id: "msg-complex-tool-1",
  role: "assistant",
  timestamp: "2024-12-30T10:09:00Z",
  content: [
    {
      type: "text",
      content: "我来帮你创建这个新文件：",
    },
    {
      type: "tool_use",
      content: "",
      toolName: "Write",
      toolInput: {
        file_path: "/src/hooks/useTheme.ts",
        content: "export function useTheme() {\n  // Theme hook implementation\n}",
        create_intermediate_dirs: true,
        overwrite: false,
      },
    },
  ],
};

/**
 * 包含所有类型内容块的演示消息集
 * 用于 Story 2.4 验证
 */
export const MOCK_MESSAGES_WITH_ALL_TYPES: NarrativeMessage[] = [
  // 用户问题
  {
    id: "demo-1",
    role: "user",
    timestamp: "2024-12-30T10:00:00Z",
    content: [
      {
        type: "text",
        content: "帮我分析这个组件的性能问题",
      },
    ],
  },
  // AI 思考 + 回复
  MOCK_THINKING_MESSAGE,
  // 用户追问
  {
    id: "demo-2",
    role: "user",
    timestamp: "2024-12-30T10:05:30Z",
    content: [
      {
        type: "text",
        content: "可以看一下具体的代码吗？",
      },
    ],
  },
  // AI 工具调用
  MOCK_TOOL_USE_MESSAGE,
  // 工具成功结果
  MOCK_TOOL_RESULT_SUCCESS,
  // 用户继续
  {
    id: "demo-3",
    role: "user",
    timestamp: "2024-12-30T10:07:30Z",
    content: [
      {
        type: "text",
        content: "missing-file.ts 在哪里？",
      },
    ],
  },
  // 工具错误结果
  MOCK_TOOL_RESULT_ERROR,
  // 混合消息
  MOCK_MIXED_CONTENT_MESSAGE,
  // 复杂工具调用
  MOCK_COMPLEX_TOOL_CALL,
];
