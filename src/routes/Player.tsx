/**
 * Player Page - 会话回放页面
 * Story 2.8: Task 1
 *
 * 封装 DualStreamLayout，用于播放会话内容
 * 
 * 从后端加载真实会话数据并转换为前端格式
 */

import * as React from "react";
import { useParams, useNavigate } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { ThemeToggle } from "@/components/theme-toggle";
import { DualStreamLayout, type DualStreamLayoutRef } from "@/components/layout";
import { convertSessionToMessages, type MantraSession } from "@/lib/session-utils";
import type { NarrativeMessage } from "@/types/message";
import {
  messagesToTimelineEvents,
  getTimelineRange,
  type TimelineEvent,
} from "@/types/timeline";
import { TimberLine } from "@/components/timeline";
import { ArrowLeft, AlertCircle, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";

/**
 * Player 页面组件
 * 展示会话回放的 DualStreamLayout
 */
export default function Player() {
  const { sessionId } = useParams<{ sessionId: string }>();
  const navigate = useNavigate();

  // DualStreamLayout ref 用于程序化滚动
  const layoutRef = React.useRef<DualStreamLayoutRef>(null);

  // 选中的消息 ID
  const [selectedMessageId, setSelectedMessageId] = React.useState<string | undefined>();

  // 会话数据状态
  const [messages, setMessages] = React.useState<NarrativeMessage[]>([]);
  const [loading, setLoading] = React.useState(true);
  const [error, setError] = React.useState<string | null>(null);
  const [sessionCwd, setSessionCwd] = React.useState<string | undefined>();

  // 时间轴状态 (Story 2.6)
  const [timelineEvents, setTimelineEvents] = React.useState<TimelineEvent[]>([]);
  const [timelineRange, setTimelineRange] = React.useState<{ startTime: number; endTime: number }>({
    startTime: Date.now(),
    endTime: Date.now(),
  });
  const [currentTime, setCurrentTime] = React.useState<number>(Date.now());

  // 加载会话数据
  React.useEffect(() => {
    if (!sessionId) {
      setLoading(false);
      return;
    }

    let cancelled = false;

    async function loadSession() {
      try {
        setLoading(true);
        setError(null);

        const session = await invoke<MantraSession | null>("get_session", {
          sessionId,
        });

        if (cancelled) return;

        if (!session) {
          setError("会话不存在");
          setMessages([]);
        } else {
          const narrativeMessages = convertSessionToMessages(session);
          setMessages(narrativeMessages);
          setSessionCwd(session.cwd);

          // 计算时间轴数据 (Story 2.6)
          const events = messagesToTimelineEvents(narrativeMessages);
          setTimelineEvents(events);
          const range = getTimelineRange(events);
          setTimelineRange(range);
          // 初始化当前时间为第一条消息的时间
          setCurrentTime(range.startTime);
        }
      } catch (err) {
        if (cancelled) return;
        console.error("Failed to load session:", err);
        setError(err instanceof Error ? err.message : "加载会话失败");
      } finally {
        if (!cancelled) {
          setLoading(false);
        }
      }
    }

    loadSession();

    return () => {
      cancelled = true;
    };
  }, [sessionId]);

  // 消息选中回调
  const handleMessageSelect = React.useCallback(
    (messageId: string, message: NarrativeMessage) => {
      setSelectedMessageId(messageId);
      // 同步更新时间轴位置
      const msgTime = new Date(message.timestamp).getTime();
      setCurrentTime(msgTime);
    },
    []
  );

  // 时间轴 Seek 回调 (Story 2.6)
  const handleTimelineSeek = React.useCallback(
    (timestamp: number) => {
      setCurrentTime(timestamp);
      // 找到最近的消息并选中
      const nearestEvent = timelineEvents.reduce<TimelineEvent | null>((nearest, event) => {
        if (!nearest) return event;
        const currentDiff = Math.abs(event.timestamp - timestamp);
        const nearestDiff = Math.abs(nearest.timestamp - timestamp);
        return currentDiff < nearestDiff ? event : nearest;
      }, null);

      if (nearestEvent && nearestEvent.messageIndex !== undefined) {
        const msg = messages[nearestEvent.messageIndex];
        if (msg) {
          setSelectedMessageId(msg.id);
          layoutRef.current?.scrollToMessage(msg.id);
        }
      }
    },
    [timelineEvents, messages]
  );

  // 返回 Dashboard
  const handleBack = React.useCallback(() => {
    navigate("/");
  }, [navigate]);

  // DEV: 滚动到指定消息的调试函数 (可在 DevTools console 中调用 scrollToMessage('id'))
  React.useEffect(() => {
    if (import.meta.env.DEV) {
      (window as unknown as { scrollToMessage: (id: string) => void }).scrollToMessage = (id: string) => {
        layoutRef.current?.scrollToMessage(id);
      };
    }
    return () => {
      if (import.meta.env.DEV) {
        delete (window as unknown as { scrollToMessage?: unknown }).scrollToMessage;
      }
    };
  }, []);

  // 无效 sessionId 错误处理
  if (!sessionId) {
    return (
      <div className="h-screen flex flex-col bg-background">
        <header className="shrink-0 sticky top-0 z-50 w-full border-b border-border bg-background/95 backdrop-blur">
          <div className="flex h-14 items-center px-4">
            <Button variant="ghost" size="icon" onClick={handleBack}>
              <ArrowLeft className="h-5 w-5" />
            </Button>
            <span className="text-xl font-bold text-foreground ml-2">
              Mantra <span className="text-primary">心法</span>
            </span>
          </div>
        </header>
        <main className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <AlertCircle className="w-12 h-12 text-destructive mx-auto mb-4" />
            <h2 className="text-lg font-semibold text-foreground mb-2">会话未找到</h2>
            <p className="text-sm text-muted-foreground mb-4">请从项目列表中选择一个会话</p>
            <Button onClick={handleBack}>返回 Dashboard</Button>
          </div>
        </main>
      </div>
    );
  }

  // 加载中状态
  if (loading) {
    return (
      <div className="h-screen flex flex-col bg-background">
        <header className="shrink-0 sticky top-0 z-50 w-full border-b border-border bg-background/95 backdrop-blur">
          <div className="flex h-14 items-center px-4">
            <Button variant="ghost" size="icon" onClick={handleBack}>
              <ArrowLeft className="h-5 w-5" />
            </Button>
            <span className="text-xl font-bold text-foreground ml-2">
              Mantra <span className="text-primary">心法</span>
            </span>
          </div>
        </header>
        <main className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <Loader2 className="w-12 h-12 text-primary mx-auto mb-4 animate-spin" />
            <p className="text-sm text-muted-foreground">加载会话中...</p>
          </div>
        </main>
      </div>
    );
  }

  // 错误状态
  if (error) {
    return (
      <div className="h-screen flex flex-col bg-background">
        <header className="shrink-0 sticky top-0 z-50 w-full border-b border-border bg-background/95 backdrop-blur">
          <div className="flex h-14 items-center px-4">
            <Button variant="ghost" size="icon" onClick={handleBack}>
              <ArrowLeft className="h-5 w-5" />
            </Button>
            <span className="text-xl font-bold text-foreground ml-2">
              Mantra <span className="text-primary">心法</span>
            </span>
          </div>
        </header>
        <main className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <AlertCircle className="w-12 h-12 text-destructive mx-auto mb-4" />
            <h2 className="text-lg font-semibold text-foreground mb-2">加载失败</h2>
            <p className="text-sm text-muted-foreground mb-4">{error}</p>
            <Button onClick={handleBack}>返回 Dashboard</Button>
          </div>
        </main>
      </div>
    );
  }

  // 空消息状态
  if (messages.length === 0) {
    return (
      <div className="h-screen flex flex-col bg-background">
        <header className="shrink-0 sticky top-0 z-50 w-full border-b border-border bg-background/95 backdrop-blur">
          <div className="flex h-14 items-center px-4">
            <Button variant="ghost" size="icon" onClick={handleBack}>
              <ArrowLeft className="h-5 w-5" />
            </Button>
            <span className="text-xl font-bold text-foreground ml-2">
              Mantra <span className="text-primary">心法</span>
            </span>
          </div>
        </header>
        <main className="flex-1 flex items-center justify-center">
          <div className="text-center">
            <AlertCircle className="w-12 h-12 text-muted-foreground mx-auto mb-4" />
            <h2 className="text-lg font-semibold text-foreground mb-2">会话为空</h2>
            <p className="text-sm text-muted-foreground mb-4">这个会话没有任何消息</p>
            <Button onClick={handleBack}>返回 Dashboard</Button>
          </div>
        </main>
      </div>
    );
  }

  return (
    <div className="h-screen flex flex-col bg-background">
      {/* Header with Theme Toggle */}
      <header className="shrink-0 sticky top-0 z-50 w-full border-b border-border bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="flex h-14 items-center justify-between px-4">
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="icon"
              onClick={handleBack}
              className="mr-2"
            >
              <ArrowLeft className="h-5 w-5" />
            </Button>
            <span className="text-xl font-bold text-foreground">
              Mantra <span className="text-primary">心法</span>
            </span>
            {sessionCwd && (
              <span className="text-sm text-muted-foreground ml-4 font-mono truncate max-w-md">
                {sessionCwd}
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            <ThemeToggle />
          </div>
        </div>
      </header>

      {/* Main Content - DualStreamLayout */}
      <main className="flex-1 min-h-0 flex flex-col">
        <div className="flex-1 min-h-0 overflow-hidden">
          <DualStreamLayout
            ref={layoutRef}
            messages={messages}
            selectedMessageId={selectedMessageId}
            onMessageSelect={handleMessageSelect}
            // TimberLine 时间轴 Props (Story 2.6)
            showTimeline={false}
            timelineStartTime={timelineRange.startTime}
            timelineEndTime={timelineRange.endTime}
            timelineCurrentTime={currentTime}
            timelineEvents={timelineEvents}
            onTimelineSeek={handleTimelineSeek}
          />
        </div>
        {/* 直接在 Player 层渲染 TimberLine */}
        {messages.length > 0 && (
          <TimberLine
            startTime={timelineRange.startTime}
            endTime={timelineRange.endTime}
            currentTime={currentTime}
            events={timelineEvents}
            onSeek={handleTimelineSeek}
          />
        )}
      </main>
    </div>
  );
}
