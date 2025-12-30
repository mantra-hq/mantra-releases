/**
 * Mock Messages - 开发用模拟数据
 * Story 2.3: Task 6.1
 *
 * 生成 100+ 条消息用于验证虚拟化性能
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

