/**
 * Project Types - 项目和会话类型定义
 * Story 2.8: Dashboard 项目列表
 *
 * 定义项目聚合和会话相关的数据结构
 */

/**
 * 会话来源类型
 */
export type SessionSource = "claude" | "gemini" | "cursor";

/**
 * 会话摘要信息
 * 用于项目列表中的会话展示
 */
export interface Session {
  /** 会话唯一标识 */
  id: string;
  /** 会话标题 (通常是第一条用户消息的摘要) */
  title: string;
  /** 会话来源 */
  source: SessionSource;
  /** 消息数量 */
  messageCount: number;
  /** 开始时间 (Unix 毫秒时间戳) */
  startTime: number;
  /** 结束时间 (Unix 毫秒时间戳) */
  endTime: number;
}

/**
 * 项目信息
 * 聚合同一工作目录下的所有会话
 */
export interface Project {
  /** 项目唯一标识 (通常是 CWD 的 hash) */
  id: string;
  /** 项目名称 (目录名) */
  name: string;
  /** 项目路径 */
  path: string;
  /** 包含的会话列表 */
  sessions: Session[];
  /** 最后活动时间 (Unix 毫秒时间戳) */
  lastActivity: number;
}

